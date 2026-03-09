"use client";

import React from "react";
import ConnectWalletButton from "@/components/ConnectWalletButton";

export default function Header() {
    return (
        <header className="flex justify-between items-center px-6 py-4 border-b border-[#1f1f22] bg-[#0a0a0a]/50 backdrop-blur-md sticky top-0 z-40 w-full">
            <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-blue-600 to-violet-600 flex items-center justify-center shadow-lg shadow-blue-500/20">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z"></path>
                    </svg>
                </div>
                <span className="font-bold text-[#f8fafc] tracking-tight">Liquidity Injector</span>
            </div>

            <div className="flex items-center gap-4">
                <ConnectWalletButton />
            </div>
        </header>
    );
}
