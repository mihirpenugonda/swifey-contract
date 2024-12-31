"use client";

import { useSwifey } from "@/hooks/useSwifey";
import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { motion } from "framer-motion";

export default function TokenLaunch() {
  const { launch } = useSwifey();
  const { connected } = useWallet();
  const [name, setName] = useState("");
  const [symbol, setSymbol] = useState("");
  const [uri, setUri] = useState("");
  const [loading, setLoading] = useState(false);

  const handleLaunch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!connected) return;
    
    setLoading(true);
    try {
      const { tx, mint } = await launch(name, symbol, uri);
      console.log("Token launched!", { tx, mint: mint.toString() });
    } catch (error) {
      console.error("Launch failed:", error);
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
        <h2 className="text-xl font-medium text-white">Launch Token</h2>
        <p className="text-sm text-white/60">Create your token in seconds</p>
      </div>

      <form onSubmit={handleLaunch} className="space-y-4">
        <div>
          <input
            type="text"
            placeholder="Token Name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white 
                     placeholder-white/40 focus:outline-none focus:border-white/20
                     text-sm transition-colors"
            required
          />
        </div>

        <div>
          <input
            type="text"
            placeholder="Token Symbol"
            value={symbol}
            onChange={(e) => setSymbol(e.target.value)}
            className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white 
                     placeholder-white/40 focus:outline-none focus:border-white/20
                     text-sm transition-colors"
            required
          />
        </div>

        <div>
          <input
            type="url"
            placeholder="Metadata URI"
            value={uri}
            onChange={(e) => setUri(e.target.value)}
            className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white 
                     placeholder-white/40 focus:outline-none focus:border-white/20
                     text-sm transition-colors"
            required
          />
        </div>

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
              <span>Launching...</span>
            </div>
          ) : (
            "Launch Token"
          )}
        </button>

        {!connected && (
          <p className="text-center text-xs text-white/60">
            Connect wallet to launch
          </p>
        )}
      </form>
    </motion.div>
  );
}
