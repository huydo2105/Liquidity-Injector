"use client";

import React from "react";
import { motion } from "framer-motion";

export default function AttestationPage() {
    return (
        <main className="max-w-4xl mx-auto w-full px-6 py-8 space-y-6 flex-1">
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h1 className="text-2xl font-bold text-white tracking-tight">TEE Attestation</h1>
                    <p className="text-sm text-zinc-400 mt-1">Verify SGX DCAP Quotes & Enclave Measurements.</p>
                </div>
                <button className="px-4 py-2 rounded-xl bg-gradient-to-r from-blue-600 to-violet-600 text-white text-sm font-medium shadow-lg shadow-blue-500/20 hover:shadow-blue-500/30 transition-shadow">
                    Connect Keplr
                </button>
            </div>

            <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="hero-card p-6">
                <div className="flex items-center gap-3 mb-6">
                    <div className="w-10 h-10 rounded-lg bg-[#18191b] border border-[#27272a] flex items-center justify-center text-emerald-400">
                        <LockIcon />
                    </div>
                    <div>
                        <h2 className="text-lg font-semibold text-white">Enclave Status: Secure</h2>
                        <div className="text-xs text-emerald-500 mt-0.5">MRENCLAVE matches known build</div>
                    </div>
                </div>

                <div className="space-y-4">
                    <div className="p-4 bg-[#111111] border border-[#27272a] rounded-xl font-mono text-xs text-zinc-400 break-all">
                        <span className="text-zinc-500 block mb-1">MRENCLAVE Hash</span>
                        8a3b5c7d...e9f01a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f
                    </div>
                    <div className="flex items-center justify-between text-sm">
                        <span className="text-zinc-400">Intel SGX Quote Status</span>
                        <span className="text-emerald-400 font-medium bg-emerald-400/10 px-2 py-1 rounded">VERIFIED</span>
                    </div>
                    <div className="flex items-center justify-between text-sm">
                        <span className="text-zinc-400">TCB Level (Trusted Computing Base)</span>
                        <span className="text-emerald-400 font-medium bg-emerald-400/10 px-2 py-1 rounded">UP TO DATE</span>
                    </div>
                </div>
            </motion.div>
        </main>
    );
}

function LockIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="w-5 h-5">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
        </svg>
    );
}
