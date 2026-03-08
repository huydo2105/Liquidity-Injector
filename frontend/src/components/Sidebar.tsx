"use client";

import React from "react";
import Link from "next/link";

export default function Sidebar() {
    const menuItems = [
        { name: "Dashboard", icon: <DashboardIcon fill="#3b82f6" />, href: "/dashboard" },
        { name: "Obligations", icon: <FolderIcon fill="#eab308" />, href: "/obligations" },
        { name: "Consensus", icon: <ChatIcon fill="#6366f1" />, href: "/consensus" },
        { name: "Attestation", icon: <LockIcon fill="#10b981" />, href: "/attestation" },
    ];

    return (
        <aside className="w-[200px] flex-shrink-0 flex flex-col justify-between h-screen sticky top-0 bg-[#0a0a0a] border-r border-[#1f1f22] p-4 text-sm text-[#a1a1aa]">
            <div className="space-y-6">
                <div className="mt-8 mb-4">
                    <ul className="space-y-3">
                        {menuItems.map((item, index) => (
                            <li key={index}>
                                <Link
                                    href={item.href}
                                    className="flex items-center gap-3 px-2 py-1.5 rounded-md hover:bg-[#18181b] hover:text-[#f4f4f5] transition-colors"
                                >
                                    <span className="w-4 h-4 flex items-center justify-center">
                                        {item.icon}
                                    </span>
                                    <span>{item.name}</span>
                                </Link>
                            </li>
                        ))}
                    </ul>
                </div>
            </div>

            <div className="mb-4 flex items-center gap-2 px-2 text-xs">
                <div className="w-6 h-6 rounded-full bg-zinc-800 flex items-center justify-center text-white font-medium border border-zinc-700">
                    N
                </div>
                <div className="flex flex-col">
                    <span className="text-zinc-400">acting...</span>
                    <span className="text-[10px] text-zinc-600">...Documents</span>
                </div>
            </div>
        </aside>
    );
}

// Icons matching the vibe
function DashboardIcon({ fill }: { fill: string }) {
    return (
        <svg viewBox="0 0 24 24" fill={fill} className="w-full h-full">
            <path d="M12 2L2 12h3v8h6v-6h2v6h6v-8h3L12 2z" />
        </svg>
    );
}

function FolderIcon({ fill }: { fill: string }) {
    return (
        <svg viewBox="0 0 24 24" fill={fill} className="w-full h-full">
            <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" />
        </svg>
    );
}

function ChatIcon({ fill }: { fill: string }) {
    return (
        <svg viewBox="0 0 24 24" fill={fill} className="w-full h-full">
            <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z" />
        </svg>
    );
}

function LockIcon({ fill }: { fill: string }) {
    return (
        <svg viewBox="0 0 24 24" fill={fill} className="w-full h-full">
            <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z" />
        </svg>
    );
}
