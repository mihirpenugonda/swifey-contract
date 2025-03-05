import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
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
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import BN from "bn.js";

// Add chai-bn for BigNumber assertions
const chai = require("chai");
const chaiAsPromised = require("chai-as-promised");
const chaiBN = require("chai-bn")(BN);
chai.use(chaiAsPromised);
chai.use(chaiBN);

// Constants
export const ESCROW_SEED = Buffer.from("escrow");
export const ESCROW_REGISTRY_SEED = Buffer.from("escrow-registry");
export const METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Test parameters
export const ESCROW_AMOUNT = new BN(1000000000); // 1 token with 9 decimals
export const INITIAL_MINT_AMOUNT = new BN(10000000000); // 10 tokens with 9 decimals

export interface TestContext {
  program: Program;
  mint: PublicKey;
  initiator: Keypair;
  initiatorTokenAccount: PublicKey;
  beneficiary: Keypair;
  beneficiaryTokenAccount: PublicKey;
  escrow: PublicKey;
  escrowBump: number;
  escrowTokenAccount: PublicKey;
  initiatorRegistry: PublicKey;
  initiatorRegistryBump: number;
  beneficiaryRegistry: PublicKey;
  beneficiaryRegistryBump: number;
}

export async function createTestContext(): Promise<TestContext> {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swifey as Program;

  // Create test accounts
  const initiator = Keypair.generate();
  const beneficiary = Keypair.generate();

  // Airdrop SOL to initiator
  const airdropSignature = await provider.connection.requestAirdrop(
    initiator.publicKey,
    2 * anchor.web3.LAMPORTS_PER_SOL
  );
  await provider.connection.confirmTransaction(airdropSignature);

  // Create test token mint
  const mint = await createMint(
    provider.connection,
    initiator,
    initiator.publicKey,
    null,
    9 // 9 decimals
  );

  // Create token accounts
  const initiatorTokenAccountInfo = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    initiator,
    mint,
    initiator.publicKey
  );
  const initiatorTokenAccount = initiatorTokenAccountInfo.address;

  const beneficiaryTokenAccountInfo = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    initiator,
    mint,
    beneficiary.publicKey
  );
  const beneficiaryTokenAccount = beneficiaryTokenAccountInfo.address;

  // Mint tokens to initiator
  await mintTo(
    provider.connection,
    initiator,
    mint,
    initiatorTokenAccount,
    initiator.publicKey,
    INITIAL_MINT_AMOUNT.toNumber()
  );

  // Derive PDAs
  const [escrow, escrowBump] = await PublicKey.findProgramAddress(
    [
      ESCROW_SEED,
      initiator.publicKey.toBuffer(),
      beneficiary.publicKey.toBuffer(),
      mint.toBuffer(),
    ],
    program.programId
  );

  const escrowTokenAccount = await getAssociatedTokenAddress(
    mint,
    escrow,
    true
  );

  const [initiatorRegistry, initiatorRegistryBump] =
    await PublicKey.findProgramAddress(
      [ESCROW_REGISTRY_SEED, initiator.publicKey.toBuffer()],
      program.programId
    );

  const [beneficiaryRegistry, beneficiaryRegistryBump] =
    await PublicKey.findProgramAddress(
      [ESCROW_REGISTRY_SEED, beneficiary.publicKey.toBuffer()],
      program.programId
    );

  return {
    program,
    mint,
    initiator,
    initiatorTokenAccount,
    beneficiary,
    beneficiaryTokenAccount,
    escrow,
    escrowBump,
    escrowTokenAccount,
    initiatorRegistry,
    initiatorRegistryBump,
    beneficiaryRegistry,
    beneficiaryRegistryBump,
  };
}

export async function createUniqueTestContext(
  existingContext: TestContext
): Promise<TestContext> {
  const newContext = await createTestContext();

  // Reuse the mint and program from existing context
  return {
    ...newContext,
    program: existingContext.program,
    mint: existingContext.mint,
  };
}
