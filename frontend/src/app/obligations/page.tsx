"use client";

import React from "react";
import ObligationSubmit from "@/components/ObligationSubmit";
import { motion } from "framer-motion";

export default function ObligationsPage() {
    return (
        <main className="max-w-4xl mx-auto w-full px-6 py-8 space-y-6 flex-1">
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h1 className="text-2xl font-bold text-white tracking-tight">Obligation Vault</h1>
                    <p className="text-sm text-zinc-400 mt-1">Submit encrypted obligations securely to the Enclave.</p>
                </div>
                <button className="px-4 py-2 rounded-xl bg-gradient-to-r from-blue-600 to-violet-600 text-white text-sm font-medium shadow-lg shadow-blue-500/20 hover:shadow-blue-500/30 transition-shadow">
                    Connect Keplr
                </button>
            </div>

            <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}>
                <ObligationSubmit />
            </motion.div>
        </main>
    );
}
