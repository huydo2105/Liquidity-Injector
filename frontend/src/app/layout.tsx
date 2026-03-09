import type { Metadata } from "next";
import "./globals.css";
import Sidebar from "@/components/Sidebar";
import Header from "@/components/Header";

export const metadata: Metadata = {
  title: "Cycles Liquidity Injector",
  description: "Privacy-Preserving Debt Clearing",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <head>
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&display=swap"
          rel="stylesheet"
        />
      </head>
      <body className="antialiased bg-[#111111] text-[#f8fafc] font-sans flex min-h-screen">
        <Sidebar />
        <div className="flex-1 flex flex-col min-h-screen w-full relative">
          <Header />
          {children}
        </div>
      </body>
    </html>
  );
}
