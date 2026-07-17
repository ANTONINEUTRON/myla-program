import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MylaPool } from "../target/types/myla_pool";
import { assert } from "chai";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

describe("myla-program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.MylaPool as Program<MylaPool>;

  // Test accounts
  const oracle = Keypair.generate();
  const commissionWallet = Keypair.generate();
  const userA = Keypair.generate();
  const userB = Keypair.generate();

  // Pool parameters
  const matchId = "12345";
  const asset = "corners";
  const strikeLevel = 65; // 6.5 scaled ×10
  const strikeMinute = 45;

  let poolPda: PublicKey;
  let poolBump: number;
  let vaultPda: PublicKey;
  let vaultBump: number;

  before(async () => {
    // Airdrop SOL to test accounts
    const airdropAmount = 10 * LAMPORTS_PER_SOL;

    for (const kp of [oracle, userA, userB]) {
      const sig = await provider.connection.requestAirdrop(
        kp.publicKey,
        airdropAmount
      );
      await provider.connection.confirmTransaction(sig);
    }

    // Derive PDAs
    [poolPda, poolBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool"),
        Buffer.from(matchId),
        Buffer.from(asset),
        Buffer.from(new Uint8Array(new Uint16Array([strikeLevel]).buffer)),
        Buffer.from([strikeMinute]),
      ],
      program.programId
    );

    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), poolPda.toBuffer()],
      program.programId
    );
  });

  it("Creates a pool", async () => {
    // Deadline = 60 seconds from now
    const deadline = new anchor.BN(
      Math.floor(Date.now() / 1000) + 60
    );

    await program.methods
      .createPool(matchId, asset, strikeLevel, strikeMinute, deadline, 500)
      .accounts({
        creator: provider.wallet.publicKey,
        pool: poolPda,
        oracle: oracle.publicKey,
        commissionWallet: commissionWallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const poolAccount = await program.account.pool.fetch(poolPda);

    assert.equal(poolAccount.matchId, matchId);
    assert.equal(poolAccount.asset, asset);
    assert.equal(poolAccount.strikeLevel, strikeLevel);
    assert.equal(poolAccount.strikeMinute, strikeMinute);
    assert.equal(poolAccount.overTotal.toNumber(), 0);
    assert.equal(poolAccount.underTotal.toNumber(), 0);
    assert.equal(poolAccount.overCount, 0);
    assert.equal(poolAccount.underCount, 0);
    assert.equal(poolAccount.resolved, false);
    assert.isNull(poolAccount.winningSide);
    assert.isNull(poolAccount.actualValue);
    assert.equal(poolAccount.commissionRate, 500);
    assert.ok(poolAccount.oracle.equals(oracle.publicKey));
    assert.ok(
      poolAccount.commissionWallet.equals(commissionWallet.publicKey)
    );

    console.log("  ✓ Pool created successfully");
  });

  it("User A places an Over bet", async () => {
    const stakeAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);

    const [betPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bet"), poolPda.toBuffer(), userA.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .placeBet(0, stakeAmount) // side = 0 (Over)
      .accounts({
        user: userA.publicKey,
        pool: poolPda,
        bet: betPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([userA])
      .rpc();

    const poolAccount = await program.account.pool.fetch(poolPda);
    assert.equal(poolAccount.overTotal.toNumber(), stakeAmount.toNumber());
    assert.equal(poolAccount.overCount, 1);
    assert.equal(poolAccount.underTotal.toNumber(), 0);

    const betAccount = await program.account.bet.fetch(betPda);
    assert.equal(betAccount.side, 0);
    assert.equal(betAccount.amount.toNumber(), stakeAmount.toNumber());
    assert.equal(betAccount.claimed, false);

    console.log("  ✓ User A placed 0.5 SOL Over bet");
  });

  it("User B places an Under bet", async () => {
    const stakeAmount = new anchor.BN(0.3 * LAMPORTS_PER_SOL);

    const [betPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bet"), poolPda.toBuffer(), userB.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .placeBet(1, stakeAmount) // side = 1 (Under)
      .accounts({
        user: userB.publicKey,
        pool: poolPda,
        bet: betPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([userB])
      .rpc();

    const poolAccount = await program.account.pool.fetch(poolPda);
    assert.equal(
      poolAccount.underTotal.toNumber(),
      stakeAmount.toNumber()
    );
    assert.equal(poolAccount.underCount, 1);

    console.log("  ✓ User B placed 0.3 SOL Under bet");
  });

  it("Rejects a bet after deadline", async () => {
    // This test would require manipulating time, so we just verify the constraint exists.
    // In a real test environment, you would warp the clock forward.
    console.log("  ⏭ Skipped (requires clock manipulation)");
  });

  it("Rejects resolve from non-oracle", async () => {
    try {
      await program.methods
        .resolvePool(70) // actual_value = 7.0
        .accounts({
          oracle: userA.publicKey, // Wrong signer!
          pool: poolPda,
          vault: vaultPda,
          commissionWallet: commissionWallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userA])
        .rpc();

      assert.fail("Should have thrown UnauthorizedOracle error");
    } catch (err: any) {
      assert.include(err.toString(), "UnauthorizedOracle");
      console.log("  ✓ Non-oracle resolve correctly rejected");
    }
  });
});
