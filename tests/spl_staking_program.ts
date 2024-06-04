import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SplStakingProgram } from "../target/types/spl_staking_program";
import { createMint, mintTo, getMint, getOrCreateAssociatedTokenAccount, getAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { BN } from "bn.js";
import { assert } from "chai";

describe("spl_staking_program", () => {
  // Configure the client to use the local cluster.
  const provier = anchor.AnchorProvider.env();
  anchor.setProvider(provier);
  const program = anchor.workspace.SplStakingProgram as Program<SplStakingProgram>;
  const wallet = provier.wallet as anchor.Wallet;
  const connection = provier.connection;

  let mint = null;
  let userTokenAccount = null;
  let userInfoAccount = null;
  let vaultAccount = null;
  let vualtPDA = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault")], program.programId)[0];
  let vualtATA = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault_ata")], program.programId)[0];
  let userInfoPDA = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("user_info"), wallet.publicKey.toBuffer()], program.programId)[0];
  const minterAccount = anchor.web3.Keypair.generate();
  it("Initializes test state", async () => {
    // Add your test here.
    // const tx = await program.methods.initialize().rpc();
    // console.log("Your transaction signature", tx);
    console.log("My address", wallet.publicKey.toString());
    const balance = await connection.getBalance(wallet.publicKey);
    console.log("My balance", balance);

    const mintAuthority = wallet.publicKey;
    const freezeAuthority = wallet.publicKey;
    mint = await createMint(connection, wallet.payer, mintAuthority, freezeAuthority, 9);
    console.log("Mint", mint.toBase58());
    
    userTokenAccount = await getOrCreateAssociatedTokenAccount(connection, wallet.payer, mint, wallet.publicKey);
    console.log("tokenAccount", userTokenAccount.address.toBase58());
    await mintTo(connection, wallet.payer, mint, userTokenAccount.address, mintAuthority, 9999_000000000);

    let mintInfo = await getMint(connection, mint);
    console.log("supply", mintInfo.supply);
  });

  it("Initialize the Staking contract", async () => {
    const txHashInit = await program.methods
      .initializeStaking(wallet.publicKey)
      .accounts({
          mint: mint,
      })
      .rpc();

      console.log(`User 'solana confirm -v ${txHashInit}' to see the logs`);
  })
  it("Stake tokens", async () => {
    const txHashStake = await program.methods
      .stakeTokens(new BN(1_000000000))
      .accounts({
          mint: mint,
          fromAta: userTokenAccount.address,
      })
      .rpc();

      console.log(`User 'solana confirm -v ${txHashStake}' to see the logs`);
      
    userInfoAccount = await program.account.userInfo.fetch(userInfoPDA);
    vaultAccount = await program.account.vaultInfo.fetch(vualtPDA);
    assert(vaultAccount.amount.eq(new BN(1_000000000)));
    assert(userInfoAccount.stakedAmount.eq(new BN(1_000000000)));
    console.log("user", userInfoAccount.user);
    // assert(userInfoAccount.user.eq(wallet.publicKey));

    console.log("re", userInfoAccount.rewardAmount);
    assert.ok(userInfoAccount.user.equals(wallet.publicKey));

  })

  it("UnStake tokens", async () => {
    const txHashUnstake = await program.methods
    .unstakeTokens(new BN(1_000000000))
    .accounts({
        mint: mint,
        toAta: userTokenAccount.address,
    })
    .rpc();

    console.log(`User 'solana confirm -v ${txHashUnstake}' to see the logs`);

    userInfoAccount = await program.account.userInfo.fetch(userInfoPDA);
    vaultAccount = await program.account.vaultInfo.fetch(vualtPDA);
    console.log("vault account amount", vaultAccount.amount);
    assert(vaultAccount.amount.eq(new BN(0)));
    console.log("userInfoAccount.stakedAmount", userInfoAccount.stakedAmount);
    assert(userInfoAccount.stakedAmount.eq(new BN(0)));
  })
  
  it("Claim rewards", async () => {
    const txHashReward = await program.methods
    .claimReward()
    .accounts({
        mint: mint,
        toAta: userTokenAccount.address,
    })
    .rpc();

    console.log(`User 'solana confirm -v ${txHashReward}' to see the logs`);

    userInfoAccount = await program.account.userInfo.fetch(userInfoPDA);
    vaultAccount = await program.account.vaultInfo.fetch(vualtPDA);
    assert(vaultAccount.amount.eq(new BN(0)));
    console.log("reward Amount", userInfoAccount.rewardAmount);
    // assert(userInfoAccount.rewardAmount.eq(new BN(0)));
  })
});


