'use client'

import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { motion } from "framer-motion";
import Link from "next/link";

export default function AppBar() {
  return (
    <motion.div 
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="fixed top-0 left-0 right-0 z-50 backdrop-blur-sm bg-black/20 border-b border-white/15"
    >
      <div className="max-w-7xl mx-auto px-4 py-3">
        <div className="flex items-center justify-between">
          <Link href="/">
          <motion.div 
            className="flex items-center space-x-3"
            whileHover={{ scale: 1.01 }}
          >
            <div className="w-8 h-8 rounded-lg bg-white/10 flex items-center justify-center">
              <span className="text-white font-medium">S</span>
            </div>
            <span className="text-white font-medium">Swifey</span>
          </motion.div>
          </Link>

          <div className="hidden md:flex items-center space-x-4">
            {['Launch', 'Swap'].map((item) => (
              <motion.div
                key={item}
                whileHover={{ scale: 1.02 }}
                className="text-white/80 hover:text-white px-3 py-1.5 rounded-lg text-sm"
              >
                <Link href={`/${item.toLowerCase()}`}>{item}</Link>
              </motion.div>
            ))}
          </div>

          <WalletMultiButton/>
        </div>
      </div>
    </motion.div>
  );
}
