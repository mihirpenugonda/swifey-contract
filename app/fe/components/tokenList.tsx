'use client'
import { useEffect, useState } from 'react';
import { LaunchedToken, useSwifey } from '../hooks/useSwifey';
import { useWallet } from '@solana/wallet-adapter-react';
import { motion } from "framer-motion";
import Link from 'next/link';

export function TokenList() {
    const { getLaunchedTokens, program } = useSwifey();
    const { connected } = useWallet();
    const [tokens, setTokens] = useState<LaunchedToken[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        async function fetchTokens() {
            if (!program || !connected) {
                setLoading(false);
                return;
            }

            try {
                const launchedTokens = await getLaunchedTokens();
                setTokens(launchedTokens);
            } catch (e) {
                console.error('Error fetching tokens:', e);
            } finally {
                setLoading(false);
            }
        }

        setLoading(true); // Set loading to true when effect runs
        fetchTokens();
    }, [getLaunchedTokens, program, connected]);

    if (!connected) return (
        <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="w-full max-w-4xl p-6 bg-black/20 backdrop-blur-sm rounded-lg border border-white/50"
        >
            <p className="text-center text-white/60">Please connect your wallet</p>
        </motion.div>
    );

    return (
        <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="w-full max-w-4xl p-6 bg-black/20 backdrop-blur-sm rounded-lg border border-white/50"
        >
            <div className="mb-6">
                <h2 className="text-xl font-medium text-white">Launched Tokens</h2>
                <p className="text-sm text-white/60">View all tokens launched through the program</p>
            </div>

            <div className="flex flex-col gap-4">
                {loading ? (
                    [1,2,3].map(i => (
                        <div key={i} className="p-4 bg-white/5 rounded-lg border border-white/10">
                            <div className="flex justify-between items-start mb-2">
                                <div className="space-y-2">
                                    <div className="h-5 w-24 bg-white/5 rounded animate-pulse"/>
                                    <div className="h-4 w-96 bg-white/5 rounded animate-pulse"/>
                                </div>
                                <div className="h-6 w-16 bg-white/5 rounded-full animate-pulse"/>
                            </div>
                            <div className="grid grid-cols-3 gap-8 mt-4">
                                {[1,2,3].map(j => (
                                    <div key={j} className="space-y-2">
                                        <div className="h-4 w-32 bg-white/5 rounded animate-pulse"/>
                                        <div className="h-5 w-24 bg-white/5 rounded animate-pulse"/>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ))
                ) : (
                        tokens.map(token => (
                            <Link 
                                href={`/swap/${token.tokenMint.toString()}`}
                                key={token.tokenMint.toString()}
                            >
                                <div className="p-4 bg-white/5 rounded-lg border border-white/10 hover:bg-white/10 transition-colors cursor-pointer">
                                    <div className="flex justify-between items-start mb-2">
                                        <div className="truncate">
                                            <h3 className="text-white font-medium">Token</h3>
                                            <p className="text-xs text-white/60 truncate">{token.tokenMint.toString()}</p>
                                        </div>
                                        <span className={`px-2 py-1 rounded-full text-xs ${
                                            token.isCompleted ? 'bg-green-500/20 text-green-300' : 'bg-blue-500/20 text-blue-300'
                                        }`}>
                                            {token.isCompleted ? 'Completed' : 'Active'}
                                        </span>
                                    </div>
                                    
                                    <div className="grid grid-cols-3 gap-8 mt-4 text-sm">
                                        <div>
                                            <p className="text-white/60">Total Supply</p>
                                            <p className="text-white">{token.tokenTotalSupply.toString()}</p>
                                        </div>
                                        <div>
                                            <p className="text-white/60">Virtual Token Reserve</p>
                                            <p className="text-white">{token.virtualTokenReserve.toString()}</p>
                                        </div>
                                        <div>
                                            <p className="text-white/60">Virtual SOL Reserve</p>
                                            <p className="text-white">{token.virtualSolReserve.toString()}</p>
                                        </div>
                                    </div>
                                </div>
                            </Link>
                    ))
                )}
            </div>
        </motion.div>
    );
}
