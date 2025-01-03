import { describe, it } from "mocha";
import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import type { Swifey } from "../target/types/swifey";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { expect } from "chai";
import { before } from "mocha";

const METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("swifey", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swifey as Program<Swifey>;
  const creator = Keypair.generate();
  const user = Keypair.generate();

  // Test data
  const name = "Test Token";
  const symbol = "TEST";
  const uri = "https://test.uri";
  const buyFeePercentage = 5;
  const sellFeePercentage = 5;
  const curveLimit = new anchor.BN(1000000000);

  let configPda: PublicKey;
  let tokenMint: PublicKey;
  let bondingCurvePda: PublicKey;
  let curveTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let metadataPda: PublicKey;

  before(async () => {
    // Airdrop SOL to creator and user
    await provider.connection.requestAirdrop(
      creator.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      user.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );

    // Add delay to confirm airdrop
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Derive PDAs
    [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_config")],
      program.programId
    );

    tokenMint = Keypair.generate().publicKey;

    [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), tokenMint.toBuffer()],
      program.programId
    );

    [metadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        tokenMint.toBuffer(),
      ],
      program.programId
    );
  });

  it("Can configure", async () => {
    const configuration = {
      admin: creator.publicKey,
      globalConfig: configPda,
      systemProgram: SystemProgram.programId,
    };
    await program.methods
      .configure({
        authority: creator.publicKey,
        feeRecipient: creator.publicKey,
        curveLimit: new anchor.BN(1000000000),
        initialVirtualTokenReserve: new anchor.BN(1000000000),
        initialVirtualSolReserve: new anchor.BN(10000000000),
        initialRealTokenReserve: new anchor.BN(100000000000),
        totalTokenSupply: new anchor.BN(100000000),
        buyFeePercentage: 5,
        sellFeePercentage: 5,
        migrationFeePercentage: 0,
      })
      .accounts(configuration)
      .signers([creator])
      .rpc();

    const config = await program.account.Config.fetch(configPda);
    expect(config.buyFeePercentage).to.equal(buyFeePercentage);
    expect(config.sellFeePercentage).to.equal(sellFeePercentage);
    expect(config.curveLimit.toString()).to.equal(curveLimit.toString());
  });

  it("Can launch", async () => {
    const mintKeypair = Keypair.generate();
    tokenMint = mintKeypair.publicKey;

    [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), tokenMint.toBuffer()],
      program.programId
    );

    // Get the ATA for the bonding curve's token account
    curveTokenAccount = await anchor.utils.token.associatedAddress({
      mint: tokenMint,
      owner: bondingCurvePda,
    });

    // Fix metadata PDA derivation
    [metadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );

    const launch = {
      creator: creator.publicKey,
      globalConfig: configPda,
      tokenMint: mintKeypair.publicKey,
      bondingCurve: bondingCurvePda,
      curveTokenAccount,
      tokenMetadataAccount: metadataPda,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      metadataProgram: METADATA_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    };

    await program.methods
      .launch(name, symbol, uri)
      .accounts(launch)
      .signers([creator, mintKeypair])
      .rpc();
  });

  describe("Swap tests", () => {
    it("Can swap (buy)", async () => {
      try {
        const userTokenAccount = await getAssociatedTokenAddress(
          tokenMint,
          user.publicKey
        );

        // Log initial balances
        const solBalance = await provider.connection.getBalance(user.publicKey);
        console.log(
          "Initial SOL balance:",
          solBalance / anchor.web3.LAMPORTS_PER_SOL
        );

        // Ensure enough SOL
        const signature = await provider.connection.requestAirdrop(
          user.publicKey,
          10 * anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction({
          signature,
          ...(await provider.connection.getLatestBlockhash()),
        });

        // Log balances after airdrop
        const newBalance = await provider.connection.getBalance(user.publicKey);
        console.log(
          "SOL balance after airdrop:",
          newBalance / anchor.web3.LAMPORTS_PER_SOL
        );

        // Use very small amount for testing
        const amount = new anchor.BN(10000);
        console.log("Attempting buy with amount:", amount.toString());
        const buyConfig = {
          user: user.publicKey,
          globalConfig: configPda,
          feeRecipient: creator.publicKey,
          bondingCurve: bondingCurvePda,
          tokenMint: tokenMint,
          curveTokenAccount: curveTokenAccount,
          userTokenAccount: userTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        };
        const tx = await program.methods
          .swap(amount, 0, new anchor.BN(1))
          .accounts(buyConfig)
          .signers([user])
          .rpc({
            skipPreflight: true,
            commitment: "confirmed",
          });

        console.log("Buy transaction signature:", tx);

        // Wait and verify the transaction
        await provider.connection.confirmTransaction(tx);

        const finalBalance = await provider.connection.getBalance(
          user.publicKey
        );
        console.log(
          "Final SOL balance:",
          finalBalance / anchor.web3.LAMPORTS_PER_SOL
        );
      } catch (error) {
        console.error("Detailed buy error:", error);
        if (error.logs) console.error("Transaction logs:", error.logs);
        throw error;
      }
    });

    it("Can swap (sell)", async () => {
      try {
        const bondingCurveAccount = await program.account.bondingCurve.fetch(
          bondingCurvePda
        );
        if (bondingCurveAccount.isCompleted) {
          console.log("Curve limit reached, skipping sell test");
          return;
        }

        const userTokenAccount = await getAssociatedTokenAddress(
          tokenMint,
          user.publicKey
        );

        // Get token balance
        const tokenBalance = await provider.connection.getTokenAccountBalance(
          userTokenAccount
        );
        console.log("Token balance before sell:", tokenBalance.value.uiAmount);

        // Use much smaller amount for sell
        const amount = new anchor.BN(1000);
        console.log("Attempting sell with amount:", amount.toString());
        const sellConfig = {
          user: user.publicKey,
          globalConfig: configPda,
          feeRecipient: creator.publicKey,
          bondingCurve: bondingCurvePda,
          tokenMint: tokenMint,
          curveTokenAccount: curveTokenAccount,
          userTokenAccount: userTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        };
        // Get the transaction instruction
        const ix = await program.methods
          .swap(amount, 1, new anchor.BN(1))
          .accounts(sellConfig)
          .instruction();

        // Create and send transaction
        const tx = new anchor.web3.Transaction().add(ix);
        const latestBlockhash = await provider.connection.getLatestBlockhash();
        tx.recentBlockhash = latestBlockhash.blockhash;
        tx.feePayer = user.publicKey;

        // Sign and send
        tx.sign(user);
        const txid = await provider.connection.sendRawTransaction(
          tx.serialize()
        );
        console.log("Sell transaction signature:", txid);

        // Wait for confirmation
        await provider.connection.confirmTransaction({
          signature: txid,
          ...latestBlockhash,
        });

        // Check final balances
        const finalTokenBalance =
          await provider.connection.getTokenAccountBalance(userTokenAccount);
        console.log("Final token balance:", finalTokenBalance.value.uiAmount);
      } catch (error) {
        console.error("Detailed sell error:", error);
        if (error.logs) console.error("Transaction logs:", error.logs);
        throw error;
      }
    });
  });
});
