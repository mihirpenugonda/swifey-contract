import './globals.css'
import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import Providers from '@/components/providers'
import AppBar from '@/components/appbar'

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'Swifey Token Platform',
  description: 'Launch and trade tokens on Solana',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={`${inter.className} flex min-h-screen flex-col items-center p-24 bg-black`}>
        <Providers>
          <AppBar />
          {children}
        </Providers>
      </body>
    </html>
  )
}
