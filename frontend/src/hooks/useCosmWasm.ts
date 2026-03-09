"use client";

import { useState, useCallback } from "react";

// CosmWasm contract configuration
export const CONTRACT_CONFIG = {
    // Will be set after deployment
    contractAddress: "",
    chainId: "uni-7", // Juno testnet
    rpcEndpoint: "https://rpc.uni.junonetwork.io",
    denom: "ujunox",
};

interface CosmWasmState {
    connected: boolean;
    address: string | null;
    balance: string | null;
}

/**
 * Hook for Keplr wallet connection and CosmWasm contract interaction.
 * Uses @cosmjs/cosmwasm-stargate under the hood.
 */
export function useCosmWasm() {
    const [state, setState] = useState<CosmWasmState>({
        connected: false,
        address: null,
        balance: null,
    });
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const connect = useCallback(async () => {
        setLoading(true);
        setError(null);

        try {
            // Check for Keplr
            if (typeof window === "undefined" || !(window as any).keplr) {
                throw new Error("Keplr wallet not found. Please install the Keplr extension.");
            }

            const keplr = (window as any).keplr;

            // Suggest chain if needed
            if (keplr.experimentalSuggestChain) {
                try {
                    await keplr.experimentalSuggestChain({
                        chainId: CONTRACT_CONFIG.chainId,
                        chainName: "Juno Testnet",
                        rpc: CONTRACT_CONFIG.rpcEndpoint,
                        rest: "https://api.uni.junonetwork.io",
                        bip44: {
                            coinType: 118,
                        },
                        bech32Config: {
                            bech32PrefixAccAddr: "juno",
                            bech32PrefixAccPub: "junopub",
                            bech32PrefixValAddr: "junovaloper",
                            bech32PrefixValPub: "junovaloperpub",
                            bech32PrefixConsAddr: "junovalcons",
                            bech32PrefixConsPub: "junovalconspub",
                        },
                        currencies: [
                            {
                                coinDenom: "JUNOX",
                                coinMinimalDenom: CONTRACT_CONFIG.denom,
                                coinDecimals: 6,
                                coinGeckoId: "juno-network",
                            },
                        ],
                        feeCurrencies: [
                            {
                                coinDenom: "JUNOX",
                                coinMinimalDenom: CONTRACT_CONFIG.denom,
                                coinDecimals: 6,
                                coinGeckoId: "juno-network",
                                gasPriceStep: {
                                    low: 0.03,
                                    average: 0.04,
                                    high: 0.05,
                                },
                            },
                        ],
                        stakeCurrency: {
                            coinDenom: "JUNOX",
                            coinMinimalDenom: CONTRACT_CONFIG.denom,
                            coinDecimals: 6,
                            coinGeckoId: "juno-network",
                        },
                    });
                } catch {
                    throw new Error("Failed to suggest the chain to Keplr.");
                }
            }

            await keplr.enable(CONTRACT_CONFIG.chainId);

            const offlineSigner = keplr.getOfflineSigner(CONTRACT_CONFIG.chainId);
            const accounts = await offlineSigner.getAccounts();

            if (accounts.length === 0) {
                throw new Error("No accounts found in Keplr");
            }

            const address = accounts[0].address;

            let balanceAmount = "0";
            try {
                // Get balance using CosmJS
                const { SigningCosmWasmClient } = await import("@cosmjs/cosmwasm-stargate");
                const client = await SigningCosmWasmClient.connectWithSigner(
                    CONTRACT_CONFIG.rpcEndpoint,
                    offlineSigner
                );

                const balance = await client.getBalance(address, CONTRACT_CONFIG.denom);
                balanceAmount = balance.amount;
            } catch (rpcError) {
                console.warn("Failed to fetch balance via RPC, but keplr is connected", rpcError);
            }

            setState({
                connected: true,
                address,
                balance: balanceAmount,
            });
        } catch (err: any) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    }, []);

    const disconnect = useCallback(() => {
        setState({
            connected: false,
            address: null,
            balance: null,
        });
    }, []);

    const executeContract = useCallback(
        async (msg: Record<string, unknown>, funds?: { denom: string; amount: string }[]) => {
            if (!state.connected || !state.address) {
                throw new Error("Wallet not connected");
            }

            const keplr = (window as any).keplr;
            const offlineSigner = keplr.getOfflineSigner(CONTRACT_CONFIG.chainId);

            const { SigningCosmWasmClient } = await import("@cosmjs/cosmwasm-stargate");
            const client = await SigningCosmWasmClient.connectWithSigner(
                CONTRACT_CONFIG.rpcEndpoint,
                offlineSigner
            );

            const result = await client.execute(
                state.address,
                CONTRACT_CONFIG.contractAddress,
                msg,
                "auto",
                undefined,
                funds
            );

            return result;
        },
        [state.connected, state.address]
    );

    const queryContract = useCallback(
        async (msg: Record<string, unknown>) => {
            const { CosmWasmClient } = await import("@cosmjs/cosmwasm-stargate");
            const client = await CosmWasmClient.connect(CONTRACT_CONFIG.rpcEndpoint);
            return client.queryContractSmart(CONTRACT_CONFIG.contractAddress, msg);
        },
        []
    );

    return {
        ...state,
        loading,
        error,
        connect,
        disconnect,
        executeContract,
        queryContract,
    };
}
