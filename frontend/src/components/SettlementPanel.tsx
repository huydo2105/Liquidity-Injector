"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";

interface SettlementPanelProps {
    totalDebt?: number;
    totalCleared?: number;
    injectionUsed?: number;
    multiplier?: number;
    isQuorumReached?: boolean;
    onSettle?: () => void;
}

export default function SettlementPanel({
    totalDebt = 138000,
    totalCleared = 100000,
    injectionUsed = 5000,
    multiplier = 20,
    isQuorumReached = true,
    onSettle,
}: SettlementPanelProps) {
    const [settling, setSettling] = useState(false);
    const [settled, setSettled] = useState(false);

    const handleSettle = async () => {
        setSettling(true);
        // Simulate settlement transaction
        await new Promise((r) => setTimeout(r, 2500));
        setSettling(false);
        setSettled(true);
        onSettle?.();
    };

    const stats = [
        {
            label: "Total Debt in Network",
            value: `$${(totalDebt / 1000).toFixed(0)}k`,
            color: "text-slate-300",
            icon: "📊",
        },
        {
            label: "Debt Cleared by MTCS",
            value: `$${(totalCleared / 1000).toFixed(0)}k`,
            color: "text-emerald-400",
            icon: "✨",
        },
        {
            label: "Injection Liquidity Used",
            value: `$${(injectionUsed / 1000).toFixed(1)}k`,
            color: "text-cyan-400",
            icon: "💎",
        },
        {
            label: "Multiplier Effect",
            value: `${multiplier}x`,
            color: "text-violet-400",
            icon: "🚀",
        },
    ];

    return (
        <div className="glass-card p-6">
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h2 className="text-lg font-semibold text-white">Settlement Execution</h2>
                    <p className="text-sm text-slate-400 mt-1">
                        Atomic on-chain clearing via CosmWasm
                    </p>
                </div>
                {settled && (
                    <motion.div
                        initial={{ opacity: 0, scale: 0.8 }}
                        animate={{ opacity: 1, scale: 1 }}
                        className="px-3 py-1 rounded-full text-xs font-medium bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
                    >
                        ✓ Settlement Complete
                    </motion.div>
                )}
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-2 gap-4 mb-6">
                {stats.map((stat, i) => (
                    <motion.div
                        key={stat.label}
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: i * 0.1 }}
                        className="p-4 rounded-xl bg-slate-800/30 border border-slate-700/30"
                    >
                        <div className="flex items-center gap-2 mb-2">
                            <span className="text-lg">{stat.icon}</span>
                            <span className="text-xs text-slate-400">{stat.label}</span>
                        </div>
                        <div className={`text-2xl font-bold ${stat.color}`}>{stat.value}</div>
                    </motion.div>
                ))}
            </div>

            {/* Multiplier Visualization */}
            <div className="mb-6 p-4 rounded-xl bg-gradient-to-r from-violet-500/10 via-blue-500/10 to-cyan-500/10 border border-violet-500/20">
                <div className="flex items-center justify-between mb-3">
                    <span className="text-sm text-slate-300">Liquidity Leverage</span>
                    <span
                        className="text-sm text-violet-400"
                        style={{ fontFamily: "'JetBrains Mono', monospace" }}
                    >
                        ${(injectionUsed / 1000).toFixed(1)}k → ${(totalCleared / 1000).toFixed(0)}k cleared
                    </span>
                </div>
                <div className="relative h-3 rounded-full bg-slate-800/50 overflow-hidden">
                    <motion.div
                        className="absolute inset-y-0 left-0 rounded-full"
                        style={{
                            background:
                                "linear-gradient(90deg, #8b5cf6, #3b82f6, #06b6d4)",
                        }}
                        initial={{ width: 0 }}
                        animate={{ width: `${Math.min((totalCleared / totalDebt) * 100, 100)}%` }}
                        transition={{ duration: 1.5, ease: "easeOut" }}
                    />
                    {/* Injection marker */}
                    <motion.div
                        className="absolute top-1/2 -translate-y-1/2 w-2 h-5 rounded bg-amber-400"
                        style={{ left: `${(injectionUsed / totalDebt) * 100}%` }}
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        transition={{ delay: 0.8 }}
                    />
                </div>
                <div className="flex justify-between mt-2 text-xs text-slate-500">
                    <span>Injection: {((injectionUsed / totalDebt) * 100).toFixed(1)}%</span>
                    <span>Cleared: {((totalCleared / totalDebt) * 100).toFixed(1)}%</span>
                </div>
            </div>

            {/* Settle Button */}
            <motion.button
                onClick={handleSettle}
                disabled={!isQuorumReached || settling || settled}
                whileHover={isQuorumReached && !settling && !settled ? { scale: 1.02 } : {}}
                whileTap={isQuorumReached && !settling && !settled ? { scale: 0.98 } : {}}
                className={`
          w-full py-4 rounded-xl font-semibold text-base transition-all
          ${settled
                        ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 cursor-default"
                        : isQuorumReached
                            ? "bg-gradient-to-r from-blue-600 to-violet-600 text-white shadow-lg shadow-blue-500/25 hover:shadow-blue-500/40 cursor-pointer"
                            : "bg-slate-800/50 text-slate-500 border border-slate-700/30 cursor-not-allowed"
                    }
        `}
            >
                {settled ? (
                    <span className="flex items-center justify-center gap-2">
                        ✓ Settlement Executed
                    </span>
                ) : settling ? (
                    <span className="flex items-center justify-center gap-2">
                        <svg
                            className="animate-spin h-5 w-5"
                            fill="none"
                            viewBox="0 0 24 24"
                        >
                            <circle
                                className="opacity-25"
                                cx="12"
                                cy="12"
                                r="10"
                                stroke="currentColor"
                                strokeWidth="4"
                            />
                            <path
                                className="opacity-75"
                                fill="currentColor"
                                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                            />
                        </svg>
                        Broadcasting to Juno Testnet...
                    </span>
                ) : isQuorumReached ? (
                    "Execute Settlement on Chain"
                ) : (
                    "Waiting for Quorum..."
                )}
            </motion.button>
        </div>
    );
}
