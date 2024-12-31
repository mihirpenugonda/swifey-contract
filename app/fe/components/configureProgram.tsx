"use client"
import { FC, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { useSwifey } from '../hooks/useSwifey';
import { BN } from '@coral-xyz/anchor';
import { motion } from "framer-motion";

export const ConfigureProgram: FC = () => {
    const { publicKey, connected } = useWallet();
    const { configure } = useSwifey();
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState(false);

    const handleConfigure = async () => {
        if (!connected) return;
        if (!publicKey) return;
        
        setLoading(true);
        setError(null);
        try {
            const TOTAL_SUPPLY = new BN(1_000_000_000); // 1 billion tokens
            const INITIAL_MARKET_CAP = new BN(2_200); // 2.2k initial market cap
            const TARGET_MARKET_CAP = new BN(69_000); // 69k target market cap
            const BONDING_RATIO = 0.8; // 80% for bonding curve, 20% for Raydium LP
            
            const config = {
                authority: publicKey,
                feeRecipient: publicKey,
                curveLimit: TARGET_MARKET_CAP, // 69K SOL for bonding curve
                initialVirtualTokenReserve: TOTAL_SUPPLY.muln(BONDING_RATIO), // 800M tokens
                initialVirtualSolReserve: new BN(INITIAL_MARKET_CAP), // 2.2k SOL
                initialRealTokenReserve: new BN(0),
                totalTokenSupply: TOTAL_SUPPLY, // 1B total supply
                buyFeePercentage: 1, // 1% fee
                sellFeePercentage: 1, // 1% fee
                migrationFeePercentage: 1, // 1% fee
            };
            const tx = await configure(config);
            console.log('Program configured:', tx);
            setSuccess(true);
        } catch (err) {
            console.error('Configuration failed:', err);
            setError(err instanceof Error ? err.message : 'An unknown error occurred');
        } finally {
            setLoading(false);
        }
    };

    return (
        <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="w-full max-w-md p-6 bg-black/20 backdrop-blur-sm rounded-lg border border-white/50"
        >
            <div className="mb-6">
                <h2 className="text-xl font-medium text-white">Configure Program</h2>
                <p className="text-sm text-white/60">Set up your program parameters</p>
            </div>

            {error && (
                <div className="text-sm text-red-400 mb-4">
                    {error}
                </div>
            )}
            {success && (
                <div className="text-sm text-green-400 mb-4">
                    Program configured successfully!
                </div>
            )}

            <button
                onClick={handleConfigure}
                disabled={!connected || loading}
                className="w-full py-2 bg-white/10 hover:bg-white/15 
                         rounded-lg text-sm text-white font-medium
                         disabled:opacity-50 disabled:cursor-not-allowed
                         transition-colors"
            >
                {loading ? (
                    <div className="flex items-center justify-center gap-2">
                        <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                        </svg>
                        <span>Configuring...</span>
                    </div>
                ) : (
                    "Configure Program"
                )}
            </button>

            {!connected && (
                <p className="text-center text-xs text-white/60 mt-4">
                    Connect wallet to configure
                </p>
            )}
        </motion.div>
    );
};
