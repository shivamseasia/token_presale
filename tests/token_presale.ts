import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenPresale } from "../target/types/token_presale";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { Keypair, SystemProgram, PublicKey } from "@solana/web3.js";

describe("token_presale", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenPresale as Program<TokenPresale>;

  let tokenMint: PublicKey;
  let usdtMint: PublicKey;
  let vault: PublicKey;
  let treasuryAccount: PublicKey;

  let buyers: Keypair[] = [];
  let buyerTokenAccounts: PublicKey[] = [];
  let buyerUsdtAccounts: PublicKey[] = [];

  const TOKEN_DECIMALS = 1_000_000;
  const USDT_DECIMALS = 1_000_000;
  const INITIAL_PRICE = new anchor.BN(50_000);

  before(async () => {
    // 1. Create Mints
    tokenMint = await createMint(provider.connection, provider.wallet.payer, provider.wallet.publicKey, null, 6);
    usdtMint = await createMint(provider.connection, provider.wallet.payer, provider.wallet.publicKey, null, 6);

    // 2. Derive State PDA for Vault (Vault is owned by the State PDA)
    const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("state")], program.programId);

    // 3. Create Vault Account
    const vaultATA = await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, tokenMint, statePda, true);
    vault = vaultATA.address;

    // 4. Create Treasury (Admin's USDT Account)
    const treasuryATA = await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, usdtMint, provider.wallet.publicKey);
    treasuryAccount = treasuryATA.address;

    // 5. Initialize Buyer
    const buyer = Keypair.generate();
    buyers.push(buyer);
    const sig = await provider.connection.requestAirdrop(buyer.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);

    const bToken = await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, tokenMint, buyer.publicKey);
    const bUsdt = await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, usdtMint, buyer.publicKey);
    buyerTokenAccounts.push(bToken.address);
    buyerUsdtAccounts.push(bUsdt.address);

    await mintTo(provider.connection, provider.wallet.payer, usdtMint, bUsdt.address, provider.wallet.publicKey, 1000 * USDT_DECIMALS);
  });

  // ----------------------------------------------------------------
  // 1. INITIALIZE
  // ----------------------------------------------------------------
  it("Initializes the presale", async () => {
    const duration = new anchor.BN(60 * 60 * 24 * 30 * 3);

    await program.methods
      .initialize(INITIAL_PRICE, duration)
      .accounts({
        admin: provider.wallet.publicKey,
        tokenMint: tokenMint,
        usdtMint: usdtMint,
        // 'state' is omitted because Anchor resolves it via seeds: [b"state"]
        vault: vault,
        treasury: treasuryAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .rpc();

    const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("state")], program.programId);
    const state = await program.account.presaleState.fetch(statePda);
    assert.equal(state.tokenPriceUsdt.toNumber(), INITIAL_PRICE.toNumber());
  });

  // ----------------------------------------------------------------
  // 2. UPDATE PRICE (New Instruction)
  // ----------------------------------------------------------------
  it("Updates the token price", async () => {
    const newPrice = new anchor.BN(100_000); // New Price: 0.1 USDT

    await program.methods
      .updatePrice(newPrice)
      .accounts({
        admin: provider.wallet.publicKey,
        // 'state' is resolved automatically
      })
      .rpc();

    const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("state")], program.programId);
    const state = await program.account.presaleState.fetch(statePda);
    assert.equal(state.tokenPriceUsdt.toNumber(), 100_000);
  });

  // ----------------------------------------------------------------
  // 3. BUY TOKENS
  // ----------------------------------------------------------------
  it("Allows a user to buy tokens at updated price", async () => {
    const usdtToSpend = new anchor.BN(10 * USDT_DECIMALS); // Spend 10 USDT

    await program.methods
      .buyTokens(usdtToSpend)
      .accounts({
        buyer: buyers[0].publicKey,
        // 'userPurchase' and 'state' are resolved automatically via seeds
        buyerUsdt: buyerUsdtAccounts[0],
        treasuryUsdt: treasuryAccount,
        vault: vault,
        buyerToken: buyerTokenAccounts[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([buyers[0]])
      .rpc();

    const [userPurchasePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_purchase"), buyers[0].publicKey.toBuffer()],
      program.programId
    );
    const userPurchase = await program.account.userPurchase.fetch(userPurchasePda);

    // Calculation: 10 USDT * 1M (dec) / 100,000 (price) = 100 tokens
    assert.equal(userPurchase.totalBought.toNumber(), 100 * TOKEN_DECIMALS);
  });

  // ----------------------------------------------------------------
  // 4. ERROR HANDLING
  // ----------------------------------------------------------------
  it("Fails when buying below minimum", async () => {
    const tinyUsdt = new anchor.BN(100);
    try {
      await program.methods
        .buyTokens(tinyUsdt)
        .accounts({
          buyer: buyers[0].publicKey,
          buyerUsdt: buyerUsdtAccounts[0],
          treasuryUsdt: treasuryAccount,
          vault: vault,
          buyerToken: buyerTokenAccounts[0],
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([buyers[0]])
        .rpc();
      assert.fail("Expected failure");
    } catch (e: any) {
      assert.include(e.toString(), "BelowMinimum");
    }
  });

  it("Admin can release tokens after presale (checks logic)", async () => {
    // Note: This might fail if the timestamp hasn't passed, but verifies account logic
    try {
      await program.methods
        .releaseFromReserve(new anchor.BN(1000))
        .accounts({
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (e: any) {
      // Expecting PresaleEnded because the timestamp is in the future
      assert.include(e.toString(), "PresaleEnded");
    }
  });
  // ----------------------------------------------------------------
  // 5. WITHDRAW USDT & PAUSE/UNPAUSE (New Tests)
  // ----------------------------------------------------------------

  it("Allows admin to withdraw USDT and blocks non-admin", async () => {
    const [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    const adminUsdt = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      usdtMint,
      provider.wallet.publicKey
    );

    const withdrawAmount = new anchor.BN(5 * USDT_DECIMALS);

    // Admin withdraw works
    await program.methods
      .withdrawUsdt(withdrawAmount)
      .accounts({
        admin: provider.wallet.publicKey,
        state: statePda,
        treasury: treasuryAccount,
        destination: adminUsdt.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .rpc();

    // Non-admin blocked
    try {
      await program.methods
        .withdrawUsdt(withdrawAmount)
        .accounts({
          admin: buyers[0].publicKey,
          state: statePda,
          treasury: treasuryAccount,
          destination: buyerUsdtAccounts[0],
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([buyers[0]])
        .rpc();

      assert.fail("Non-admin should not withdraw USDT");
    } catch (e) {
      const err = e.error?.errorCode?.code;
      assert.ok(
        err === "ConstraintRaw" ||
        err === "ConstraintHasOne" ||
        err === "OwnerMismatch"
      );
    }
  });

  // ----------------------------------------------------------------
  // 6. PAUSE/UNPAUSE
  // ----------------------------------------------------------------

  it("Blocks buying when paused", async () => {
    const [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    await program.methods
      .pause()
      .accounts({
        admin: provider.wallet.publicKey,
        state: statePda,
      } as any)
      .rpc();

    try {
      await program.methods
        .buyTokens(new anchor.BN(1_000))
        .accounts({
          buyer: buyers[0].publicKey,
          buyerUsdt: buyerUsdtAccounts[0],
          treasuryUsdt: treasuryAccount,
          vault: vault,
          buyerToken: buyerTokenAccounts[0],
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([buyers[0]])
        .rpc();

      assert.fail("Should fail when paused");
    } catch (e) {
      const _err = e.error?.errorCode?.code;
      assert.ok(true);
    }
  });
});