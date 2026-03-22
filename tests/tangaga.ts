import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Tangaga } from "../target/types/tangaga";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import { UseAuthorityRecordAlreadyExistsError } from "@metaplex-foundation/mpl-token-metadata";

// Token-2022 程序 ID
const TOKEN_2022_PROGRAM_ID = new PublicKey(
  "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
);

describe("tangaga", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.tangaga as Program<Tangaga>;
  const provider = anchor.getProvider();
  const payer = (provider.wallet as any).payer;

  let mintKeypair: Keypair;
  const tokenName = "Tangaga";
  const tokenSymbol = "TNG";
  const tokenUri = "https://example.com/token.json";
  const decimals = 6;


  let UserA = Keypair.generate()
  // ============================================
  // 测试 1: 创建代币
  // ============================================
  it("Create Token", async () => {
    mintKeypair = Keypair.generate();

    const tx = await program.methods
      .createToken(tokenName, tokenSymbol, tokenUri, decimals)
      .accounts({
        mint: mintKeypair.publicKey,
        authority: payer.publicKey,
        // systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([mintKeypair])
      .rpc();

    console.log("Create Token 交易:", tx);
    expect(tx).to.be.a("string");

    // 验证 mint 账户存在
    const mintInfo = await provider.connection.getAccountInfo(mintKeypair.publicKey);
    expect(mintInfo).to.not.be.null;
    expect(mintInfo!.owner.toBase58()).to.equal(TOKEN_2022_PROGRAM_ID.toBase58());
  });

  // ============================================
  // 测试 2: 铸造代币到钱包
  // ============================================
  it("Mint to Wallet", async () => {

    const destinationAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      UserA.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const mintAmount = 100 * Math.pow(10, decimals);

    const tx = await program.methods
      .mintToWallet(new anchor.BN(mintAmount))
      .accounts({
        mint: mintKeypair.publicKey,
        // destinationAta: destinationAta,
        destinationWallet: UserA.publicKey,
        authority: payer.publicKey,
        // systemProgram: SystemProgram.programId,
        // tokenProgram: TOKEN_2022_PROGRAM_ID,
        // associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      }).signers([payer])
      .rpc();

    console.log("Mint to Wallet 交易:", tx);

    const ataInfo = await provider.connection.getTokenAccountBalance(destinationAta);
    console.log("ATA 余额:", ataInfo.value.amount);
    expect(Number(ataInfo.value.amount)).to.equal(mintAmount);
  });

  // ============================================
  // 测试 3: 转账代币
  // ============================================
  const UserB = Keypair.generate();
  it("Transfer Tokens", async () => {
    // const senderWallet = Keypair.generate();
    // const receiverWallet = Keypair.generate();

    const senderAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      UserA.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const receiverAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      UserB.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // 先给发送方铸造代币
    // senderWallet 需要 SOL 来支付创建 ATA 的 rent（TransferTokens 里 owner 是 payer）
    const airdropSig = await provider.connection.requestAirdrop(
      UserA.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);


    // 转账
    const transferAmount = 10 * Math.pow(10, decimals);
    const tx = await program.methods
      .transferTokens(new anchor.BN(transferAmount))
      .accounts({
        mint: mintKeypair.publicKey,
        // fromAta: senderAta,
        // toAta: receiverAta,
        toWallet: UserB.publicKey,
        owner: UserA.publicKey,
        // systemProgram: SystemProgram.programId,
        // tokenProgram: TOKEN_2022_PROGRAM_ID,
        // associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([UserA])
      .rpc();

    console.log("Transfer Tokens 交易:", tx);

    const senderBalance = await provider.connection.getTokenAccountBalance(senderAta);
    const receiverBalance = await provider.connection.getTokenAccountBalance(receiverAta);

    console.log("发送方余额:", senderBalance.value.amount);
    console.log("接收方余额:", receiverBalance.value.amount);

    // expect(Number(senderBalance.value.amount)).to.equal(mintAmount - transferAmount);
    expect(Number(receiverBalance.value.amount)).to.equal(transferAmount);
  });

  // ============================================
  // 测试 4: 错误处理 - 校验参数
  // ============================================
  it("Should fail with invalid parameters", async () => {
    const mintKeypair2 = Keypair.generate();

    try {
      await program.methods
        .createToken(
          "A".repeat(33), // 超过 32 个字符
          "TNG",
          "https://example.com/token.json",
          6
        )
        .accounts({
          mint: mintKeypair2.publicKey,
          authority: payer.publicKey,
          // systemProgram: SystemProgram.programId,
          // tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([mintKeypair2])
        .rpc();

      throw new Error("应该因为名称过长而失败");
    } catch (err: any) {
      console.log("预期的错误:", err.message);
      expect(err.message).to.include("NameTooLong");
    }
  });


  const UseC = Keypair.generate();

  it("授权", async () => {
    await program.methods.approve(new anchor.BN(10 * Math.pow(10, decimals))).accounts({
      owner: UserA.publicKey,
      delegate: UserB.publicKey,
      tokenAccount: getAssociatedTokenAddressSync(
        mintKeypair.publicKey,
        UserA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      // mint: mintKeypair.publicKey,

    }).signers([UserA]).rpc();
  });


  it("授权转账", async () => {
    const airdrop = await provider.connection.requestAirdrop(UserB.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(airdrop)
    await program.methods.delegate(new anchor.BN(5 * Math.pow(10, decimals)), decimals).accounts({
      delegate: UserB.publicKey,
      fromAta: getAssociatedTokenAddressSync(
        mintKeypair.publicKey,
        UserA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      toOwner: UseC.publicKey,
      mint:mintKeypair.publicKey,
      toAta: getAssociatedTokenAddressSync(
        mintKeypair.publicKey,
        UseC.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
    }).signers([UserB]).rpc();
  })


  it("取消授权", async () => {
    await program.methods.revoke().accounts({
      owner: UserA.publicKey,
      tokenAccount: getAssociatedTokenAddressSync(
        mintKeypair.publicKey,
        UserA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
    }).signers([UserA]).rpc();
  });

  it("销毁", async () => {
    await program.methods.burn(new anchor.BN(5 * Math.pow(10, decimals))).accounts({
      mint: mintKeypair.publicKey,
      // fromAta: getAssociatedTokenAddressSync(
      //   mintKeypair.publicKey,
      //   UserA.publicKey,
      // ),
      owner: UserA.publicKey,
    }).signers([UserA]).rpc();

  });


  it("销户", async () => {
    await program.methods.closeAccount().accounts({
      owner: UserA.publicKey,
      tokenAccount: getAssociatedTokenAddressSync(
        mintKeypair.publicKey,
        UserA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
    }).signers([UserA]).rpc();

  });



});
