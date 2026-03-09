"use client";

import React from "react";
import QuorumTracker from "@/components/QuorumTracker";
import { motion } from "framer-motion";

export default function ConsensusPage() {
    return (
        <main className="max-w-4xl mx-auto w-full px-6 py-8 space-y-6 flex-1">
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h1 className="text-2xl font-bold text-white tracking-tight">BFT Quorum</h1>
                    <p className="text-sm text-zinc-400 mt-1">Real-time status of 2f+1 Validator Network.</p>
                </div>
            </div>

            <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}>
                <QuorumTracker />
            </motion.div>
        </main>
    );
}
