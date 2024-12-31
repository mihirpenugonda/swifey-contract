'use client'

import React, { useMemo } from 'react'
import {
  ConnectionProvider,
  WalletProvider,
} from '@solana/wallet-adapter-react'
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import { clusterApiUrl } from '@solana/web3.js'
import { RPC_ENDPOINT } from '../utils/constants'

// Default styles that can be overridden by your app
import '@solana/wallet-adapter-react-ui/styles.css'

export default function AppWalletProvider({
  children,
}: {
  children: React.ReactNode
}) {

  const network = WalletAdapterNetwork.Devnet
  const endpoint = useMemo(
    () => RPC_ENDPOINT || clusterApiUrl(network),
    [network, RPC_ENDPOINT],
  )

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={[]} autoConnect>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  )
}