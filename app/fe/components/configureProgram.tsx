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
        if (!connected || !publicKey) return;
        
        setLoading(true);
        setError(null);
        try {
            // 1. First define base amounts WITHOUT decimals
            const BASE_SUPPLY = new BN('1000000000');  // 1B base tokens
            
            // 2. Calculate 80% for virtual reserve BEFORE adding decimals
            const VIRTUAL_RESERVE = BASE_SUPPLY.muln(8).divn(10);  // 800M base tokens
            
            // 3. Then add decimals to both
            const DECIMALS = new BN(6);
            const TOTAL_SUPPLY = BASE_SUPPLY.mul(new BN(10).pow(DECIMALS));  // 1B with decimals
            const VIRTUAL_RESERVE_WITH_DECIMALS = VIRTUAL_RESERVE.mul(new BN(10).pow(DECIMALS));  // 800M with decimals
            
            // SOL amounts
            const INITIAL_MARKET_CAP = new BN('2200').mul(new BN(10).pow(new BN(9)));  // 2.2k SOL
            const TARGET_MARKET_CAP = new BN('69000').mul(new BN(10).pow(new BN(9)));  // 69k SOL
            
            const config = {
                authority: publicKey,
                feeRecipient: publicKey,
                curveLimit: TARGET_MARKET_CAP,
                initialVirtualTokenReserve: VIRTUAL_RESERVE_WITH_DECIMALS,  // 800M WITH decimals
                initialVirtualSolReserve: INITIAL_MARKET_CAP,
                initialRealTokenReserve: new BN(0),
                totalTokenSupply: TOTAL_SUPPLY,
                buyFeePercentage: 0.5,
                sellFeePercentage: 0.5,
                migrationFeePercentage: 0.5,
            };
            
            console.log("Virtual Reserve:", VIRTUAL_RESERVE_WITH_DECIMALS.toString());
            const tx = await configure(config);
            setSuccess(true);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'An unknown error occurred');
        }
        setLoading(false);
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
