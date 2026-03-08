"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";
import CycleVisualizer from "./CycleVisualizer";
import QuorumTracker from "./QuorumTracker";
import SettlementPanel from "./SettlementPanel";
import ObligationSubmit from "./ObligationSubmit";

export default function Dashboard() {
    const [appLaunched, setAppLaunched] = useState(false);

    if (appLaunched) {
        return (
            <div className="min-h-screen bg-[#111111]">
                {/* Header for launched app */}
                <nav className="sticky top-0 z-50 backdrop-blur-xl border-b border-[#27272a] bg-[#111111]/80">
                    <div className="max-w-[1440px] mx-auto px-6 py-4 flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className="w-8 h-8 rounded bg-blue-600 flex items-center justify-center">
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                            </div>
                            <div>
                                <h1 className="text-lg font-bold text-white tracking-tight">
                                    Liquidity Injector
                                </h1>
                            </div>
                        </div>

                        <div className="flex items-center gap-4">
                            <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-[#18191b] border border-[#27272a]">
                                <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                                <span className="text-xs text-zinc-300">Juno Testnet</span>
                            </div>
                            <button
                                onClick={() => setAppLaunched(false)}
                                className="px-4 py-2 rounded-xl bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors"
                            >
                                Leave App
                            </button>
                        </div>
                    </div>
                </nav>

                {/* Main Content */}
                <main className="max-w-[1440px] mx-auto px-6 py-8 space-y-6">
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
                        <motion.div initial={{ opacity: 0, x: -20 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: 0.2 }} className="col-span-12 md:col-span-7">
                            <CycleVisualizer />
                        </motion.div>
                        <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: 0.3 }} className="col-span-12 md:col-span-5">
                            <QuorumTracker />
                        </motion.div>
                        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.4 }} className="col-span-12 md:col-span-7">
                            <SettlementPanel />
                        </motion.div>
                        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.5 }} className="col-span-12 md:col-span-5">
                            <ObligationSubmit />
                        </motion.div>
                    </div>
                </main>
            </div>
        );
    }

    // Landing View (Matches Image)
    return (
        <div className="flex-1 flex flex-col min-h-screen bg-[#111111] text-[#f8fafc] font-sans">
            <header className="flex justify-between items-center p-6 text-sm">
                <div className="flex items-center gap-3">
                    <div className="w-5 h-5 rounded bg-blue-600 flex items-center justify-center">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                    </div>
                    <span className="font-semibold tracking-wide">Liquidity Injector</span>
                </div>
                <div className="flex gap-4">
                    <button className="flex items-center gap-2 bg-[#18191b] border border-[#27272a] hover:bg-[#27272a] text-xs px-3 py-1.5 rounded-full transition-colors">
                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                        <span className="text-zinc-300">0x6F8c...E88B</span>
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                    </button>
                </div>
            </header>

            <main className="flex-1 flex flex-col items-center justify-center -mt-10 px-6">
                <div className="max-w-4xl text-center">
                    <h1 className="text-5xl md:text-6xl font-bold tracking-tight mb-6 mt-12 text-[#f8fafc]">
                        Clearing without <br />
                        <span className="text-blue-500">Single Points of Failure</span>
                    </h1>
                    <p className="text-zinc-400 text-lg md:text-[19px] max-w-2xl mx-auto mb-10 leading-relaxed font-normal">
                        Solve liquidity gridlock inside a TEE.<br />
                        Upload obligations securely. Find cycles blindly. Settle atomically.
                    </p>

                    <div className="flex justify-center items-center gap-4 mb-24 text-sm font-medium">
                        <button
                            onClick={() => setAppLaunched(true)}
                            className="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2.5 rounded-full transition-colors"
                        >
                            Launch App
                        </button>
                        <button className="bg-transparent border border-zinc-700 hover:border-zinc-500 text-zinc-300 px-6 py-2.5 rounded-full transition-colors">
                            View Enclave Status
                        </button>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 text-left">
                        <div className="hero-card p-6 rounded-2xl flex flex-col">
                            <div className="w-10 h-10 rounded-lg bg-[#18191b] border border-[#27272a] flex items-center justify-center mb-4 text-blue-400 opacity-90">
                                <LockIcon />
                            </div>
                            <h3 className="font-semibold text-white mb-2 tracking-tight">Privacy First (TEE)</h3>
                            <p className="text-zinc-400/90 text-[13px] leading-[1.6]">
                                Obligations are encrypted with AES-GCM. Raw data never leaves the enclave.
                            </p>
                        </div>
                        <div className="hero-card p-6 rounded-2xl flex flex-col">
                            <div className="w-10 h-10 rounded-lg bg-[#18191b] border border-[#27272a] flex items-center justify-center mb-4 text-amber-400 opacity-90">
                                <UsersIcon />
                            </div>
                            <h3 className="font-semibold text-white mb-2 tracking-tight">Optimal Settlement</h3>
                            <p className="text-zinc-400/90 text-[13px] leading-[1.6]">
                                Johnson's algorithm discovers cycles and maximizes injection multipliers.
                            </p>
                        </div>
                        <div className="hero-card p-6 rounded-2xl flex flex-col">
                            <div className="w-10 h-10 rounded-lg bg-[#18191b] border border-[#27272a] flex items-center justify-center mb-4 text-cyan-400 opacity-90">
                                <LightningIcon />
                            </div>
                            <h3 className="font-semibold text-white mb-2 tracking-tight">Atomic Finality</h3>
                            <p className="text-zinc-400/90 text-[13px] leading-[1.6]">
                                CosmWasm contract settles dependencies atomically with 2f+1 BFT quorum.
                            </p>
                        </div>
                    </div>
                </div>

                <div className="mt-20 text-[11px] text-zinc-600 font-medium tracking-wide">
                    Based on <a className="text-blue-500 hover:text-blue-600" href="https://arxiv.org/pdf/2507.22309" target="_blank" rel="noopener noreferrer">Cycles Protocol (arXiv:2507.22309)</a>
                </div>
            </main>
        </div>
    );
}

// Icons
function LockIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="w-5 h-5">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
        </svg>
    );
}

function UsersIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="w-5 h-5">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
        </svg>
    );
}

function LightningIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="w-5 h-5">
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
        </svg>
    );
}
