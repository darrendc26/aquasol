import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Aquasol } from "../target/types/aquasol";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";
import { BN } from "bn.js";
import { set } from "@coral-xyz/anchor/dist/cjs/utils/features";

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
  let vault: PublicKey;
  let userTokenAccount: PublicKey;
  let userYtAccount: PublicKey;
  let userPtAccount: PublicKey;
  let YTamm : PublicKey;

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
    const [registryPda, registryBump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("registry")],
      program.programId
    );
    registry = registryPda;
    console.log("Registry address: ", registry.toString());

    // Create liquid mint
    liquidMint = await createMint(
      provider.connection,      // connection
      admin,    // fee payer
      admin.publicKey,          // mint authority
      null,                     // freeze authority
      9                         // decimals
    );
    console.log("Liquid mint created: ", liquidMint.toString());

    const userTokenAccountInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      liquidMint,
      user.publicKey,          // owner of the fee account
    );
    userTokenAccount = userTokenAccountInfo.address; // Assign to the top-level variable
    console.log("User token account created: ", userTokenAccount.toBase58());

    // Mint to user token account
    await mintTo(
      provider.connection,      // connection
      admin,    // fee payer
      liquidMint,               // mint
      userTokenAccount,         // destination
      admin,          // mint authority
      1000000000                // amount
    );
    console.log("Minted tokens to user account: ", userTokenAccount.toBase58());

    // Create fee ATA for the protocol
    const feeAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
     admin,
      liquidMint,
      admin.publicKey,          // owner of the fee account
    );
    feeAccount = feeAta.address;
    console.log("Fee account created: ", feeAccount.toString());

    ytMint = await createMint(
      provider.connection,      // connection
      admin,    // fee payer
      registry,          // mint authority
      null,                     // freeze authority
      9                         // decimals
    );
    console.log("YT mint created: ", ytMint.toString());

    ptMint = await createMint(
      provider.connection,      // connection
      admin,    // fee payer
      registry,          // mint authority
      null,                     // freeze authority
      9                         // decimals
    );
    console.log("PT mint created: ", ptMint.toString());
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

    try {
      const tx = await program.methods
        .listAsset(
          "aqSOL",          // asset_name: string
          ptMint,           // pt_mint: PublicKey
          ytMint,           // yt_mint: PublicKey
          new BN(8),        // expected_apy: u64
          new BN(1000000000), // yield_index: u64
          new BN(10)     // duration: i64
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
      assert.equal(assetAccount.duration.toNumber(), 10);
      assert.equal(assetAccount.yieldIndex.toNumber(), 1_000_000_000);
      assert.ok(assetAccount.maturityTs.sub(new BN(now + 10)).abs().lte(new BN(1)));

    } catch(err) {
      console.error("Error listing asset:", err);
      throw err;
    }
  });

  it("Strips an asset", async () => {
    let now = Math.floor(Date.now() / 1000);
    const [userYtPosition] = await PublicKey.findProgramAddressSync(
      [Buffer.from("user_yt_position"), user.publicKey.toBuffer()],
      program.programId
    );
    console.log("User YT position address: ", userYtPosition.toString());

    const [registry] = await PublicKey.findProgramAddressSync(
      [Buffer.from("registry")],
      program.programId
    );
    console.log("Registry address: ", registry.toString());

    const [userYtAccountATA] = await PublicKey.findProgramAddressSync(
      [Buffer.from("user_yt_account"), user.publicKey.toBuffer()],
      program.programId
    );
    userYtAccount = userYtAccountATA;
    console.log("User YT account address: ", userYtAccount.toString());

    const [userPtAccountATA] = await PublicKey.findProgramAddressSync(
      [Buffer.from("user_pt_account"), user.publicKey.toBuffer()],
      program.programId
    );
    userPtAccount = userPtAccountATA;
    console.log("User PT account address: ", userPtAccount.toString());

    
    const vaultAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin, // fee payer
      liquidMint,
      registry, // owner (the PDA)
      true // allowOwnerOffCurve - required for PDA owners (true)
    );
    vault = vaultAta.address;
    console.log("Vault ATA address: ", vault.toString());


    await mintTo(
      provider.connection,      // connection
      admin,    // fee payer
      liquidMint,               // mint
      vault,         // destination
      admin,          // mint authority
      10000000000                // amount
    );
    console.log("Minted tokens to vault: ", vault.toBase58());


    try {
      const tx = await program.methods
        .strip(new BN(1000000000))
        .accounts({
          user: user.publicKey,
          asset: asset,
          // registry: registry,
          // userYtPosition: userYtPosition,
          userTokenAccount: userTokenAccount, 
          vault: vault,
          // userPtAccount: userPtAccount,
          ptMint: ptMint,
          // userYtAccount: userYtAccount,
          ytMint: ytMint,
          // tokenProgram: TOKEN_PROGRAM_ID,
          // associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          // systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const userYtPositionAccount = await program.account.userYtPosition.fetch(userYtPosition);
      assert.equal(userYtPositionAccount.user.toString(), user.publicKey.toString());
      assert.equal(userYtPositionAccount.totalYtTokens.toNumber(), 1000000000);
      assert.equal(userYtPositionAccount.accruedYield.toNumber(), 0);
      assert.ok(userYtPositionAccount.lastUpdateTs.sub(new BN(now)).abs().lte(new BN(1)));

    const finalUserTokenBalance = await connection.getTokenAccountBalance(userTokenAccount);
    const vaultBalance = await connection.getTokenAccountBalance(vault);
    const userPtAccountBalance = await connection.getTokenAccountBalance(userPtAccount);
    const userYtAccountBalance = await connection.getTokenAccountBalance(userYtAccount);

      assert.equal(userYtAccountBalance.value.amount, "1000000000");
      assert.equal(vaultBalance.value.amount, "11000000000");
      assert.equal(userPtAccountBalance.value.amount, "1000000000");
      assert.equal(finalUserTokenBalance.value.amount, "0");

    } catch (err) {
      console.error("Error stripping asset:", err);
      throw err;
    }
  });

  it("Claims YT yield", async () => {
    let now = Math.floor(Date.now() / 1000);
    const [userYtPosition] = await PublicKey.findProgramAddressSync(
      [Buffer.from("user_yt_position"), user.publicKey.toBuffer()],
      program.programId
    );
    console.log("User YT position address: ", userYtPosition.toString());

    try {
     await new Promise((resolve) => setTimeout(resolve, 5000));

await program.methods
  .claimYield()
  .accounts({
    user: user.publicKey,
    asset: asset,
    userTokenAccount: userTokenAccount, 
    vault: vault,
  })
  .signers([user])
  .rpc();

    const userTokenAccountBalance = await connection.getTokenAccountBalance(userTokenAccount);
    const vaultBalance = await connection.getTokenAccountBalance(vault);
    const userYtAccountBalance = await connection.getTokenAccountBalance(userYtAccount);

    console.log("User YT account balance: ", userYtAccountBalance.value.amount);
    console.log("Vault balance: ", vaultBalance.value.amount);
    console.log("User token account balance: ", userTokenAccountBalance.value.amount);

    assert.ok(userYtAccountBalance.value.uiAmount = 11000000000);
    assert.ok(vaultBalance.value.uiAmount < 1000000000);
    assert.ok(userTokenAccountBalance.value.uiAmount > 0);
  } catch (err) {
      console.error("Error claiming yield:", err);
      throw err;
    }
  });

  it("Redeems an asset", async () => {
    const [userPtAccountATA] = await PublicKey.findProgramAddressSync(
      [Buffer.from("user_pt_account"), user.publicKey.toBuffer()],
      program.programId
    );
    userPtAccount = userPtAccountATA;
    console.log("User PT account address: ", userPtAccount.toString());
    let now = Math.floor(Date.now() / 1000);

    

    try {
      await new Promise((resolve) => setTimeout(resolve, 15000));
      await program.methods
      .redeem(new BN(1000000000))
      .accounts({
        user: user.publicKey,
        asset: asset,
        userTokenAccount: userTokenAccount, 
        vault: vault,
        ptMint: ptMint,
        // userPtAccount: userPtAccount,
      })
      .signers([user])
      .rpc();
    } catch (err) {
      console.error("Error redeeming asset:", err);
      throw err;
    }

  });
  
});