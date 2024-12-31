

import AppWalletProvider from "@/components/walletProvider";

export default function Providers({ children }: { children: React.ReactNode }) {
  return <AppWalletProvider>{children}</AppWalletProvider>;
}
