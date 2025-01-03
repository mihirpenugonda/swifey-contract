import TokenSwap from '@/components/tokenSwap';

export default function Page({ params }: { params: { tokenMint: string } }) {
    const { tokenMint } = params;
    return <TokenSwap tokenMint={tokenMint} />
}
