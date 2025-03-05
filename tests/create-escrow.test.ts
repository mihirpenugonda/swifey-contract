import { describe, it } from "mocha";
import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
} from "@solana/spl-token";
import { SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { assert, expect } from "chai";
import BN from "bn.js";
import {
  createTestContext,
  TestContext,
  ESCROW_AMOUNT,
  INITIAL_MINT_AMOUNT,
} from "./utils/setup";

describe("Create Escrow Tests", () => {
  let context: TestContext;

  before(async () => {
    context = await createTestContext();
  });

  it("Creates an escrow", async () => {
    await context.program.methods
      .createEscrow(
        ESCROW_AMOUNT,
        null // No expiry
      )
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

    // Verify escrow account data
    const escrowAccount = await context.program.account.escrow.fetch(
      context.escrow
    );
    expect(escrowAccount.initiator.toString()).to.equal(
      context.initiator.publicKey.toString()
    );
    expect(escrowAccount.beneficiary.toString()).to.equal(
      context.beneficiary.publicKey.toString()
    );
    expect(escrowAccount.escrowTokenAccount.toString()).to.equal(
      context.escrowTokenAccount.toString()
    );
    expect(escrowAccount.mint.toString()).to.equal(context.mint.toString());
    expect(escrowAccount.amount.toString()).to.equal(ESCROW_AMOUNT.toString());
    expect(escrowAccount.isClosed).to.be.false;
    expect(escrowAccount.bump).to.equal(context.escrowBump);

    // Verify token balances
    const initiatorTokenAccountInfo = await getAccount(
      context.program.provider.connection,
      context.initiatorTokenAccount
    );
    const escrowTokenAccountInfo = await getAccount(
      context.program.provider.connection,
      context.escrowTokenAccount
    );

    expect(initiatorTokenAccountInfo.amount.toString()).to.equal(
      INITIAL_MINT_AMOUNT.sub(ESCROW_AMOUNT).toString()
    );
    expect(escrowTokenAccountInfo.amount.toString()).to.equal(
      ESCROW_AMOUNT.toString()
    );

    // Verify registry updates
    const initiatorRegistryAccount =
      await context.program.account.escrowRegistry.fetch(
        context.initiatorRegistry
      );
    const beneficiaryRegistryAccount =
      await context.program.account.escrowRegistry.fetch(
        context.beneficiaryRegistry
      );

    expect(initiatorRegistryAccount.user.toString()).to.equal(
      context.initiator.publicKey.toString()
    );
    expect(beneficiaryRegistryAccount.user.toString()).to.equal(
      context.beneficiary.publicKey.toString()
    );

    expect(initiatorRegistryAccount.initiatedEscrows).to.have.lengthOf(1);
    expect(initiatorRegistryAccount.initiatedEscrows[0].toString()).to.equal(
      context.escrow.toString()
    );

    expect(beneficiaryRegistryAccount.beneficiaryEscrows).to.have.lengthOf(1);
    expect(
      beneficiaryRegistryAccount.beneficiaryEscrows[0].toString()
    ).to.equal(context.escrow.toString());
  });

  it("Fails to create escrow with insufficient funds", async () => {
    const poorContext = await createTestContext();

    // Attempt to create escrow with insufficient funds
    try {
      await poorContext.program.methods
        .createEscrow(
          INITIAL_MINT_AMOUNT.mul(new BN(2)), // Try to escrow more than minted
          null // No expiry
        )
        .accounts({
          initiator: poorContext.initiator.publicKey,
          beneficiary: poorContext.beneficiary.publicKey,
          initiatorTokenAccount: poorContext.initiatorTokenAccount,
          mint: poorContext.mint,
          escrow: poorContext.escrow,
          escrowTokenAccount: poorContext.escrowTokenAccount,
          initiatorRegistry: poorContext.initiatorRegistry,
          beneficiaryRegistry: poorContext.beneficiaryRegistry,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([poorContext.initiator])
        .rpc();

      assert.fail("Should have failed with insufficient funds");
    } catch (error) {
      expect(error.toString()).to.include("InsufficientTokenBalance");
    }
  });
});
