"use client";

import React from "react";
import CycleVisualizer from "@/components/CycleVisualizer";
import SettlementPanel from "@/components/SettlementPanel";
import { motion } from "framer-motion";

export default function DashboardPage() {
    return (
        <main className="max-w-[1440px] w-full mx-auto px-6 py-8 space-y-6 flex-1">
            <div className="flex justify-between items-center mb-6">
                <h1 className="text-2xl font-bold text-white tracking-tight">Protocol Overview</h1>
                <div className="flex gap-4">
                    <button className="px-4 py-2 rounded-xl bg-gradient-to-r from-blue-600 to-violet-600 text-white text-sm font-medium shadow-lg shadow-blue-500/20 hover:shadow-blue-500/30 transition-shadow">
                        Connect Keplr
                    </button>
                </div>
            </div>

            {/* Protocol Stats Banner */}
            <motion.div
                initial={{ opacity: 0, y: -10 }}
                animate={{ opacity: 1, y: 0 }}
                className="grid grid-cols-4 gap-4"
            >
                {[
                    { label: "Total Obligations", value: "47", icon: "📃" },
                    { label: "Cycles Found", value: "12", icon: "🔄" },
                    { label: "TEE Committee", value: "5 nodes", icon: "🔐" },
                    { label: "Total Cleared", value: "$2.3M", icon: "💰" },
                ].map((stat, i) => (
                    <motion.div
                        key={stat.label}
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: i * 0.1 }}
                        className="hero-card p-4 flex items-center gap-3"
                    >
                        <span className="text-2xl">{stat.icon}</span>
                        <div>
                            <div className="text-xl font-bold text-white">{stat.value}</div>
                            <div className="text-xs text-zinc-400">{stat.label}</div>
                        </div>
                    </motion.div>
                ))}
            </motion.div>

            <div className="grid grid-cols-12 gap-6">
                <motion.div initial={{ opacity: 0, x: -20 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: 0.2 }} className="col-span-12 md:col-span-12">
                    <CycleVisualizer />
                </motion.div>
                <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.4 }} className="col-span-12 md:col-span-12">
                    <SettlementPanel />
                </motion.div>
            </div>
        </main>
    );
}
