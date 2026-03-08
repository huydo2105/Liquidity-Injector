"use client";

import React, { useEffect, useRef, useState } from "react";
import * as d3 from "d3";

interface Node {
    id: string;
    label: string;
    x?: number;
    y?: number;
    fx?: number | null;
    fy?: number | null;
}

interface Edge {
    source: string;
    target: string;
    amount: number;
    inCycle: boolean;
}

interface CycleVisualizerProps {
    nodes?: Node[];
    edges?: Edge[];
    activeCycle?: string[];
}

// Demo data
const DEMO_NODES: Node[] = [
    { id: "A", label: "Acme Corp" },
    { id: "B", label: "BuildTech" },
    { id: "C", label: "ClearPay" },
    { id: "D", label: "DataFlow" },
    { id: "E", label: "EnviroServ" },
];

const DEMO_EDGES: Edge[] = [
    { source: "A", target: "B", amount: 15000, inCycle: true },
    { source: "B", target: "C", amount: 22000, inCycle: true },
    { source: "C", target: "A", amount: 18000, inCycle: true },
    { source: "C", target: "D", amount: 30000, inCycle: true },
    { source: "D", target: "E", amount: 25000, inCycle: true },
    { source: "E", target: "C", amount: 20000, inCycle: true },
    { source: "A", target: "D", amount: 8000, inCycle: false },
];

export default function CycleVisualizer({
    nodes = DEMO_NODES,
    edges = DEMO_EDGES,
    activeCycle,
}: CycleVisualizerProps) {
    const svgRef = useRef<SVGSVGElement>(null);
    const [dimensions, setDimensions] = useState({ width: 600, height: 450 });

    useEffect(() => {
        const container = svgRef.current?.parentElement;
        if (container) {
            const rect = container.getBoundingClientRect();
            setDimensions({ width: rect.width, height: Math.max(rect.height, 450) });
        }
    }, []);

    useEffect(() => {
        if (!svgRef.current) return;

        const svg = d3.select(svgRef.current);
        svg.selectAll("*").remove();

        const { width, height } = dimensions;

        // Defs for gradients and arrows
        const defs = svg.append("defs");

        // Arrow marker for edges
        defs
            .append("marker")
            .attr("id", "arrowhead")
            .attr("viewBox", "0 -5 10 10")
            .attr("refX", 28)
            .attr("refY", 0)
            .attr("markerWidth", 8)
            .attr("markerHeight", 8)
            .attr("orient", "auto")
            .append("path")
            .attr("d", "M0,-5L10,0L0,5")
            .attr("fill", "#3b82f6");

        defs
            .append("marker")
            .attr("id", "arrowhead-muted")
            .attr("viewBox", "0 -5 10 10")
            .attr("refX", 28)
            .attr("refY", 0)
            .attr("markerWidth", 8)
            .attr("markerHeight", 8)
            .attr("orient", "auto")
            .append("path")
            .attr("d", "M0,-5L10,0L0,5")
            .attr("fill", "#334155");

        // Glow filter
        const filter = defs
            .append("filter")
            .attr("id", "glow")
            .attr("x", "-50%")
            .attr("y", "-50%")
            .attr("width", "200%")
            .attr("height", "200%");
        filter
            .append("feGaussianBlur")
            .attr("stdDeviation", "4")
            .attr("result", "blur");
        filter
            .append("feMerge")
            .selectAll("feMergeNode")
            .data(["blur", "SourceGraphic"])
            .enter()
            .append("feMergeNode")
            .attr("in", (d) => d);

        // Force simulation
        const simNodes = nodes.map((n) => ({ ...n }));
        const simEdges = edges.map((e) => ({ ...e }));

        const simulation = d3
            .forceSimulation(simNodes as d3.SimulationNodeDatum[])
            .force(
                "link",
                d3
                    .forceLink(simEdges)
                    .id((d: any) => d.id)
                    .distance(140)
            )
            .force("charge", d3.forceManyBody().strength(-500))
            .force("center", d3.forceCenter(width / 2, height / 2))
            .force("collision", d3.forceCollide(50));

        const g = svg.append("g");

        // Edges
        const link = g
            .selectAll(".link")
            .data(simEdges)
            .enter()
            .append("g")
            .attr("class", "link");

        const linkLine = link
            .append("path")
            .attr("stroke", (d: any) => (d.inCycle ? "#3b82f6" : "#1e293b"))
            .attr("stroke-width", (d: any) => (d.inCycle ? 2.5 : 1.5))
            .attr("fill", "none")
            .attr("marker-end", (d: any) =>
                d.inCycle ? "url(#arrowhead)" : "url(#arrowhead-muted)"
            )
            .attr("filter", (d: any) => (d.inCycle ? "url(#glow)" : "none"))
            .attr("opacity", (d: any) => (d.inCycle ? 1 : 0.3));

        // Edge labels (amounts)
        const linkLabel = link
            .append("text")
            .text((d: any) => `$${(d.amount / 1000).toFixed(0)}k`)
            .attr("fill", (d: any) => (d.inCycle ? "#94a3b8" : "#475569"))
            .attr("font-size", "11px")
            .attr("font-family", "'JetBrains Mono', monospace")
            .attr("text-anchor", "middle")
            .attr("dy", -8);

        // Animated flow particles on cycle edges
        const particles = g
            .selectAll(".particle")
            .data(simEdges.filter((e: any) => e.inCycle))
            .enter()
            .append("circle")
            .attr("r", 3)
            .attr("fill", "#06b6d4")
            .attr("filter", "url(#glow)")
            .attr("opacity", 0.9);

        // Nodes
        const node = g
            .selectAll(".node")
            .data(simNodes)
            .enter()
            .append("g")
            .attr("class", "node")
            .style("cursor", "grab")
            .call(
                d3
                    .drag<SVGGElement, any>()
                    .on("start", (event, d) => {
                        if (!event.active) simulation.alphaTarget(0.3).restart();
                        d.fx = d.x;
                        d.fy = d.y;
                    })
                    .on("drag", (event, d) => {
                        d.fx = event.x;
                        d.fy = event.y;
                    })
                    .on("end", (event, d) => {
                        if (!event.active) simulation.alphaTarget(0);
                        d.fx = null;
                        d.fy = null;
                    })
            );

        // Node circles
        node
            .append("circle")
            .attr("r", 24)
            .attr("fill", "rgba(17, 24, 39, 0.9)")
            .attr("stroke", "#3b82f6")
            .attr("stroke-width", 2)
            .attr("filter", "url(#glow)");

        // Node labels
        node
            .append("text")
            .text((d: any) => d.id)
            .attr("fill", "#f1f5f9")
            .attr("font-size", "14px")
            .attr("font-weight", "600")
            .attr("text-anchor", "middle")
            .attr("dy", 5);

        // Company labels
        node
            .append("text")
            .text((d: any) => d.label)
            .attr("fill", "#94a3b8")
            .attr("font-size", "10px")
            .attr("text-anchor", "middle")
            .attr("dy", 42);

        // Tick animation
        let tickCount = 0;
        simulation.on("tick", () => {
            tickCount++;

            linkLine.attr("d", (d: any) => {
                const dx = d.target.x - d.source.x;
                const dy = d.target.y - d.source.y;
                return `M${d.source.x},${d.source.y} L${d.target.x},${d.target.y}`;
            });

            linkLabel
                .attr("x", (d: any) => (d.source.x + d.target.x) / 2)
                .attr("y", (d: any) => (d.source.y + d.target.y) / 2);

            node.attr("transform", (d: any) => `translate(${d.x},${d.y})`);

            // Animate particles along cycle edges
            particles.each(function (this: SVGCircleElement, d: any) {
                const t = ((tickCount * 0.02) % 1);
                const sx = d.source.x ?? 0;
                const sy = d.source.y ?? 0;
                const tx = d.target.x ?? 0;
                const ty = d.target.y ?? 0;
                d3.select(this)
                    .attr("cx", sx + (tx - sx) * t)
                    .attr("cy", sy + (ty - sy) * t);
            });
        });

        return () => {
            simulation.stop();
        };
    }, [nodes, edges, dimensions]);

    return (
        <div className="glass-card p-6 h-full">
            <div className="flex items-center justify-between mb-4">
                <div>
                    <h2 className="text-lg font-semibold text-white">Debt Cycle Discovery</h2>
                    <p className="text-sm text-slate-400 mt-1">
                        Encrypted obligations decrypted inside TEE • MTCS flow optimization
                    </p>
                </div>
                <div className="flex items-center gap-4 text-xs">
                    <div className="flex items-center gap-2">
                        <div className="w-3 h-0.5 bg-blue-500 rounded" />
                        <span className="text-slate-400">In Cycle</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <div className="w-3 h-0.5 bg-slate-700 rounded" />
                        <span className="text-slate-400">Non-cycle</span>
                    </div>
                </div>
            </div>
            <svg
                ref={svgRef}
                width="100%"
                height="450"
                viewBox={`0 0 ${dimensions.width} ${dimensions.height}`}
                style={{ maxWidth: "100%" }}
            />
        </div>
    );
}
