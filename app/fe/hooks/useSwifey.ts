"use client"

import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import * as anchor from "@coral-xyz/anchor";
import { useCallback, useEffect, useState } from "react";
import { SWIFEY_PROGRAM_ID } from "@/utils/constants";
import IDL from "@/utils/swifey.json";
import { PublicKey } from "@solana/web3.js";
import { Idl } from "@coral-xyz/anchor";
import { 
    TOKEN_PROGRAM_ID, 
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
    getAccount
} from "@solana/spl-token";

const METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

declare global {
    interface Window {
        solana: any;
    }
}

export type ConfigureParams = {
    authority: PublicKey;
    feeRecipient: PublicKey;
    curveLimit: anchor.BN;
    initialVirtualTokenReserve: anchor.BN;
    initialVirtualSolReserve: anchor.BN;
    initialRealTokenReserve: anchor.BN;
    totalTokenSupply: anchor.BN;
    buyFeePercentage: number;
    sellFeePercentage: number;
    migrationFeePercentage: number;
};

export interface LaunchedToken {
  tokenMint: PublicKey;
  virtualTokenReserve: anchor.BN;
  virtualSolReserve: anchor.BN;
  realTokenReserve: anchor.BN;
  realSolReserve: anchor.BN;
  tokenTotalSupply: anchor.BN;
  isCompleted: boolean;
}

export const useSwifey = () => {
    const { connection } = useConnection();
    const { publicKey, sendTransaction } = useWallet();
    const [program, setProgram] = useState<anchor.Program<Idl> | null>(null);

    useEffect(() => {
        if (!publicKey) return;

        const provider = new anchor.AnchorProvider(
            connection,
            window.solana,
            { commitment: "confirmed", preflightCommitment: "confirmed" }
        );
        anchor.setProvider(provider);

        const program = new anchor.Program(IDL as Idl, SWIFEY_PROGRAM_ID, provider);
        setProgram(program);
    }, [publicKey, connection]);

    const configure = useCallback(async (params: ConfigureParams) => {
        if (!program || !publicKey) throw new Error("Program not initialized");

        const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("global_config")],
            program.programId
        );

        const tx = await program.methods
            .configure(params)
            .accounts({
                admin: publicKey,
                globalConfig: configPda,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([])
            .transaction();

        const latestBlockhash = await connection.getLatestBlockhash();
        tx.feePayer = publicKey;
        tx.recentBlockhash = latestBlockhash.blockhash;
        
        const signed = await sendTransaction(tx, connection, { 
            signers: [],
            skipPreflight: true,
            preflightCommitment: 'processed',
        });
        
        // Simple delay to allow transaction to process
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        return signed;
    }, [program, publicKey, connection, sendTransaction]);

    const launch = useCallback(async (
        name: string,
        symbol: string,
        uri: string
    ) => {
        if (!program || !publicKey) throw new Error("Program not initialized");

        const tokenMint = anchor.web3.Keypair.generate();
        
        const [bondingCurvePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("bonding_curve"), tokenMint.publicKey.toBuffer()],
            program.programId
        );

        const [configPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("global_config")],
            program.programId
        );

        const [metadataPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("metadata"),
                METADATA_PROGRAM_ID.toBuffer(),
                tokenMint.publicKey.toBuffer(),
            ],
            METADATA_PROGRAM_ID
        );

        const curveTokenAccount = await getAssociatedTokenAddress(
            tokenMint.publicKey,
            bondingCurvePda,
            true
        );

        const tx = await program.methods
            .launch(name, symbol, uri)
            .accounts({
                creator: publicKey,
                globalConfig: configPda,
                tokenMint: tokenMint.publicKey,
                bondingCurve: bondingCurvePda,
                curveTokenAccount: curveTokenAccount,
                tokenMetadataAccount: metadataPda,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                metadataProgram: METADATA_PROGRAM_ID,
                systemProgram: anchor.web3.SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY
            })
            .signers([tokenMint])
            .transaction();

        const latestBlockhash = await connection.getLatestBlockhash();
        tx.feePayer = publicKey;
        tx.recentBlockhash = latestBlockhash.blockhash;
        
        const signed = await sendTransaction(tx, connection, { 
            signers: [tokenMint],
            skipPreflight: true,
            preflightCommitment: 'processed',
        });

        // Simple delay to allow transaction to process
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        return { tx: signed, mint: tokenMint.publicKey };
    }, [program, publicKey, connection, sendTransaction]);

    const swap = useCallback(async (
        tokenMint: PublicKey,
        amount: anchor.BN,
        direction: number,
        minOut: anchor.BN
    ) => {
        if (!program || !publicKey) throw new Error("Program not initialized");

        const [bondingCurvePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("bonding_curve"), tokenMint.toBuffer()],
            program.programId
        );

        console.log("Swap attempt:", {
            mint: tokenMint.toString(),
            bondingCurvePda: bondingCurvePda.toString(),
            amount: amount.toString(),
            direction
        });

        // Add account existence check
        const bondingCurveAccount = await connection.getAccountInfo(bondingCurvePda);
        if (!bondingCurveAccount) {
            throw new Error("Bonding curve not initialized for this token");
        }


        const [configPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("global_config")],
            program.programId
        );

        const config = await program.account.config.fetch(configPda);
        console.log("Config:", config);
        const feeRecipientPda = config.feeRecipient;

        // Get or create token accounts
        const curveTokenAccount = await getAssociatedTokenAddress(
            tokenMint,
            bondingCurvePda,
            true
        );

        const userTokenAccount = await getAssociatedTokenAddress(
            tokenMint,
            publicKey
        );

        const tx = await program.methods
            .swap(
                amount,
                direction,
                minOut
            )
            .accounts({
                user: publicKey,
                globalConfig: configPda,
                feeRecipient: feeRecipientPda as PublicKey,
                bondingCurve: bondingCurvePda,
                tokenMint: tokenMint,
                curveTokenAccount: curveTokenAccount,
                userTokenAccount: userTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([])
            .transaction();

        const latestBlockhash = await connection.getLatestBlockhash();
        tx.feePayer = publicKey;
        tx.recentBlockhash = latestBlockhash.blockhash;
        
        const signed = await sendTransaction(tx, connection, { 
            signers: [],
            skipPreflight: true,
            preflightCommitment: 'processed',
        });

        // Simple delay to allow transaction to process
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        return signed;
    }, [program, publicKey, connection, sendTransaction]);

    const getLaunchedTokens = useCallback(async () => {
        if (!program || !publicKey) throw new Error("Program not initialized");

        // Get config account
        const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("global_config")],
            program.programId
        );

        // Get recent signatures for the program
        const signatures = await connection.getSignaturesForAddress(
            program.programId,
            { limit: 1000 }
        );

        const tokens = new Set<string>();
        const launchedTokens: LaunchedToken[] = [];

        // Process each transaction to find launch instructions
        for (const sig of signatures) {
            try {
                const tx = await connection.getParsedTransaction(sig.signature, {
                    maxSupportedTransactionVersion: 0
                });

                // Skip if transaction not found or failed
                if (!tx || !tx.meta || tx.meta.err) continue;

                // Look for launch instruction in the transaction
                const launchInstr = tx.transaction.message.instructions.find(
                    ix => ix.programId.equals(program.programId)
                );

                if (launchInstr && 'accounts' in launchInstr) {
                    const tokenMint = launchInstr.accounts[2];
                    
                    // Skip if we've already processed this token
                    if (tokens.has(tokenMint.toString())) continue;
                    
                    try {
                        const bondingCurvePda = PublicKey.findProgramAddressSync(
                            [Buffer.from("bonding_curve"), tokenMint.toBuffer()],
                            program.programId
                        )[0];

                        // Add account existence check
                        const accountInfo = await connection.getAccountInfo(bondingCurvePda);
                        if (accountInfo) {  // Only process if account exists
                            const bondingCurve = await program.account.bondingCurve.fetch(bondingCurvePda);
                            tokens.add(tokenMint.toString());
                            launchedTokens.push({
                                tokenMint,
                                virtualTokenReserve: bondingCurve.virtualTokenReserve,
                                virtualSolReserve: bondingCurve.virtualSolReserve,
                                realTokenReserve: bondingCurve.realTokenReserve,
                                realSolReserve: bondingCurve.realSolReserve,
                                tokenTotalSupply: bondingCurve.tokenTotalSupply,
                                isCompleted: Boolean(bondingCurve.isCompleted)
                            });
                            console.log("Launched token:", launchedTokens[launchedTokens.length - 1]);
                        }
                    } catch (e) {
                        console.error('Error fetching bonding curve:', e);
                    }
                }
            } catch (e) {
                console.error('Error processing transaction:', e);
                continue;
            }
        }

        return launchedTokens;
    }, [program, publicKey, connection]);

    const getTokenDetails = useCallback(async (mint: PublicKey) => {
        if (!program || !publicKey) throw new Error("Program not initialized");
        console.log("getTokenDetails", mint.toBuffer());

        const [bondingCurvePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("bonding_curve"), mint.toBuffer()],
            program.programId
        );
        console.log("bondingCurvePda", bondingCurvePda.toString());
        // Check if account exists first
        const accountInfo = await connection.getAccountInfo(bondingCurvePda);
        if (!accountInfo) {
            console.log("Account not found");
            return null;
        }

        try {
            const bondingCurve = await program.account.bondingCurve.fetch(bondingCurvePda);
            return {
                tokenMint: mint,
                virtualTokenReserve: bondingCurve.virtualTokenReserve,
                virtualSolReserve: bondingCurve.virtualSolReserve,
                realTokenReserve: bondingCurve.realTokenReserve,
                realSolReserve: bondingCurve.realSolReserve,
                tokenTotalSupply: bondingCurve.tokenTotalSupply,
                isCompleted: bondingCurve.isCompleted
            };
        } catch (e) {
            console.error('Error fetching token details:', e);
            return null;
        }
    }, [program, publicKey, connection]);

    return {
        program,
        configure,
        launch,
        swap,
        getLaunchedTokens,
        getTokenDetails
    }
};