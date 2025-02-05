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
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { assert, expect } from "chai";
import { before } from "mocha";
import BN from "bn.js";
import { Transaction } from "@solana/web3.js";

// Add chai-bn for BigNumber assertions
const chai = require("chai");
const chaiAsPromised = require("chai-as-promised");
const chaiBN = require("chai-bn")(BN);
chai.use(chaiAsPromised);
chai.use(chaiBN);

const METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("swifey", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swifey as Program<Swifey>;
  const creator = Keypair.generate();
  const user = Keypair.generate();

  let configPda: PublicKey;
  let tokenMint: PublicKey;
  let bondingCurvePda: PublicKey;
  let curveTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let metadataPda: PublicKey;
  let wsolMint: PublicKey;
  let ammConfig: PublicKey;
  const RAYDIUM_V3_PROGRAM_ID = new PublicKey(
    "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK"
  );

  before(async () => {
    // Initialize WSOL mint (this is a well-known address on devnet/mainnet)
    wsolMint = new PublicKey("So11111111111111111111111111111111111111112");

    // Use the actual Raydium AMM config
    ammConfig = new PublicKey("GVSwm4smQBYcgAJU7qjFHLQBHTc4AdB3F2HbZp6KqKof");

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

    tokenMint = new PublicKey("FRMyHDzRexrZ5sVaWthsSW6pTRNnVmCbs1yDGRxGmpiC");

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

  describe("basic operations", () => {
    it("Can configure program settings", async () => {
      try {
        // Token supply calculations
        const BASE_SUPPLY = new BN("1000000000");
        const DECIMALS = new BN(6);
        const TOTAL_SUPPLY = BASE_SUPPLY.mul(new BN(10).pow(DECIMALS));

        // Must allocate at least 80% to bonding curve
        const VIRTUAL_RESERVE = BASE_SUPPLY.muln(8).divn(10); // 80% allocation
        const VIRTUAL_RESERVE_WITH_DECIMALS = VIRTUAL_RESERVE.mul(
          new BN(10).pow(DECIMALS)
        );

        // SOL amounts
        const INITIAL_SOL = new BN(12.33 * anchor.web3.LAMPORTS_PER_SOL);
        const CURVE_LIMIT = new BN(42).mul(
          new BN(anchor.web3.LAMPORTS_PER_SOL)
        );

        // Create the correct nested array structure for reserved
        const reserved = Array(8)
          .fill(0)
          .map(() => Array(8).fill(0));

        const configSettings = {
          authority: creator.publicKey,
          feeRecipient: creator.publicKey,
          curveLimit: CURVE_LIMIT,
          initialVirtualTokenReserve: VIRTUAL_RESERVE_WITH_DECIMALS,
          initialVirtualSolReserve: INITIAL_SOL,
          initialRealTokenReserve: new BN(0),
          totalTokenSupply: TOTAL_SUPPLY,
          buyFeePercentage: 0.5,
          sellFeePercentage: 0.5,
          migrationFeePercentage: 0.5,
          reserved: reserved,
        };

        await program.methods
          .configure(configSettings)
          .accounts({
            admin: creator.publicKey,
            globalConfig: configPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([creator])
          .rpc({
            skipPreflight: true,
            commitment: "processed",
          });

        const config = await program.account.config.fetch(configPda);
        expect(config.authority.toString()).to.equal(
          creator.publicKey.toString()
        );
        expect(config.feeRecipient.toString()).to.equal(
          creator.publicKey.toString()
        );
      } catch (error) {
        console.error("Configuration error:", error);
        if (error.logs) console.error("Transaction logs:", error.logs);
        throw error;
      }
    });

    it("Can launch token", async () => {
      try {
        // Create new token mint
        const tokenMintKeypair = Keypair.generate();
        tokenMint = tokenMintKeypair.publicKey;

        // Derive PDAs
        [bondingCurvePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("bonding_curve"), tokenMint.toBuffer()],
          program.programId
        );

        // Use the correct Metaplex Token Metadata Program ID
        const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
          "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        );

        [metadataPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            TOKEN_METADATA_PROGRAM_ID.toBuffer(),
            tokenMint.toBuffer(),
          ],
          TOKEN_METADATA_PROGRAM_ID
        );

        curveTokenAccount = await getAssociatedTokenAddress(
          tokenMint,
          bondingCurvePda,
          true
        );

        const tx = await program.methods
          .launch("Test Token", "TEST", "https://test.uri")
          .accounts({
            creator: creator.publicKey,
            globalConfig: configPda,
            tokenMint: tokenMint,
            bondingCurve: bondingCurvePda,
            curveTokenAccount: curveTokenAccount,
            tokenMetadataAccount: metadataPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([creator, tokenMintKeypair])
          .transaction();

        // Get latest blockhash
        const latestBlockhash = await provider.connection.getLatestBlockhash();
        tx.feePayer = creator.publicKey;
        tx.recentBlockhash = latestBlockhash.blockhash;

        // Send and confirm transaction
        const signature = await provider.connection.sendTransaction(
          tx,
          [creator, tokenMintKeypair],
          {
            skipPreflight: true,
            preflightCommitment: "confirmed",
          }
        );

        // Wait for confirmation
        const confirmation = await provider.connection.confirmTransaction({
          signature,
          blockhash: latestBlockhash.blockhash,
          lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        });

        if (confirmation.value.err) {
          // Get detailed logs for the failed transaction
          const txDetails = await provider.connection.getTransaction(
            signature,
            {
              commitment: "confirmed",
              maxSupportedTransactionVersion: 0,
            }
          );
          console.error("\n=== Transaction Error Details ===");
          console.error("Signature:", signature);
          console.error("Error:", confirmation.value.err);
          console.error("\nTransaction Logs:");
          if (txDetails?.meta?.logMessages) {
            txDetails.meta.logMessages.forEach((log, i) => {
              console.error(`${i}: ${log}`);
            });
          }
          throw new Error("Transaction failed with logs above");
        }

        // Verify the bonding curve was created
        const bondingCurve = await program.account.bondingCurve.fetch(
          bondingCurvePda
        );
        expect(bondingCurve.isCompleted).to.be.false;
        expect(bondingCurve.isMigrated).to.be.false;

        console.log("Token launched successfully");
        console.log("Token Mint:", tokenMint.toString());
        console.log("Transaction signature:", signature);
      } catch (error) {
        console.error("Launch error:", error);
        if (error.logs) console.error("Transaction logs:", error.logs);
        throw error;
      }
    });

    it("Can buy tokens", async () => {
      try {
        console.log("\n=== Starting Buy Test ===");

        // Get the config PDA with correct seed
        [configPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("global_config")],
          program.programId
        );

        // Get the bonding curve PDA with correct seeds
        [bondingCurvePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("bonding_curve"), tokenMint.toBuffer()],
          program.programId
        );

        console.log("PDAs:", {
          configPda: configPda.toString(),
          bondingCurvePda: bondingCurvePda.toString(),
        });

        // Create user token account
        userTokenAccount = await getAssociatedTokenAddress(
          tokenMint,
          user.publicKey
        );

        // Required accounts for swap
        const swapAccounts = {
          user: user.publicKey,
          globalConfig: configPda,
          feeRecipient: creator.publicKey,
          bondingCurve: bondingCurvePda,
          tokenMint,
          curveTokenAccount,
          userTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        };

        // Buy tokens
        const buyAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL);

        // Create ATA first
        try {
          const createAtaIx = createAssociatedTokenAccountInstruction(
            user.publicKey,
            userTokenAccount,
            user.publicKey,
            tokenMint
          );

          const createAtaTx = new Transaction().add(createAtaIx);
          const ataTxHash = await provider.connection.getLatestBlockhash(
            "confirmed"
          );
          createAtaTx.feePayer = user.publicKey;
          createAtaTx.recentBlockhash = ataTxHash.blockhash;

          await provider.connection.sendTransaction(createAtaTx, [user], {
            preflightCommitment: "confirmed",
          });
          console.log("Created user token account");
        } catch (e) {
          console.log("User token account might already exist");
        }

        // Buy transaction
        const buyTx = await program.methods
          .swap(buyAmount, 0, new BN(0))
          .accounts(swapAccounts)
          .signers([user])
          .transaction();

        const buyLatestBlockhash = await provider.connection.getLatestBlockhash(
          "confirmed"
        );
        buyTx.feePayer = user.publicKey;
        buyTx.recentBlockhash = buyLatestBlockhash.blockhash;

        console.log("Sending buy transaction...");
        const buySignature = await provider.connection.sendTransaction(
          buyTx,
          [user],
          {
            skipPreflight: true,
            preflightCommitment: "confirmed",
          }
        );

        await provider.connection.confirmTransaction(
          {
            signature: buySignature,
            blockhash: buyLatestBlockhash.blockhash,
            lastValidBlockHeight: buyLatestBlockhash.lastValidBlockHeight,
          },
          "confirmed"
        );

        // Get and log user's token balance
        const userTokenBalance =
          await provider.connection.getTokenAccountBalance(userTokenAccount);

        console.log("\n=== Buy Transaction Results ===");
        console.log("Tokens received:", userTokenBalance.value.amount);
        console.log(
          "User SOL balance:",
          (await provider.connection.getBalance(user.publicKey)) /
            anchor.web3.LAMPORTS_PER_SOL
        );

        // Get transaction data and decode events
        const transactionData =
          await program.provider.connection.getTransaction(buySignature, {
            commitment: "confirmed",
          });

        // Extract the CPI (inner instruction) that contains the event data
        const eventIx =
          transactionData.meta.innerInstructions[0].instructions[0];

        // Decode the event data
        const rawData = anchor.utils.bytes.bs58.decode(eventIx.data);
        const base64Data = anchor.utils.bytes.base64.encode(
          rawData.subarray(8)
        );
        const event = program.coder.events.decode(base64Data);
        console.log(event);

        console.log("\n=== Decoded Event Data ===");
        console.log(event);
      } catch (error) {
        console.error("\n=== Transaction Error ===");
        console.error("Error:", error);
        if (error.logs) {
          console.error("\nTransaction Logs:");
          error.logs.forEach((log: string, i: number) => {
            console.error(`${i}: ${log}`);
          });
        }
        throw error;
      }
    });

    it("Can sell tokens", async () => {
      try {
        console.log("\n=== Starting Sell Test ===");

        // Get initial balances
        const initialUserSolBalance = await provider.connection.getBalance(
          user.publicKey
        );
        const initialUserTokenBalance = (
          await provider.connection.getTokenAccountBalance(userTokenAccount)
        ).value.amount;

        console.log("Initial balances:", {
          sol: initialUserSolBalance / anchor.web3.LAMPORTS_PER_SOL,
          tokens: initialUserTokenBalance,
        });

        // Required accounts for swap (same as buy)
        const swapAccounts = {
          user: user.publicKey,
          globalConfig: configPda,
          feeRecipient: creator.publicKey,
          bondingCurve: bondingCurvePda,
          tokenMint,
          curveTokenAccount,
          userTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        };

        // Sell half of the tokens we have
        const sellAmount = new BN(initialUserTokenBalance).divn(2);

        console.log("\nSell amount:", sellAmount.toString());

        // Get latest blockhash
        const sellLatestBlockhash =
          await provider.connection.getLatestBlockhash();

        // Build sell transaction
        const sellTx = await program.methods
          .swap(sellAmount, 1, new BN(0)) // direction 1 for sell
          .accounts(swapAccounts)
          .signers([user])
          .transaction();

        // Send and confirm transaction
        const sellSignature = await provider.connection.sendTransaction(
          sellTx,
          [user],
          {
            skipPreflight: true,
            preflightCommitment: "confirmed",
            maxRetries: 3,
          }
        );

        // Wait for confirmation
        await provider.connection.confirmTransaction(
          {
            signature: sellSignature,
            blockhash: sellLatestBlockhash.blockhash,
            lastValidBlockHeight: sellLatestBlockhash.lastValidBlockHeight,
          },
          "confirmed"
        );

        // Get final balances
        const finalUserSolBalance = await provider.connection.getBalance(
          user.publicKey
        );
        const finalUserTokenBalance = (
          await provider.connection.getTokenAccountBalance(userTokenAccount)
        ).value.amount;

        console.log("\n=== Sell Transaction Results ===");
        console.log(
          "SOL received:",
          (finalUserSolBalance - initialUserSolBalance) /
            anchor.web3.LAMPORTS_PER_SOL
        );
        console.log(
          "Tokens sold:",
          Number(initialUserTokenBalance) - Number(finalUserTokenBalance)
        );
        console.log("Final token balance:", finalUserTokenBalance);

        // Get transaction logs
        const txLogs = await provider.connection.getTransaction(sellSignature, {
          maxSupportedTransactionVersion: 0,
          commitment: "confirmed"
        });
        console.log("\n=== Transaction Logs ===");
        txLogs?.meta?.logMessages?.forEach((log, i) => {
          console.log(`${i}: ${log}`);
        });
      } catch (error) {
        console.error("\n=== Transaction Error ===");
        console.error("Error:", error);

        // Get detailed transaction logs
        if (error.logs) {
          console.error("\nTransaction Logs:");
          error.logs.forEach((log: string, i: number) => {
            console.error(`${i}: ${log}`);
          });
        }

        // Get program error details if available
        if (error.error) {
          console.error("\nProgram Error Details:");
          console.error("Error Code:", error.error.errorCode);
          console.error("Error Message:", error.error.errorMessage);
        }

        // Get instruction error details if available
        if (error.instruction) {
          console.error("\nInstruction Error Details:");
          console.error("Failed Instruction Index:", error.instruction);
          console.error("Failed Instruction Data:", error.instructionData);
        }

        throw error;
      }
    });
  });
});
