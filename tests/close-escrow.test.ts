import { describe, it } from "mocha";
import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } from "@solana/web3.js";
import { assert, expect } from "chai";
import BN from "bn.js";
import {
  createTestContext,
  createUniqueTestContext,
  TestContext,
  ESCROW_AMOUNT,
} from "./utils/setup";

describe("Close Escrow Tests", () => {
  let context: TestContext;

  beforeEach(async () => {
    // Create fresh context for each test
    context = await createTestContext();

    // Create escrow for testing
    await context.program.methods
      .createEscrow(ESCROW_AMOUNT, null)
      .accounts({
        initiator: context.initiator.publicKey,
        beneficiary: context.beneficiary.publicKey,
        initiatorTokenAccount: context.initiatorTokenAccount,
        mint: context.mint,
        escrow: context.escrow,
        escrowTokenAccount: context.escrowTokenAccount,
        initiatorRegistry: context.initiatorRegistry,
        beneficiaryRegistry: context.beneficiaryRegistry,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([context.initiator])
      .rpc();
  });

  it("Closes an escrow with single recipient", async () => {
    await context.program.methods
      .closeEscrow([
        {
          recipient: context.beneficiary.publicKey,
          amount: new BN(100), // 100%
        },
      ])
      .accounts({
        initiator: context.initiator.publicKey,
        escrow: context.escrow,
        escrowTokenAccount: context.escrowTokenAccount,
        mint: context.mint,
        recipientTokenAccount: context.beneficiaryTokenAccount,
        initiatorRegistry: context.initiatorRegistry,
        beneficiaryRegistry: context.beneficiaryRegistry,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([context.initiator])
      .rpc();

    // Verify escrow is closed
    const escrowAccount = await context.program.account.escrow.fetch(
      context.escrow
    );
    expect(escrowAccount.isClosed).to.be.true;

    // Verify token balances
    const beneficiaryTokenAccountInfo = await getAccount(
      context.program.provider.connection,
      context.beneficiaryTokenAccount
    );
    expect(beneficiaryTokenAccountInfo.amount.toString()).to.equal(
      ESCROW_AMOUNT.toString()
    );

    // Verify escrow token account is closed
    try {
      await getAccount(
        context.program.provider.connection,
        context.escrowTokenAccount
      );
      assert.fail("Escrow token account should be closed");
    } catch (error) {
      // Expected error
    }

    // Verify registry updates
    const initiatorRegistryAccount =
      await context.program.account.escrowRegistry.fetch(
        context.initiatorRegistry
      );
    const beneficiaryRegistryAccount =
      await context.program.account.escrowRegistry.fetch(
        context.beneficiaryRegistry
      );

    expect(initiatorRegistryAccount.initiatedEscrows).to.have.lengthOf(0);
    expect(beneficiaryRegistryAccount.beneficiaryEscrows).to.have.lengthOf(0);
  });

  it("Closes an escrow with multiple recipients", async () => {
    // Create new recipients
    const recipient1 = Keypair.generate();
    const recipient2 = Keypair.generate();

    // Create token accounts for recipients
    const recipient1TokenAccount = await getOrCreateAssociatedTokenAccount(
      context.program.provider.connection,
      context.initiator,
      context.mint,
      recipient1.publicKey
    );

    const recipient2TokenAccount = await getOrCreateAssociatedTokenAccount(
      context.program.provider.connection,
      context.initiator,
      context.mint,
      recipient2.publicKey
    );

    // Close escrow with multiple recipients
    await context.program.methods
      .closeEscrowMulti([
        {
          recipient: recipient1.publicKey,
          amount: new BN(60), // 60%
        },
        {
          recipient: recipient2.publicKey,
          amount: new BN(40), // 40%
        },
      ])
      .accounts({
        initiator: context.initiator.publicKey,
        escrow: context.escrow,
        escrowTokenAccount: context.escrowTokenAccount,
        initiatorRegistry: context.initiatorRegistry,
        beneficiaryRegistry: context.beneficiaryRegistry,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: recipient1TokenAccount.address,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: recipient2TokenAccount.address,
          isWritable: true,
          isSigner: false,
        },
      ])
      .signers([context.initiator])
      .rpc();

    // Verify escrow is closed
    const escrowAccount = await context.program.account.escrow.fetch(
      context.escrow
    );
    expect(escrowAccount.isClosed).to.be.true;

    // Verify token balances
    const recipient1Balance = await getAccount(
      context.program.provider.connection,
      recipient1TokenAccount.address
    );
    const recipient2Balance = await getAccount(
      context.program.provider.connection,
      recipient2TokenAccount.address
    );

    const expectedAmount1 = ESCROW_AMOUNT.mul(new BN(60)).div(new BN(100));
    const expectedAmount2 = ESCROW_AMOUNT.mul(new BN(40)).div(new BN(100));

    expect(recipient1Balance.amount.toString()).to.equal(
      expectedAmount1.toString()
    );
    expect(recipient2Balance.amount.toString()).to.equal(
      expectedAmount2.toString()
    );
  });

  it("Fails to close escrow with invalid distribution percentages", async () => {
    try {
      await context.program.methods
        .closeEscrow([
          {
            recipient: context.beneficiary.publicKey,
            amount: new BN(80), // Only 80%
          },
        ])
        .accounts({
          initiator: context.initiator.publicKey,
          escrow: context.escrow,
          escrowTokenAccount: context.escrowTokenAccount,
          mint: context.mint,
          recipientTokenAccount: context.beneficiaryTokenAccount,
          initiatorRegistry: context.initiatorRegistry,
          beneficiaryRegistry: context.beneficiaryRegistry,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([context.initiator])
        .rpc();

      assert.fail("Should have failed with invalid distribution percentages");
    } catch (error) {
      expect(error.message).to.include("InvalidDistributionAmounts");
    }
  });

  it("Fails to close escrow with unauthorized signer", async () => {
    const unauthorized = Keypair.generate();

    // Airdrop SOL to unauthorized signer
    const airdropSignature =
      await context.program.provider.connection.requestAirdrop(
        unauthorized.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );
    await context.program.provider.connection.confirmTransaction(
      airdropSignature
    );

    try {
      await context.program.methods
        .closeEscrow([
          {
            recipient: context.beneficiary.publicKey,
            amount: new BN(100),
          },
        ])
        .accounts({
          initiator: unauthorized.publicKey,
          escrow: context.escrow,
          escrowTokenAccount: context.escrowTokenAccount,
          mint: context.mint,
          recipientTokenAccount: context.beneficiaryTokenAccount,
          initiatorRegistry: context.initiatorRegistry,
          beneficiaryRegistry: context.beneficiaryRegistry,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([unauthorized])
        .rpc();

      assert.fail("Should have failed with unauthorized signer");
    } catch (error) {
      expect(error.message).to.include("UnauthorizedEscrowClose");
    }
  });

  it("Allows beneficiary to close escrow", async () => {
    // Airdrop SOL to beneficiary
    const airdropSignature =
      await context.program.provider.connection.requestAirdrop(
        context.beneficiary.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );
    await context.program.provider.connection.confirmTransaction(
      airdropSignature
    );

    await context.program.methods
      .closeEscrow([
        {
          recipient: context.beneficiary.publicKey,
          amount: new BN(100),
        },
      ])
      .accounts({
        initiator: context.beneficiary.publicKey,
        escrow: context.escrow,
        escrowTokenAccount: context.escrowTokenAccount,
        mint: context.mint,
        recipientTokenAccount: context.beneficiaryTokenAccount,
        initiatorRegistry: context.initiatorRegistry,
        beneficiaryRegistry: context.beneficiaryRegistry,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([context.beneficiary])
      .rpc();

    const escrowAccount = await context.program.account.escrow.fetch(
      context.escrow
    );
    expect(escrowAccount.isClosed).to.be.true;
  });

  it("Creates an escrow with expiry and fails to close after expiry", async () => {
    const expiryContext = await createUniqueTestContext(context);
    const currentTime = Math.floor(Date.now() / 1000);

    // Create escrow with expiry in the past
    await expiryContext.program.methods
      .createEscrow(
        ESCROW_AMOUNT,
        new BN(currentTime - 10) // Expired 10 seconds ago
      )
      .accounts({
        initiator: expiryContext.initiator.publicKey,
        beneficiary: expiryContext.beneficiary.publicKey,
        initiatorTokenAccount: expiryContext.initiatorTokenAccount,
        mint: expiryContext.mint,
        escrow: expiryContext.escrow,
        escrowTokenAccount: expiryContext.escrowTokenAccount,
        initiatorRegistry: expiryContext.initiatorRegistry,
        beneficiaryRegistry: expiryContext.beneficiaryRegistry,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([expiryContext.initiator])
      .rpc();

    try {
      await expiryContext.program.methods
        .closeEscrow([
          {
            recipient: expiryContext.beneficiary.publicKey,
            amount: new BN(100),
          },
        ])
        .accounts({
          initiator: expiryContext.initiator.publicKey,
          escrow: expiryContext.escrow,
          escrowTokenAccount: expiryContext.escrowTokenAccount,
          mint: expiryContext.mint,
          recipientTokenAccount: expiryContext.beneficiaryTokenAccount,
          initiatorRegistry: expiryContext.initiatorRegistry,
          beneficiaryRegistry: expiryContext.beneficiaryRegistry,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([expiryContext.initiator])
        .rpc();

      assert.fail("Should have failed with expired escrow");
    } catch (error) {
      expect(error.message).to.include("EscrowExpired");
    }
  });
});
