"use client";

import React from "react";
import { motion, AnimatePresence } from "framer-motion";

interface Validator {
    id: number;
    name: string;
    pubkey: string;
    hasSigned: boolean;
}

interface QuorumTrackerProps {
    validators?: Validator[];
    signaturesRequired?: number;
    proposalHash?: string;
}

const DEMO_VALIDATORS: Validator[] = [
    { id: 0, name: "TEE-Alpha", pubkey: "0x1a2b...3c4d", hasSigned: true },
    { id: 1, name: "TEE-Beta", pubkey: "0x5e6f...7a8b", hasSigned: true },
    { id: 2, name: "TEE-Gamma", pubkey: "0x9c0d...1e2f", hasSigned: true },
    { id: 3, name: "TEE-Delta", pubkey: "0x3a4b...5c6d", hasSigned: false },
    { id: 4, name: "TEE-Epsilon", pubkey: "0x7e8f...9a0b", hasSigned: false },
];

export default function QuorumTracker({
    validators = DEMO_VALIDATORS,
    signaturesRequired = 3,
    proposalHash = "0x2a2a...4242",
}: QuorumTrackerProps) {
    const signed = validators.filter((v) => v.hasSigned).length;
    const total = validators.length;
    const progress = signed / signaturesRequired;
    const quorumReached = signed >= signaturesRequired;

    // SVG progress ring
    const size = 160;
    const strokeWidth = 6;
    const radius = (size - strokeWidth) / 2;
    const circumference = 2 * Math.PI * radius;
    const offset = circumference - (Math.min(progress, 1) * circumference);

    return (
        <div className="glass-card p-6 h-full">
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h2 className="text-lg font-semibold text-white">Quorum Consensus</h2>
                    <p className="text-sm text-slate-400 mt-1">
                        BFT 2f+1 threshold • Ed25519 attestation signatures
                    </p>
                </div>
                <div
                    className={`px-3 py-1 rounded-full text-xs font-medium ${quorumReached
                            ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
                            : "bg-amber-500/20 text-amber-400 border border-amber-500/30"
                        }`}
                >
                    {quorumReached ? "✓ Quorum Reached" : "⏳ Collecting Votes"}
                </div>
            </div>

            <div className="flex items-center gap-8">
                {/* Progress Ring */}
                <div className="relative flex-shrink-0">
                    <svg width={size} height={size} className={quorumReached ? "pulse-ring" : ""}>
                        {/* Background ring */}
                        <circle
                            cx={size / 2}
                            cy={size / 2}
                            r={radius}
                            fill="none"
                            stroke="rgba(255,255,255,0.05)"
                            strokeWidth={strokeWidth}
                        />
                        {/* Progress ring */}
                        <motion.circle
                            cx={size / 2}
                            cy={size / 2}
                            r={radius}
                            fill="none"
                            stroke={quorumReached ? "#10b981" : "#3b82f6"}
                            strokeWidth={strokeWidth}
                            strokeLinecap="round"
                            strokeDasharray={circumference}
                            initial={{ strokeDashoffset: circumference }}
                            animate={{ strokeDashoffset: offset }}
                            transition={{ duration: 1.5, ease: "easeInOut" }}
                            style={{
                                transform: "rotate(-90deg)",
                                transformOrigin: "center",
                                filter: quorumReached
                                    ? "drop-shadow(0 0 8px rgba(16, 185, 129, 0.5))"
                                    : "drop-shadow(0 0 8px rgba(59, 130, 246, 0.3))",
                            }}
                        />
                    </svg>
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                        <motion.span
                            className="text-3xl font-bold text-white"
                            key={signed}
                            initial={{ scale: 1.5, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            transition={{ type: "spring", stiffness: 200 }}
                        >
                            {signed}/{signaturesRequired}
                        </motion.span>
                        <span className="text-xs text-slate-400 mt-1">Validators</span>
                    </div>
                </div>

                {/* Validator list */}
                <div className="flex-1 space-y-2">
                    <AnimatePresence>
                        {validators.map((v, i) => (
                            <motion.div
                                key={v.id}
                                initial={{ opacity: 0, x: -20 }}
                                animate={{ opacity: 1, x: 0 }}
                                transition={{ delay: i * 0.1 }}
                                className={`flex items-center justify-between p-3 rounded-lg transition-all ${v.hasSigned
                                        ? "bg-emerald-500/10 border border-emerald-500/20"
                                        : "bg-slate-800/30 border border-slate-700/30"
                                    }`}
                            >
                                <div className="flex items-center gap-3">
                                    <div
                                        className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-mono ${v.hasSigned
                                                ? "bg-emerald-500/20 text-emerald-400"
                                                : "bg-slate-700/50 text-slate-500"
                                            }`}
                                    >
                                        {v.hasSigned ? "✓" : (i + 1)}
                                    </div>
                                    <div>
                                        <div className="text-sm font-medium text-white">{v.name}</div>
                                        <div
                                            className="text-xs text-slate-500"
                                            style={{ fontFamily: "'JetBrains Mono', monospace" }}
                                        >
                                            {v.pubkey}
                                        </div>
                                    </div>
                                </div>
                                <motion.div
                                    initial={false}
                                    animate={{
                                        scale: v.hasSigned ? [1, 1.2, 1] : 1,
                                    }}
                                    transition={{ duration: 0.3 }}
                                    className={`text-xs font-medium px-2 py-1 rounded ${v.hasSigned
                                            ? "text-emerald-400"
                                            : "text-slate-500"
                                        }`}
                                >
                                    {v.hasSigned ? "Signed" : "Pending"}
                                </motion.div>
                            </motion.div>
                        ))}
                    </AnimatePresence>
                </div>
            </div>

            {/* Proposal Hash */}
            {quorumReached && (
                <motion.div
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="mt-4 p-3 rounded-lg bg-emerald-500/5 border border-emerald-500/20"
                >
                    <div className="flex items-center justify-between">
                        <span className="text-xs text-slate-400">Quorum Certificate Hash</span>
                        <span
                            className="text-xs text-emerald-400"
                            style={{ fontFamily: "'JetBrains Mono', monospace" }}
                        >
                            {proposalHash}
                        </span>
                    </div>
                </motion.div>
            )}
        </div>
    );
}
