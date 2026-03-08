"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";

interface ObligationSubmitProps {
    onSubmit?: (obligation: {
        debtor: string;
        creditor: string;
        amount: number;
    }) => void;
}

export default function ObligationSubmit({ onSubmit }: ObligationSubmitProps) {
    const [debtor, setDebtor] = useState("");
    const [creditor, setCreditor] = useState("");
    const [amount, setAmount] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [submitted, setSubmitted] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!debtor || !creditor || !amount) return;

        setSubmitting(true);
        // Simulate encryption + on-chain submission
        await new Promise((r) => setTimeout(r, 1500));
        setSubmitting(false);
        setSubmitted(true);

        onSubmit?.({
            debtor,
            creditor,
            amount: parseFloat(amount),
        });

        // Reset after animation
        setTimeout(() => {
            setSubmitted(false);
            setDebtor("");
            setCreditor("");
            setAmount("");
        }, 2000);
    };

    return (
        <div className="glass-card p-6">
            <div className="mb-4">
                <h2 className="text-lg font-semibold text-white">Submit Obligation</h2>
                <p className="text-sm text-slate-400 mt-1">
                    Encrypted with TEE&apos;s xPub before posting to chain
                </p>
            </div>

            <form onSubmit={handleSubmit} className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <label className="block text-xs text-slate-400 mb-1.5">Debtor</label>
                        <input
                            type="text"
                            value={debtor}
                            onChange={(e) => setDebtor(e.target.value)}
                            placeholder="juno1abc..."
                            className="w-full px-3 py-2.5 rounded-lg bg-slate-800/50 border border-slate-700/50 text-white text-sm placeholder-slate-600 focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 transition-all"
                            style={{ fontFamily: "'JetBrains Mono', monospace" }}
                        />
                    </div>
                    <div>
                        <label className="block text-xs text-slate-400 mb-1.5">Creditor</label>
                        <input
                            type="text"
                            value={creditor}
                            onChange={(e) => setCreditor(e.target.value)}
                            placeholder="juno1xyz..."
                            className="w-full px-3 py-2.5 rounded-lg bg-slate-800/50 border border-slate-700/50 text-white text-sm placeholder-slate-600 focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 transition-all"
                            style={{ fontFamily: "'JetBrains Mono', monospace" }}
                        />
                    </div>
                </div>
                <div>
                    <label className="block text-xs text-slate-400 mb-1.5">Amount (JUNO)</label>
                    <input
                        type="number"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                        placeholder="0.00"
                        className="w-full px-3 py-2.5 rounded-lg bg-slate-800/50 border border-slate-700/50 text-white text-sm placeholder-slate-600 focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 transition-all"
                        style={{ fontFamily: "'JetBrains Mono', monospace" }}
                    />
                </div>

                {/* Encryption indicator */}
                <div className="flex items-center gap-2 p-2.5 rounded-lg bg-cyan-500/5 border border-cyan-500/10">
                    <svg
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="#06b6d4"
                        strokeWidth="2"
                    >
                        <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                        <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                    </svg>
                    <span className="text-xs text-cyan-400">
                        Data encrypted with AES-256-GCM to TEE&apos;s public key
                    </span>
                </div>

                <motion.button
                    type="submit"
                    disabled={submitting || !debtor || !creditor || !amount}
                    whileHover={{ scale: 1.01 }}
                    whileTap={{ scale: 0.99 }}
                    className={`
            w-full py-3 rounded-xl font-medium text-sm transition-all
            ${submitted
                            ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
                            : submitting
                                ? "bg-blue-600/50 text-blue-300"
                                : "bg-blue-600 text-white hover:bg-blue-500"
                        }
            disabled:opacity-50 disabled:cursor-not-allowed
          `}
                >
                    {submitted
                        ? "✓ Obligation Encrypted & Submitted"
                        : submitting
                            ? "Encrypting & Broadcasting..."
                            : "Encrypt & Submit to Chain"}
                </motion.button>
            </form>
        </div>
    );
}
