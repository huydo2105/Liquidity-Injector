#!/bin/bash

# Cycles Protocol: Reproducible Paper-Grade Demonstration Script
# This orchestrator compiles and runs the end-to-end evaluation scenario
# spanning dataset ingestion, MTCS flow enclave compute, Quorum collection,
# and smart contract result reporting.

set -e

echo "=========================================================="
echo " Starting Cycles Protocol End-to-End Evaluation...        "
echo "=========================================================="
echo "Compiling crates in release mode. This may take a minute..."

# Execute the consensus runner example
cargo run -p consensus --example paper_demo --release

echo ""
echo "Paper demonstration finished successfully."
