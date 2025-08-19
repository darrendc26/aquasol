import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Aquasol } from "../target/types/aquasol";
import { Keypair, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { assert } from "chai";
import { BN } from "bn.js";

describe("aquasol", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const connection = anchor.getProvider().connection;
  const program = anchor.workspace.Aquasol as Program<Aquasol>;
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  let admin: Keypair;
  let user: Keypair;
  let liquidMint: PublicKey;
  let feeAccount: PublicKey;
  let registry: PublicKey;
  let ytMint: PublicKey;
  let ptMint: PublicKey;
  let asset: PublicKey;


  before(async () => {
    // Generate keypairs
    admin = anchor.web3.Keypair.generate();
    user = anchor.web3.Keypair.generate();

    // Airdrop SOL to admin
    const airdropSignature = await connection.requestAirdrop(
      admin.publicKey,
      anchor.web3.LAMPORTS_PER_SOL // 1 SOL
    );
    await connection.confirmTransaction({
      signature: airdropSignature,
      blockhash: (await connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: (await connection.getLatestBlockhash()).lastValidBlockHeight,
    });
    console.log("Admin: ", admin.publicKey.toString());

    // Airdrop SOL to user
    const airdropSignature2 = await connection.requestAirdrop(
      user.publicKey,
      anchor.web3.LAMPORTS_PER_SOL // 1 SOL
    );
    await connection.confirmTransaction({
      signature: airdropSignature2,
      blockhash: (await connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: (await connection.getLatestBlockhash()).lastValidBlockHeight,
    });
    console.log("User: ", user.publicKey.toString());

    // Find registry PDA
    [registry] = await PublicKey.findProgramAddressSync(
      [Buffer.from("registry")],
      program.programId
    );
    console.log("Registry address: ", registry.toString());

    // Create liquid mint
    liquidMint = await createMint(
      provider.connection,      // connection
      provider.wallet.payer,    // fee payer
      admin.publicKey,          // mint authority
      null,                     // freeze authority
      9                         // decimals
    );
    console.log("Liquid mint created: ", liquidMint.toString());

    // Create fee ATA for the protocol
    const feeAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      liquidMint,
      admin.publicKey,          // owner of the fee account
    );
    feeAccount = feeAta.address;
    console.log("Fee account created: ", feeAccount.toString());
  });

  it("Initializes the registry", async () => {
    try {
      const tx = await program.methods
        .initRegistry()
        .accounts({
          admin: admin.publicKey,
          // registry: registry,
          liquidMint: liquidMint,
          feeAccount: feeAccount,
        })
        .signers([admin])
        .rpc();
      
      const registryAccount = await program.account.registry.fetch(registry);
      
      assert.equal(registryAccount.admin.toString(), admin.publicKey.toString());
      assert.equal(registryAccount.feeAccount.toString(), feeAccount.toString());
      assert.equal(registryAccount.commissionBps, 300);
        
    } catch (err) {
      console.error("Error initializing registry:", err);
      throw err;
    }
  });

  it("Lists an asset", async () => {
    let now = Math.floor(Date.now() / 1000);
    [asset] = await PublicKey.findProgramAddressSync(
      [Buffer.from("asset"), liquidMint.toBuffer()],
      program.programId
    );
    console.log("Asset address: ", asset.toString());

    ytMint = await createMint(
      provider.connection,      // connection
      provider.wallet.payer,    // fee payer
      admin.publicKey,          // mint authority
      null,                     // freeze authority
      9                         // decimals
    );
    console.log("YT mint created: ", ytMint.toString());

    ptMint = await createMint(
      provider.connection,      // connection
      provider.wallet.payer,    // fee payer
      admin.publicKey,          // mint authority
      null,                     // freeze authority
      9                         // decimals
    );
    console.log("PT mint created: ", ptMint.toString());

    try {
      const tx = await program.methods
  .listAsset(
    "aqSOL",          // asset_name: string
    ptMint,           // pt_mint: PublicKey
    ytMint,           // yt_mint: PublicKey
    new BN(8),        // expected_apy: u64
    new BN(1000000000), // yield_index: u64
    new BN(86400)     // duration: i64
  )
  .accounts({
    admin: admin.publicKey,
    tokenMint: liquidMint,    
  }).signers([admin])
  .rpc();

  const assetAccount = await program.account.asset.fetch(asset);

  assert.equal(assetAccount.name, "aqSOL");
  assert.equal(assetAccount.tokenMint.toString(), liquidMint.toString());
  assert.equal(assetAccount.ptMint.toString(), ptMint.toString());
  assert.equal(assetAccount.ytMint.toString(), ytMint.toString());
  assert.equal(assetAccount.totalTokens.toNumber(), 0);
assert.equal(assetAccount.expectedApy.toNumber(), 8);
assert.equal(assetAccount.isActive, true);
assert.equal(assetAccount.yieldIndex.toNumber(), 1_000_000_000);
assert.ok(assetAccount.maturityTs.sub(new BN(now + 86400)).abs().lte(new BN(1)));

  }catch(err){
    console.error("Error listing asset:", err);
    throw err;
  }
  });

});