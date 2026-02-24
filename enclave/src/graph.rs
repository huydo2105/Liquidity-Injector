use std::collections::{HashMap, HashSet};

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;

use crate::types::{Cycle, FlowElement, FlowSolution, Obligation, ParticipantId};

/// Errors from graph operations.
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("No cycles found in the debt graph")]
    NoCyclesFound,
    #[error("Invalid graph: {0}")]
    InvalidGraph(String),
    #[error("Flow validation failed: {0}")]
    FlowValidationFailed(String),
}

/// Build a directed debt graph from a list of obligations.
/// Returns the graph and a mapping from ParticipantId to NodeIndex.
pub fn build_debt_graph(
    obligations: &[Obligation],
) -> (DiGraph<ParticipantId, u128>, HashMap<ParticipantId, NodeIndex>) {
    let mut graph = DiGraph::new();
    let mut node_map: HashMap<ParticipantId, NodeIndex> = HashMap::new();

    // Add all unique participants as nodes
    for ob in obligations {
        if !node_map.contains_key(&ob.debtor) {
            let idx = graph.add_node(ob.debtor.clone());
            node_map.insert(ob.debtor.clone(), idx);
        }
        if !node_map.contains_key(&ob.creditor) {
            let idx = graph.add_node(ob.creditor.clone());
            node_map.insert(ob.creditor.clone(), idx);
        }
    }

    // Add edges (debtor → creditor with amount as weight)
    for ob in obligations {
        let from = node_map[&ob.debtor];
        let to = node_map[&ob.creditor];
        graph.add_edge(from, to, ob.amount);
    }

    (graph, node_map)
}

/// Find all elementary cycles in the debt graph using Johnson's algorithm.
/// Returns cycles sorted by clearable amount (descending).
pub fn find_all_cycles(
    graph: &DiGraph<ParticipantId, u128>,
    node_map: &HashMap<ParticipantId, NodeIndex>,
) -> Vec<Cycle> {
    let reverse_map: HashMap<NodeIndex, &ParticipantId> =
        node_map.iter().map(|(k, v)| (*v, k)).collect();

    let mut all_cycles: Vec<Vec<NodeIndex>> = Vec::new();

    // Johnson's algorithm: find all elementary circuits
    let node_count = graph.node_count();
    let nodes: Vec<NodeIndex> = graph.node_indices().collect();

    for start_idx in 0..node_count {
        let start = nodes[start_idx];
        let mut stack: Vec<NodeIndex> = vec![start];
        let mut blocked: HashSet<NodeIndex> = HashSet::new();
        let mut block_map: HashMap<NodeIndex, HashSet<NodeIndex>> = HashMap::new();
        blocked.insert(start);

        johnson_circuit(
            graph,
            start,
            &mut stack,
            &mut blocked,
            &mut block_map,
            &mut all_cycles,
            &nodes[start_idx..],
        );
    }

    // Convert NodeIndex cycles to typed Cycle structs
    let mut cycles: Vec<Cycle> = all_cycles
        .into_iter()
        .filter_map(|node_cycle| {
            if node_cycle.len() < 2 {
                return None;
            }

            let participants: Vec<ParticipantId> = node_cycle
                .iter()
                .map(|n| reverse_map[n].clone())
                .collect();

            // Find the edges forming this cycle and the bottleneck
            let mut edges = Vec::new();
            let mut min_amount = u128::MAX;

            for i in 0..node_cycle.len() {
                let from = node_cycle[i];
                let to = node_cycle[(i + 1) % node_cycle.len()];

                if let Some(edge) = graph.find_edge(from, to) {
                    let amount = *graph.edge_weight(edge).unwrap();
                    min_amount = min_amount.min(amount);
                    edges.push(Obligation {
                        id: edge.index() as u64,
                        debtor: reverse_map[&from].clone(),
                        creditor: reverse_map[&to].clone(),
                        amount,
                    });
                } else {
                    return None; // Edge missing, invalid cycle
                }
            }

            Some(Cycle {
                participants,
                edges,
                clearable_amount: min_amount,
            })
        })
        .collect();

    // Sort by clearable amount descending (best cycles first)
    cycles.sort_by(|a, b| b.clearable_amount.cmp(&a.clearable_amount));
    cycles
}

/// Recursive part of Johnson's algorithm.
fn johnson_circuit(
    graph: &DiGraph<ParticipantId, u128>,
    start: NodeIndex,
    stack: &mut Vec<NodeIndex>,
    blocked: &mut HashSet<NodeIndex>,
    block_map: &mut HashMap<NodeIndex, HashSet<NodeIndex>>,
    result: &mut Vec<Vec<NodeIndex>>,
    valid_nodes: &[NodeIndex],
) -> bool {
    let valid_set: HashSet<NodeIndex> = valid_nodes.iter().copied().collect();
    let current = *stack.last().unwrap();
    let mut found_cycle = false;

    for neighbor in graph.neighbors_directed(current, Direction::Outgoing) {
        if !valid_set.contains(&neighbor) {
            continue;
        }

        if neighbor == start {
            // Found a cycle
            result.push(stack.clone());
            found_cycle = true;
        } else if !blocked.contains(&neighbor) {
            stack.push(neighbor);
            blocked.insert(neighbor);

            if johnson_circuit(graph, start, stack, blocked, block_map, result, valid_nodes) {
                found_cycle = true;
            }

            stack.pop();
        }
    }

    if found_cycle {
        unblock(current, blocked, block_map);
    } else {
        for neighbor in graph.neighbors_directed(current, Direction::Outgoing) {
            if valid_set.contains(&neighbor) {
                block_map
                    .entry(neighbor)
                    .or_insert_with(HashSet::new)
                    .insert(current);
            }
        }
    }

    found_cycle
}

/// Unblock a node in Johnson's algorithm.
fn unblock(
    node: NodeIndex,
    blocked: &mut HashSet<NodeIndex>,
    block_map: &mut HashMap<NodeIndex, HashSet<NodeIndex>>,
) {
    blocked.remove(&node);
    if let Some(dependents) = block_map.remove(&node) {
        for dep in dependents {
            if blocked.contains(&dep) {
                unblock(dep, blocked, block_map);
            }
        }
    }
}

/// Compute a settlement flow using a simplified MTCS approach.
///
/// Strategy:
/// 1. Find all cycles in the graph
/// 2. Greedily clear cycles (largest clearable amount first)
/// 3. Use injection liquidity to clear remaining "almost-cycles" (paths
///    where only a small gap prevents full clearing)
///
/// Returns a balanced FlowSolution where for each node: flow_in == flow_out.
pub fn compute_mtcs_flow(
    obligations: &[Obligation],
    injection_amount: u128,
) -> Result<FlowSolution, GraphError> {
    let (graph, node_map) = build_debt_graph(obligations);
    let cycles = find_all_cycles(&graph, &node_map);

    if cycles.is_empty() && injection_amount == 0 {
        return Err(GraphError::NoCyclesFound);
    }

    let mut flows: Vec<FlowElement> = Vec::new();
    let mut total_cleared: u128 = 0;
    let mut injection_used: u128 = 0;

    // Track remaining obligation amounts
    let mut remaining: HashMap<(ParticipantId, ParticipantId), u128> = obligations
        .iter()
        .map(|o| ((o.debtor.clone(), o.creditor.clone()), o.amount))
        .collect();

    // Phase 1: Clear pure cycles (no injection needed)
    for cycle in &cycles {
        // Find the bottleneck amount for this cycle given remaining amounts
        let bottleneck = cycle
            .edges
            .iter()
            .filter_map(|e| remaining.get(&(e.debtor.clone(), e.creditor.clone())))
            .min()
            .copied()
            .unwrap_or(0);

        if bottleneck == 0 {
            continue;
        }

        // Clear the cycle at the bottleneck amount
        for edge in &cycle.edges {
            let key = (edge.debtor.clone(), edge.creditor.clone());
            if let Some(rem) = remaining.get_mut(&key) {
                *rem = rem.saturating_sub(bottleneck);
            }
            flows.push(FlowElement {
                debtor: edge.debtor.clone(),
                creditor: edge.creditor.clone(),
                amount: bottleneck,
            });
        }
        total_cleared += bottleneck * cycle.edges.len() as u128;
    }

    // Phase 2: Use injection liquidity for near-cycles
    // Find paths where a small injection can complete a cycle
    if injection_amount > 0 {
        let mut remaining_injection = injection_amount;

        for cycle in &cycles {
            if remaining_injection == 0 {
                break;
            }

            // Check if there's remaining debt in this cycle that needs a top-up
            let min_remaining = cycle
                .edges
                .iter()
                .filter_map(|e| {
                    let key = (e.debtor.clone(), e.creditor.clone());
                    remaining.get(&key).copied()
                })
                .min()
                .unwrap_or(0);

            if min_remaining > 0 {
                let clearable = min_remaining.min(remaining_injection);
                if clearable > 0 {
                    for edge in &cycle.edges {
                        let key = (edge.debtor.clone(), edge.creditor.clone());
                        if let Some(rem) = remaining.get_mut(&key) {
                            *rem = rem.saturating_sub(clearable);
                        }
                        flows.push(FlowElement {
                            debtor: edge.debtor.clone(),
                            creditor: edge.creditor.clone(),
                            amount: clearable,
                        });
                    }
                    remaining_injection -= clearable;
                    injection_used += clearable;
                    total_cleared += clearable * cycle.edges.len() as u128;
                }
            }
        }
    }

    // Merge duplicate flows (same debtor→creditor pair)
    let mut merged: HashMap<(ParticipantId, ParticipantId), u128> = HashMap::new();
    for flow in &flows {
        *merged
            .entry((flow.debtor.clone(), flow.creditor.clone()))
            .or_insert(0) += flow.amount;
    }

    let merged_flows: Vec<FlowElement> = merged
        .into_iter()
        .filter(|(_, amount)| *amount > 0)
        .map(|((debtor, creditor), amount)| FlowElement {
            debtor,
            creditor,
            amount,
        })
        .collect();

    Ok(FlowSolution {
        flows: merged_flows,
        total_cleared,
        injection_used,
    })
}

/// Validate that a flow solution satisfies the protocol invariants:
/// 1. F ⊆ G: each flow element has a corresponding obligation
/// 2. 0 < f.amount <= g.amount for each flow
/// 3. Balanced flow: for each node, sum(flow_in) == sum(flow_out)
pub fn validate_flow(
    flow: &FlowSolution,
    obligations: &[Obligation],
) -> Result<(), GraphError> {
    let obligation_map: HashMap<(ParticipantId, ParticipantId), u128> = obligations
        .iter()
        .map(|o| ((o.debtor.clone(), o.creditor.clone()), o.amount))
        .collect();

    // Check F ⊆ G and amount bounds
    for f in &flow.flows {
        let key = (f.debtor.clone(), f.creditor.clone());
        match obligation_map.get(&key) {
            None => {
                return Err(GraphError::FlowValidationFailed(format!(
                    "Flow {}->{} has no corresponding obligation",
                    f.debtor, f.creditor
                )));
            }
            Some(&max_amount) => {
                if f.amount == 0 || f.amount > max_amount {
                    return Err(GraphError::FlowValidationFailed(format!(
                        "Flow {}->{}: amount {} not in (0, {}]",
                        f.debtor, f.creditor, f.amount, max_amount
                    )));
                }
            }
        }
    }

    // Check balanced flow: for each node, flow_in == flow_out
    let mut net_flow: HashMap<ParticipantId, i128> = HashMap::new();
    for f in &flow.flows {
        *net_flow.entry(f.debtor.clone()).or_insert(0) -= f.amount as i128;
        *net_flow.entry(f.creditor.clone()).or_insert(0) += f.amount as i128;
    }

    for (node, net) in &net_flow {
        if *net != 0 {
            return Err(GraphError::FlowValidationFailed(format!(
                "Node {} has unbalanced flow: net = {}",
                node, net
            )));
        }
    }

    Ok(())
}

/// Find the single best cycle (highest clearable amount).
pub fn find_best_cycle(
    obligations: &[Obligation],
) -> Result<Cycle, GraphError> {
    let (graph, node_map) = build_debt_graph(obligations);
    let cycles = find_all_cycles(&graph, &node_map);
    cycles.into_iter().next().ok_or(GraphError::NoCyclesFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a simple 3-node cycle A→B→C→A
    fn three_node_cycle() -> Vec<Obligation> {
        vec![
            Obligation { id: 0, debtor: "A".into(), creditor: "B".into(), amount: 100 },
            Obligation { id: 1, debtor: "B".into(), creditor: "C".into(), amount: 150 },
            Obligation { id: 2, debtor: "C".into(), creditor: "A".into(), amount: 200 },
        ]
    }

    #[test]
    fn test_build_graph() {
        let obs = three_node_cycle();
        let (graph, node_map) = build_debt_graph(&obs);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 3);
        assert!(node_map.contains_key("A"));
        assert!(node_map.contains_key("B"));
        assert!(node_map.contains_key("C"));
    }

    #[test]
    fn test_find_cycles_simple() {
        let obs = three_node_cycle();
        let (graph, node_map) = build_debt_graph(&obs);
        let cycles = find_all_cycles(&graph, &node_map);

        assert!(!cycles.is_empty(), "Should find at least one cycle");
        // The bottleneck should be 100 (min of 100, 150, 200)
        let best = &cycles[0];
        assert_eq!(best.clearable_amount, 100);
        assert_eq!(best.participants.len(), 3);
    }

    #[test]
    fn test_no_cycles() {
        // Linear chain: A→B→C (no cycle)
        let obs = vec![
            Obligation { id: 0, debtor: "A".into(), creditor: "B".into(), amount: 100 },
            Obligation { id: 1, debtor: "B".into(), creditor: "C".into(), amount: 100 },
        ];
        let (graph, node_map) = build_debt_graph(&obs);
        let cycles = find_all_cycles(&graph, &node_map);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_mtcs_flow_simple_cycle() {
        let obs = three_node_cycle();
        let flow = compute_mtcs_flow(&obs, 0).unwrap();

        assert!(flow.total_cleared > 0);
        assert_eq!(flow.injection_used, 0);

        // Validate flow invariants
        validate_flow(&flow, &obs).unwrap();
    }

    #[test]
    fn test_mtcs_flow_with_injection() {
        let obs = three_node_cycle();
        let flow = compute_mtcs_flow(&obs, 50).unwrap();

        assert!(flow.total_cleared > 0);
        // The flow should clear at least the bottleneck (100) from pure cycles
        validate_flow(&flow, &obs).unwrap();
    }

    #[test]
    fn test_five_node_multi_cycle() {
        // A→B→C→A (cycle 1) and C→D→E→C (cycle 2)
        let obs = vec![
            Obligation { id: 0, debtor: "A".into(), creditor: "B".into(), amount: 100 },
            Obligation { id: 1, debtor: "B".into(), creditor: "C".into(), amount: 200 },
            Obligation { id: 2, debtor: "C".into(), creditor: "A".into(), amount: 150 },
            Obligation { id: 3, debtor: "C".into(), creditor: "D".into(), amount: 300 },
            Obligation { id: 4, debtor: "D".into(), creditor: "E".into(), amount: 250 },
            Obligation { id: 5, debtor: "E".into(), creditor: "C".into(), amount: 200 },
        ];

        let (graph, node_map) = build_debt_graph(&obs);
        let cycles = find_all_cycles(&graph, &node_map);

        // Should find at least 2 cycles (A-B-C and C-D-E)
        assert!(cycles.len() >= 2, "Should find at least 2 cycles, found {}", cycles.len());

        let flow = compute_mtcs_flow(&obs, 0).unwrap();
        validate_flow(&flow, &obs).unwrap();
    }

    #[test]
    fn test_validate_flow_unbalanced() {
        let obs = three_node_cycle();
        let bad_flow = FlowSolution {
            flows: vec![
                FlowElement { debtor: "A".into(), creditor: "B".into(), amount: 100 },
                // Missing B→C and C→A, so flow is unbalanced
            ],
            total_cleared: 100,
            injection_used: 0,
        };
        assert!(validate_flow(&bad_flow, &obs).is_err());
    }

    #[test]
    fn test_validate_flow_exceeds_obligation() {
        let obs = three_node_cycle();
        let bad_flow = FlowSolution {
            flows: vec![
                FlowElement { debtor: "A".into(), creditor: "B".into(), amount: 999 }, // exceeds 100
                FlowElement { debtor: "B".into(), creditor: "C".into(), amount: 999 },
                FlowElement { debtor: "C".into(), creditor: "A".into(), amount: 999 },
            ],
            total_cleared: 999,
            injection_used: 0,
        };
        assert!(validate_flow(&bad_flow, &obs).is_err());
    }
}
