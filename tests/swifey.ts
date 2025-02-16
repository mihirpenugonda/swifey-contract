import { describe, it } from "mocha";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
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
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert, expect } from "chai";
import { before } from "mocha";
import BN from "bn.js";

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
  let configBump: number;
  let tokenMint: Keypair;
  let bondingCurvePda: PublicKey;
  let curveTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let metadataPda: PublicKey;
  let wsolMint: PublicKey;
  let ammConfig: PublicKey;

  before(async () => {
    // Initialize WSOL mint (this is a well-known address on devnet/mainnet)
    wsolMint = new PublicKey("So11111111111111111111111111111111111111112");

    // Use the actual Raydium AMM config
    ammConfig = new PublicKey("GVSwm4smQBYcgAJU7qjFHLQBHTc4AdB3F2HbZp6KqKof");

    // Airdrop SOL to creator and user
    await provider.connection.requestAirdrop(
      creator.publicKey,
      100 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      user.publicKey,
      100 * anchor.web3.LAMPORTS_PER_SOL
    );

    // Add delay to confirm airdrop
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Derive PDAs
    const [configAddress, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_config")],
      program.programId
    );
    configPda = configAddress;
    configBump = bump;

    tokenMint = Keypair.generate();

    [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding_curve"), tokenMint.publicKey.toBuffer()],
      program.programId
    );

    [metadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        tokenMint.publicKey.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );

    // Initialize config account using Anchor's built-in initialization
    const BASE_SUPPLY = new BN("1000000000"); // 1 billion total tokens
    const DECIMALS = new BN(6); // 6 decimals to match TOKEN_DECIMAL constant
    const TOTAL_SUPPLY = BASE_SUPPLY.mul(new BN(10).pow(DECIMALS));
    const INITIAL_SOL = new BN(4 * anchor.web3.LAMPORTS_PER_SOL);
    const CURVE_LIMIT = new BN(80 * anchor.web3.LAMPORTS_PER_SOL);
    const reserved = Array(8)
      .fill(0)
      .map(() => Array(8).fill(0));

    const configSettings = {
      authority: creator.publicKey,
      feeRecipient: creator.publicKey,
      curveLimit: CURVE_LIMIT,
      initialVirtualTokenReserve: TOTAL_SUPPLY,
      initialVirtualSolReserve: INITIAL_SOL,
      initialRealTokenReserve: new BN(0),
      totalTokenSupply: TOTAL_SUPPLY,
      buyFeePercentage: new BN(100), // 1%
      sellFeePercentage: new BN(100), // 1%
      migrationFeePercentage: new BN(100), // 1%
      maxPriceImpact: new BN(10000000000), // 100%
      isPaused: false,
      reserved: reserved,
    };

    try {
      await program.methods
        .configure(configSettings)
        .accounts({
          admin: creator.publicKey,
          globalConfig: configPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();
    } catch (error) {
      // If the account is already initialized, that's fine
      if (!error.toString().includes("already in use")) {
        throw error;
      }
    }
  });

  describe("basic operations", () => {
    it("Can configure program settings", async () => {
      try {
        // Token supply calculations
        const BASE_SUPPLY = new BN("1000000000"); // 1 billion total tokens
        const DECIMALS = new BN(6); // Changed to 6 decimals to match TOKEN_DECIMAL constant
        const TOTAL_SUPPLY = BASE_SUPPLY.mul(new BN(10).pow(DECIMALS));

        // Initial SOL must be exactly 5 SOL per error code
        const INITIAL_SOL = new BN(4 * anchor.web3.LAMPORTS_PER_SOL);
        // Curve limit must be exactly 72 SOL per error code
        const CURVE_LIMIT = new BN(82 * anchor.web3.LAMPORTS_PER_SOL);

        // Create the correct nested array structure for reserved
        const reserved = Array(8)
          .fill(0)
          .map(() => Array(8).fill(0));

        const configSettings = {
          authority: creator.publicKey,
          feeRecipient: creator.publicKey,
          curveLimit: CURVE_LIMIT,
          initialVirtualTokenReserve: TOTAL_SUPPLY,
          initialVirtualSolReserve: INITIAL_SOL,
          initialRealTokenReserve: new BN(0),
          totalTokenSupply: TOTAL_SUPPLY,
          buyFeePercentage: new BN(100), // 1%
          sellFeePercentage: new BN(100), // 1%
          migrationFeePercentage: new BN(100), // 1%
          maxPriceImpact: new BN(10000000000), // 100%
          isPaused: false,
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
          .rpc();

        const config = await program.account.config.fetch(configPda);
        expect(config.authority.toString()).to.equal(
          creator.publicKey.toString()
        );
        expect(config.feeRecipient.toString()).to.equal(
          creator.publicKey.toString()
        );
        expect(config.curveLimit.eq(CURVE_LIMIT)).to.be.true;
        expect(config.initialVirtualSolReserve.eq(INITIAL_SOL)).to.be.true;
      } catch (error) {
        console.error("Configuration error:", error);
        throw error;
      }
    });

    it("can launch token", async () => {
      try {
        curveTokenAccount = await getAssociatedTokenAddress(
          tokenMint.publicKey,
          bondingCurvePda,
          true
        );

        console.log(bondingCurvePda);

        await program.methods
          .launch("Swifey Token", "SWFY", "https://swifey.io/metadata.json")
          .accounts({
            creator: creator.publicKey,
            globalConfig: configPda,
            tokenMint: tokenMint.publicKey,
            bondingCurve: bondingCurvePda,
            curveTokenAccount: curveTokenAccount,
            tokenMetadataAccount: metadataPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([creator, tokenMint])
          .rpc();

        const bondingCurve = await program.account.bondingCurve.fetch(
          bondingCurvePda
        );

        console.log(bondingCurve);
        expect(bondingCurve.isCompleted).to.be.false;
        expect(bondingCurve.isMigrated).to.be.false;
      } catch (error) {
        console.error("Launch error:", error);
        throw error;
      }
    });

    // it("Can buy tokens", async () => {
    //   try {
    //     userTokenAccount = await getAssociatedTokenAddress(
    //       tokenMint.publicKey,
    //       user.publicKey
    //     );

    //     const buyAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL); // 1 SOL

    //     await program.methods
    //       .swap(buyAmount, 0, new BN(0)) // direction 0 for buy
    //       .accounts({
    //         user: user.publicKey,
    //         globalConfig: configPda,
    //         feeRecipient: creator.publicKey,
    //         bondingCurve: bondingCurvePda,
    //         tokenMint: tokenMint.publicKey,
    //         curveTokenAccount: curveTokenAccount,
    //         userTokenAccount: userTokenAccount,
    //         tokenProgram: TOKEN_PROGRAM_ID,
    //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    //         systemProgram: SystemProgram.programId,
    //       })
    //       .signers([user])
    //       .rpc();

    //     const tokenBalance = await provider.connection.getTokenAccountBalance(
    //       userTokenAccount
    //     );
    //     console.log(`Received ${tokenBalance.value.amount} tokens for 1 SOL`);
    //     console.log("Token balance details:", tokenBalance.value);
    //   } catch (error) {
    //     console.error("Buy error:", error);
    //     throw error;
    //   }
    // });

    it("Can buy tokens with 72 SOL", async () => {
      try {
        userTokenAccount = await getAssociatedTokenAddress(
          tokenMint.publicKey,
          user.publicKey
        );

        const buyAmount = new BN(50 * anchor.web3.LAMPORTS_PER_SOL); // 72 SOL

        await program.methods
          .swap(buyAmount, 0, new BN(0)) // direction 0 for buy
          .accounts({
            user: user.publicKey,
            globalConfig: configPda,
            feeRecipient: creator.publicKey,
            bondingCurve: bondingCurvePda,
            tokenMint: tokenMint.publicKey,
            curveTokenAccount: curveTokenAccount,
            userTokenAccount: userTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .preInstructions([
            anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
              units: 1000000,
            }),
          ])
          .rpc();

        const userBalance = await provider.connection.getTokenAccountBalance(
          userTokenAccount
        );
        console.log(`Received ${userBalance.value.amount} tokens for 72 SOL`);
        console.log("Token balance details:", userBalance.value);
        expect(Number(userBalance.value.amount)).to.be.greaterThan(0);
      } catch (error) {
        console.error("Buy error:", error);
        throw error;
      }
    });

    it("Can sell tokens", async () => {
      try {
        // Get initial balances
        const userTokenBalance =
          await provider.connection.getTokenAccountBalance(userTokenAccount);
        const userSolBalanceBefore = await provider.connection.getBalance(
          user.publicKey
        );
        const feeRecipientBalanceBefore = await provider.connection.getBalance(
          creator.publicKey
        );
        const pdaBalanceBefore = await provider.connection.getBalance(
          bondingCurvePda
        );

        console.log("\nBalances Before Sell:");
        console.log(
          `User Token Balance: ${userTokenBalance.value.amount} tokens`
        );
        console.log(
          `User SOL Balance: ${
            userSolBalanceBefore / anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );
        console.log(
          `Fee Recipient SOL Balance: ${
            feeRecipientBalanceBefore / anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );
        console.log(
          `PDA SOL Balance: ${
            pdaBalanceBefore / anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );

        const sellAmount = new BN(userTokenBalance.value.amount);
        console.log(`\nSelling ${sellAmount.toString()} tokens`);

        await program.methods
          .swap(sellAmount, 1, new BN(0)) // direction 1 for sell
          .accounts({
            user: user.publicKey,
            globalConfig: configPda,
            feeRecipient: creator.publicKey,
            bondingCurve: bondingCurvePda,
            tokenMint: tokenMint.publicKey,
            curveTokenAccount: curveTokenAccount,
            userTokenAccount: userTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .preInstructions([
            anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
              units: 1000000,
            }),
          ])
          .rpc();

        // Get final balances
        const userSolBalanceAfter = await provider.connection.getBalance(
          user.publicKey
        );
        const feeRecipientBalanceAfter = await provider.connection.getBalance(
          creator.publicKey
        );
        const pdaBalanceAfter = await provider.connection.getBalance(
          bondingCurvePda
        );
        const userTokenBalanceAfter =
          await provider.connection.getTokenAccountBalance(userTokenAccount);

        console.log("\nBalances After Sell:");
        console.log(
          `User Token Balance: ${userTokenBalanceAfter.value.amount} tokens`
        );
        console.log(
          `User SOL Balance: ${
            userSolBalanceAfter / anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );
        console.log(
          `Fee Recipient SOL Balance: ${
            feeRecipientBalanceAfter / anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );
        console.log(
          `PDA SOL Balance: ${
            pdaBalanceAfter / anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );

        console.log("\nTransfer Summary:");
        console.log(
          `SOL transferred to user: ${
            (userSolBalanceAfter - userSolBalanceBefore) /
            anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );
        console.log(
          `SOL transferred to fee recipient: ${
            (feeRecipientBalanceAfter - feeRecipientBalanceBefore) /
            anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );
        console.log(
          `PDA SOL change: ${
            (pdaBalanceAfter - pdaBalanceBefore) / anchor.web3.LAMPORTS_PER_SOL
          } SOL`
        );
        console.log(
          `Tokens transferred from user: ${
            Number(userTokenBalance.value.amount) -
            Number(userTokenBalanceAfter.value.amount)
          } tokens`
        );

        expect(userSolBalanceAfter).to.be.greaterThan(userSolBalanceBefore);
        expect(feeRecipientBalanceAfter).to.be.greaterThan(
          feeRecipientBalanceBefore
        );
      } catch (error) {
        console.error("Sell error:", error);
        throw error;
      }
    });

    it("Can perform multiple buys and sells", async () => {
      try {
        // Reset the curve by relaunching
        tokenMint = Keypair.generate();
        [bondingCurvePda] = PublicKey.findProgramAddressSync(
          [Buffer.from("bonding_curve"), tokenMint.publicKey.toBuffer()],
          program.programId
        );
        [metadataPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            METADATA_PROGRAM_ID.toBuffer(),
            tokenMint.publicKey.toBuffer(),
          ],
          METADATA_PROGRAM_ID
        );
        curveTokenAccount = await getAssociatedTokenAddress(
          tokenMint.publicKey,
          bondingCurvePda,
          true
        );
        userTokenAccount = await getAssociatedTokenAddress(
          tokenMint.publicKey,
          user.publicKey
        );

        // Relaunch token
        await program.methods
          .launch("Swifey Token", "SWFY", "https://swifey.io/metadata.json")
          .accounts({
            creator: creator.publicKey,
            globalConfig: configPda,
            tokenMint: tokenMint.publicKey,
            bondingCurve: bondingCurvePda,
            curveTokenAccount: curveTokenAccount,
            tokenMetadataAccount: metadataPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: METADATA_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([creator, tokenMint])
          .rpc();

        console.log("Curve reset with new token mint");

        // First buy - 2 SOL to ensure minimum liquidity
        const buyAmount1 = new BN(2 * anchor.web3.LAMPORTS_PER_SOL);
        await program.methods
          .swap(buyAmount1, 0, new BN(0))
          .accounts({
            user: user.publicKey,
            globalConfig: configPda,
            feeRecipient: creator.publicKey,
            bondingCurve: bondingCurvePda,
            tokenMint: tokenMint.publicKey,
            curveTokenAccount: curveTokenAccount,
            userTokenAccount: userTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        let tokenBalance = await provider.connection.getTokenAccountBalance(
          userTokenAccount
        );
        console.log(
          `After first buy (2 SOL): ${tokenBalance.value.amount} tokens`
        );

        // Second buy - 5 SOL
        const buyAmount2 = new BN(5 * anchor.web3.LAMPORTS_PER_SOL);
        await program.methods
          .swap(buyAmount2, 0, new BN(0))
          .accounts({
            user: user.publicKey,
            globalConfig: configPda,
            feeRecipient: creator.publicKey,
            bondingCurve: bondingCurvePda,
            tokenMint: tokenMint.publicKey,
            curveTokenAccount: curveTokenAccount,
            userTokenAccount: userTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        tokenBalance = await provider.connection.getTokenAccountBalance(
          userTokenAccount
        );
        console.log(
          `After second buy (5 SOL): ${tokenBalance.value.amount} tokens`
        );

        // Sell all tokens
        const sellAmount = new BN(tokenBalance.value.amount).div(new BN(2));
        await program.methods
          .swap(sellAmount, 1, new BN(0)) // direction 1 for sell
          .accounts({
            user: user.publicKey,
            globalConfig: configPda,
            feeRecipient: creator.publicKey,
            bondingCurve: bondingCurvePda,
            tokenMint: tokenMint.publicKey,
            curveTokenAccount: curveTokenAccount,
            userTokenAccount: userTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        tokenBalance = await provider.connection.getTokenAccountBalance(
          userTokenAccount
        );
        console.log(
          `After selling all tokens: ${tokenBalance.value.amount} tokens`
        );
      } catch (error) {
        console.error("Multiple operations error:", error);
        throw error;
      }
    });

    it("Should fail when user has insufficient SOL balance", async () => {
      try {
        // Create a new user with minimal SOL
        const poorUser = Keypair.generate();

        // Airdrop just 0.5 SOL to the user
        await provider.connection.requestAirdrop(
          poorUser.publicKey,
          0.5 * anchor.web3.LAMPORTS_PER_SOL
        );
        await new Promise((resolve) => setTimeout(resolve, 1000)); // Wait for confirmation

        // Try to buy with 1 SOL (which is more than user has)
        const buyAmount = new BN(1 * anchor.web3.LAMPORTS_PER_SOL);
        const userTokenAccount = await getAssociatedTokenAddress(
          tokenMint.publicKey,
          poorUser.publicKey
        );

        // This should fail
        try {
          await program.methods
            .swap(buyAmount, 0, new BN(0))
            .accounts({
              user: poorUser.publicKey,
              globalConfig: configPda,
              feeRecipient: creator.publicKey,
              bondingCurve: bondingCurvePda,
              tokenMint: tokenMint.publicKey,
              curveTokenAccount: curveTokenAccount,
              userTokenAccount: userTokenAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([poorUser])
            .rpc();
          assert.fail("Should have failed due to insufficient balance");
        } catch (error) {
          expect(error.toString()).to.include("InsufficientUserBalance");
        }
      } catch (error) {
        console.error("Insufficient balance test error:", error);
        throw error;
      }
    });

    it("Should fail when user has insufficient token balance for sell", async () => {
      try {
        // Create a new user who has no tokens
        const noTokenUser = Keypair.generate();
        await provider.connection.requestAirdrop(
          noTokenUser.publicKey,
          1 * anchor.web3.LAMPORTS_PER_SOL
        );
        await new Promise((resolve) => setTimeout(resolve, 1000)); // Wait for confirmation

        // Create the token account first so it exists
        const userTokenAccount = await getAssociatedTokenAddress(
          tokenMint.publicKey,
          noTokenUser.publicKey
        );

        // Create the token account
        await program.methods
          .swap(new BN(0), 0, new BN(0)) // A dummy swap to create the account
          .accounts({
            user: noTokenUser.publicKey,
            globalConfig: configPda,
            feeRecipient: creator.publicKey,
            bondingCurve: bondingCurvePda,
            tokenMint: tokenMint.publicKey,
            curveTokenAccount: curveTokenAccount,
            userTokenAccount: userTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([noTokenUser])
          .rpc();

        // Try to sell tokens (which the user doesn't have)
        const sellAmount = new BN(1000000); // Try to sell 1 token
        try {
          await program.methods
            .swap(sellAmount, 1, new BN(0))
            .accounts({
              user: noTokenUser.publicKey,
              globalConfig: configPda,
              feeRecipient: creator.publicKey,
              bondingCurve: bondingCurvePda,
              tokenMint: tokenMint.publicKey,
              curveTokenAccount: curveTokenAccount,
              userTokenAccount: userTokenAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([noTokenUser])
            .rpc();
          assert.fail("Should have failed due to insufficient token balance");
        } catch (error) {
          // The error should be "insufficient funds" from the token program
          expect(error.toString()).to.include("insufficient funds");
          console.log(
            "Test passed: Transaction failed with insufficient funds as expected"
          );
        }
      } catch (error) {
        console.error("Insufficient token balance test error:", error);
        throw error;
      }
    });
  });
});
