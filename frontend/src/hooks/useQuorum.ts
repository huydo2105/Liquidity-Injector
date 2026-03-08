"use client";

import { useState, useEffect, useCallback } from "react";

interface ValidatorState {
    pubkey: string;
    hasSigned: boolean;
}

interface QuorumStatus {
    proposalHash: string;
    signaturesReceived: number;
    signaturesRequired: number;
    quorumReached: boolean;
    validators: ValidatorState[];
}

/**
 * Hook to poll the Collector's REST/gRPC-web endpoint for quorum progress.
 * In production, this connects to the real Collector service.
 * For demo, it simulates progressive signing.
 */
export function useQuorum(
    collectorUrl?: string,
    proposalHash?: string,
    pollingIntervalMs = 3000
) {
    const [status, setStatus] = useState<QuorumStatus>({
        proposalHash: proposalHash || "",
        signaturesReceived: 0,
        signaturesRequired: 3,
        quorumReached: false,
        validators: [
            { pubkey: "0x1a2b...3c4d", hasSigned: false },
            { pubkey: "0x5e6f...7a8b", hasSigned: false },
            { pubkey: "0x9c0d...1e2f", hasSigned: false },
            { pubkey: "0x3a4b...5c6d", hasSigned: false },
            { pubkey: "0x7e8f...9a0b", hasSigned: false },
        ],
    });
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchStatus = useCallback(async () => {
        if (!collectorUrl || !proposalHash) return;

        setLoading(true);
        try {
            const response = await fetch(
                `${collectorUrl}/quorum-status?proposal_hash=${proposalHash}`
            );
            if (!response.ok) throw new Error("Failed to fetch quorum status");
            const data = await response.json();
            setStatus(data);
        } catch (err: any) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    }, [collectorUrl, proposalHash]);

    useEffect(() => {
        if (!collectorUrl || !proposalHash) return;

        fetchStatus();
        const interval = setInterval(fetchStatus, pollingIntervalMs);
        return () => clearInterval(interval);
    }, [collectorUrl, proposalHash, pollingIntervalMs, fetchStatus]);

    return { status, loading, error, refetch: fetchStatus };
}
