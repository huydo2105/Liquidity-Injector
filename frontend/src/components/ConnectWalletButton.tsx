"use client";

import React, { useState, useRef, useEffect } from "react";
import { useCosmWasm } from "@/hooks/useCosmWasm";

interface ConnectWalletButtonProps {
    className?: string;
}

export default function ConnectWalletButton({ className }: ConnectWalletButtonProps) {
    const { connect, disconnect, connected, address, balance, loading, error } = useCosmWasm();
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const [copied, setCopied] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);

    const [showError, setShowError] = useState(false);

    // Watch for connection errors
    useEffect(() => {
        if (error) {
            console.error("Keplr Error:", error);
            setShowError(true);
            const timer = setTimeout(() => setShowError(false), 5000);
            return () => clearTimeout(timer);
        }
    }, [error]);

    // Close menu when clicking outside
    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                setIsMenuOpen(false);
            }
        }
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    const shortenAddress = (addr: string) => {
        return `${addr.slice(0, 10)}...${addr.slice(-6)}`;
    };

    const handleCopy = async () => {
        if (address) {
            await navigator.clipboard.writeText(address);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const formatBalance = (bal: string | null) => {
        if (!bal) return "0.00 JUNOX";
        const parsed = parseFloat(bal) / 1_000_000;
        return `${parsed.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} JUNOX`;
    };

    if (connected && address) {
        return (
            <div className="relative" ref={menuRef}>
                <button
                    onClick={() => setIsMenuOpen(!isMenuOpen)}
                    className={`flex items-center gap-2 bg-[#18191b] border border-[#27272a] hover:bg-[#27272a] text-xs px-4 py-2 rounded-full transition-colors ${className}`}
                    title="Wallet Menu"
                >
                    <div className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]"></div>
                    <span className="text-zinc-200 font-medium tracking-wide">{shortenAddress(address)}</span>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={`transition-transform duration-200 ${isMenuOpen ? "rotate-180" : ""}`}>
                        <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                </button>

                {isMenuOpen && (
                    <div className="absolute right-0 mt-2 w-64 bg-[#18191b] border border-[#27272a] rounded-xl shadow-2xl py-2 z-50 animate-in fade-in slide-in-from-top-2 duration-200 overflow-hidden">

                        {/* Row 1: Address & Copy */}
                        <div className="px-4 py-3 flex justify-between items-center border-b border-[#27272a]/50 hover:bg-[#27272a]/30 transition-colors">
                            <span className="text-xs text-zinc-300 font-mono tracking-tight">{shortenAddress(address)}</span>
                            <button
                                onClick={handleCopy}
                                className="text-zinc-500 hover:text-white transition-colors p-1"
                                title="Copy Address"
                            >
                                {copied ? (
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-400"><polyline points="20 6 9 17 4 12"></polyline></svg>
                                ) : (
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                                )}
                            </button>
                        </div>

                        {/* Row 2: Balance */}
                        <div className="px-4 py-3 border-b border-[#27272a]/50 flex justify-between items-center hover:bg-[#27272a]/30 transition-colors">
                            <span className="text-xs text-zinc-500 font-medium">Balance</span>
                            <span className="text-sm text-white font-bold tracking-tight">{formatBalance(balance)}</span>
                        </div>

                        {/* Row 3: Disconnect */}
                        <div className="p-1">
                            <button
                                onClick={() => {
                                    setIsMenuOpen(false);
                                    disconnect();
                                }}
                                className="w-full text-left px-3 py-2 text-xs text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-md transition-colors flex items-center gap-2 font-medium"
                            >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path><polyline points="16 17 21 12 16 7"></polyline><line x1="21" y1="12" x2="9" y2="12"></line></svg>
                                Disconnect Wallet
                            </button>
                        </div>
                    </div>
                )}
            </div>
        );
    }

    return (
        <div className="relative flex flex-col">
            <button
                onClick={connect}
                disabled={loading}
                className={`px-5 py-2 flex items-center gap-2 rounded-full bg-gradient-to-r from-blue-600 to-violet-600 text-white text-xs font-semibold shadow-lg shadow-blue-500/20 hover:shadow-blue-500/30 transition-shadow disabled:opacity-50 disabled:cursor-not-allowed ${className}`}
            >
                {loading ? (
                    <>
                        <svg className="animate-spin -ml-1 mr-1 h-3 w-3 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        Connecting...
                    </>
                ) : "Connect Keplr"}
            </button>

            {showError && error && (
                <div className="absolute top-full right-0 mt-3 w-[260px] bg-red-950/80 border border-red-900/50 rounded-xl p-3 shadow-2xl animate-in fade-in slide-in-from-top-2 z-50">
                    <div className="flex items-start gap-2.5">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="text-red-500 flex-shrink-0 mt-0.5">
                            <circle cx="12" cy="12" r="10"></circle>
                            <line x1="12" y1="8" x2="12" y2="12"></line>
                            <line x1="12" y1="16" x2="12.01" y2="16"></line>
                        </svg>
                        <div className="flex flex-col text-left">
                            <span className="text-xs font-semibold text-red-400 leading-tight mb-1">Connection Failed</span>
                            <span className="text-[11px] text-red-300/80 leading-snug">{error}</span>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
