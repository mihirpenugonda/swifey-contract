"use client";

import { useSwifey } from "@/hooks/useSwifey";
import { useState, useEffect } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { motion } from "framer-motion";
import { PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";

interface BondingCurve {
    virtualSolReserve: BN;
    virtualTokenReserve: BN;
    realSolReserve: BN;
    realTokenReserve: BN;
}

export default function TokenSwap({ tokenMint }: { tokenMint: string }) {
  const { swap, getTokenDetails, program } = useSwifey();
  const { connected } = useWallet();
  const [amount, setAmount] = useState("");
  const [slippage, setSlippage] = useState(""); // 1% default slippage
  const [isBuy, setIsBuy] = useState(true);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tokenExists, setTokenExists] = useState(false);
  const [reserves, setReserves] = useState<{
    virtualSol: string;
    virtualToken: string;
    realSol: string;
    realToken: string;
  } | null>(null);

  useEffect(() => {
    async function checkToken() {
      if (!program || !connected) {
        setLoading(false);
        return;
      }
      try {
        // Extract string from array if needed
        const mintAddress = Array.isArray(tokenMint) ? tokenMint[0] : tokenMint;
        console.log("Using mint address:", mintAddress);
        
        const tokenMintPubkey = new PublicKey(mintAddress);
        console.log("tokenMintPubkey", tokenMintPubkey.toString());
        
        const details = await getTokenDetails(tokenMintPubkey);
        setTokenExists(!!details);
      } catch (e) {
        console.error('Error checking token:', e);
        setTokenExists(false);
      } finally {
        setLoading(false);
      }
    }
    checkToken();
  }, [tokenMint, getTokenDetails, program, connected]);

  useEffect(() => {
    async function getReserves() {
      if (!program || !tokenMint) return;

      try {
        const mintAddress = Array.isArray(tokenMint) ? tokenMint[0] : tokenMint;
        const [bondingCurvePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("bonding_curve"), new PublicKey(mintAddress).toBuffer()],
          program.programId
        );

        const bondingCurve = await program.account.bondingCurve.fetch(bondingCurvePda) as unknown as BondingCurve;
        
        setReserves({
          virtualSol: bondingCurve.virtualSolReserve.toString(),
          virtualToken: bondingCurve.virtualTokenReserve.toString(),
          realSol: bondingCurve.realSolReserve.toString(),
          realToken: bondingCurve.realTokenReserve.toString(),
        });

        console.log("Current reserves:", {
          virtualSol: bondingCurve.virtualSolReserve.toString(),
          virtualToken: bondingCurve.virtualTokenReserve.toString(),
          realSol: bondingCurve.realSolReserve.toString(),
          realToken: bondingCurve.realTokenReserve.toString(),
        });
      } catch (e) {
        console.error("Error fetching reserves:", e);
      }
    }

    getReserves();
  }, [program, tokenMint]);

  if (!connected) return <div>Please connect your wallet</div>;
  if (loading) return <div>Checking token...</div>;
  if (!tokenExists) return <div>Invalid token mint address</div>;

  const handleSwap = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!connected) return;
    
    setLoading(true);
    setError(null);
    try {
      // Convert amount to lamports (assuming input is in SOL)
      const mintAddress = Array.isArray(tokenMint) ? tokenMint[0] : tokenMint;
      const amountBN = new BN(parseFloat(amount) * 1e9);
      const mintPubkey = new PublicKey(mintAddress);
      
      // Calculate minOut based on slippage
      const slippagePercent = parseFloat(slippage) || 1; // default to 1% if not set
      const minOutAmount = isBuy 
        ? amountBN.muln(100 - slippagePercent).divn(100)  // For buying tokens
        : amountBN.muln(100 - slippagePercent).divn(100); // For selling tokens

      const tx = await swap(
        mintPubkey,
        amountBN,
        isBuy ? 0 : 1,  // 0 for buy, 1 for sell
        minOutAmount    // Minimum tokens/SOL to receive
      );
      console.log("Swap executed!", tx);
    } catch (error) {
      console.error("Swap failed:", error);
      setError(error instanceof Error ? error.message : "Swap failed");
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
        <h2 className="text-xl font-medium text-white">Swap Tokens</h2>
        <p className="text-sm text-white/60">Trade tokens using the bonding curve</p>
      </div>

      <form onSubmit={handleSwap} className="space-y-4">
        <div>
          <input
            type="text"
            placeholder="Token Mint Address"
            value={tokenMint}
            readOnly
            className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white 
                     placeholder-white/40 focus:outline-none focus:border-white/20
                     text-sm transition-colors"
            required
          />
        </div>

        <div>
          <input
            type="number"
            step="any"
            min="0"
            placeholder={`Amount (in ${isBuy ? 'SOL' : 'tokens'})`}
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white 
                     placeholder-white/40 focus:outline-none focus:border-white/20
                     text-sm transition-colors"
            required
          />
        </div>

        <div>
          <input
            type="number"
            step="0.1"
            min="0.1"
            max="100"
            placeholder="Slippage %"
            value={slippage}
            onChange={(e) => setSlippage(e.target.value)}
            className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white 
                     placeholder-white/40 focus:outline-none focus:border-white/20
                     text-sm transition-colors"
            required
          />
        </div>

        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => setIsBuy(true)}
            className={`flex-1 py-2 rounded-lg text-sm font-medium transition-colors
                      ${isBuy 
                        ? 'bg-white/20 text-white' 
                        : 'bg-white/5 text-white/60 hover:bg-white/10'}`}
          >
            Buy
          </button>
          <button
            type="button"
            onClick={() => setIsBuy(false)}
            className={`flex-1 py-2 rounded-lg text-sm font-medium transition-colors
                      ${!isBuy 
                        ? 'bg-white/20 text-white' 
                        : 'bg-white/5 text-white/60 hover:bg-white/10'}`}
          >
            Sell
          </button>
        </div>

        {reserves && (
          <div className="mt-4 p-4 bg-white/5 rounded-lg">
            <h3 className="text-white text-sm font-medium mb-2">Current Reserves</h3>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-white/60">Virtual SOL</p>
                <p className="text-white">{reserves.virtualSol}</p>
              </div>
              <div>
                <p className="text-white/60">Virtual Token</p>
                <p className="text-white">{reserves.virtualToken}</p>
              </div>
              <div>
                <p className="text-white/60">Real SOL</p>
                <p className="text-white">{reserves.realSol}</p>
              </div>
              <div>
                <p className="text-white/60">Real Token</p>
                <p className="text-white">{reserves.realToken}</p>
              </div>
            </div>
          </div>
        )}

        {error && (
          <div className="text-sm text-red-400">
            {error}
          </div>
        )}

        <button
          type="submit"
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
              <span>Swapping...</span>
            </div>
          ) : (
            `Swap ${isBuy ? "SOL → Token" : "Token → SOL"}`
          )}
        </button>

        {!connected && (
          <p className="text-center text-xs text-white/60">
            Connect wallet to swap
          </p>
        )}
      </form>
    </motion.div>
  );
}
