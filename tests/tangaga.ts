import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Tangaga } from "../target/types/tangaga";
import { PublicKey, Keypair } from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";

describe("tangaga", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.tangaga as Program<Tangaga>;
  const provider = anchor.getProvider();
  const payer = (provider.wallet as any).payer;

  // 测试数据
  let mintKeypair: Keypair;
  let tokenName = "TangagaToken";
  let tokenSymbol = "TNG";
  let tokenUri = "https://example.com/token.json";
  let decimals = 6;

  // ============================================
  // 测试 1: 创建代币
  // ============================================
  it("Create Token", async () => {
    // 1. 生成新的 Mint 账户密钥对
    mintKeypair = Keypair.generate();

    // 2. 使用 Metaplex 获取 Metadata PDA 地址
    const metadataProgramId = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

    const [metadataAddress] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        metadataProgramId.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      metadataProgramId
    );

    // 3. 调用 create_token 指令
    const tx = await program.methods
      .createToken(tokenName, tokenSymbol, tokenUri, decimals)
      .accounts({
        mint: mintKeypair.publicKey,
        metadata: metadataAddress,
        authority: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenMetadataProgram: metadataProgramId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([mintKeypair])
      .rpc();

    console.log("Create Token 交易:", tx);
    expect(tx).to.be.a("string"); // 交易签名应该是字符串
  });

  // ============================================
  // 测试 2: 铸造代币到钱包
  // ============================================
  it("Mint to Wallet", async () => {
    // 1. 创建一个目标钱包（可以是任意地址，这里我们创建一个新的）
    const destinationWallet = Keypair.generate();

    // 2. 计算目标钱包的 ATA（Associated Token Account）
    const destinationAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      destinationWallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // 3. 铸造 100 个代币（decimals=6，所以实际数量是 100 * 10^6）
    const mintAmount = 100 * Math.pow(10, decimals);

    const tx = await program.methods
      .mintToWallet(new anchor.BN(mintAmount))
      .accounts({
        mint: mintKeypair.publicKey,
        destinationAta: destinationAta,
        destinationWallet: destinationWallet.publicKey,
        authority: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Mint to Wallet 交易:", tx);

    // 4. 验证：检查 ATA 的余额
    const ataInfo = await provider.connection.getTokenAccountBalance(destinationAta);
    console.log("ATA 余额:", ataInfo.value.amount);
    expect(Number(ataInfo.value.amount)).to.equal(mintAmount);
  });

  // ============================================
  // 测试 3: 转账代币
  // ============================================
  it("Transfer Tokens", async () => {
    // 1. 创建两个钱包：发送方和接收方
    const senderWallet = Keypair.generate();
    const receiverWallet = Keypair.generate();

    // 2. 计算双方的 ATA 地址
    const senderAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      senderWallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const receiverAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      receiverWallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // 3. 先给发送方铸造一些代币
    const mintAmount = 50 * Math.pow(10, decimals);
    await program.methods
      .mintToWallet(new anchor.BN(mintAmount))
      .accounts({
        mint: mintKeypair.publicKey,
        destinationAta: senderAta,
        destinationWallet: senderWallet.publicKey,
        authority: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

    // 4. 转账 10 个代币
    const transferAmount = 10 * Math.pow(10, decimals);
    const tx = await program.methods
      .transferTokens(new anchor.BN(transferAmount))
      .accounts({
        mint: mintKeypair.publicKey,
        fromAta: senderAta,
        toAta: receiverAta,
        toWallet: receiverWallet.publicKey,
        owner: senderWallet,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([senderWallet])
      .rpc();

    console.log("Transfer Tokens 交易:", tx);

    // 5. 验证：检查双方的余额
    const senderBalance = await provider.connection.getTokenAccountBalance(senderAta);
    const receiverBalance = await provider.connection.getTokenAccountBalance(receiverAta);

    console.log("发送方余额:", senderBalance.value.amount);
    console.log("接收方余额:", receiverBalance.value.amount);

    expect(Number(senderBalance.value.amount)).to.equal(
      mintAmount - transferAmount
    );
    expect(Number(receiverBalance.value.amount)).to.equal(transferAmount);
  });

  // ============================================
  // 测试 4: 错误处理 - 校验参数
  // ============================================
  it("Should fail with invalid parameters", async () => {
    const mintKeypair2 = Keypair.generate();
    const metadataProgramId = new PublicKey(
      "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
    );

    const [metadataAddress] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        metadataProgramId.toBuffer(),
        mintKeypair2.publicKey.toBuffer(),
      ],
      metadataProgramId
    );

    // 测试名称过长（应该失败）
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
          metadata: metadataAddress,
          authority: payer.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: metadataProgramId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([mintKeypair2])
        .rpc();

      throw new Error("应该因为名称过长而失败");
    } catch (err) {
      console.log("预期的错误:", err.message);
      expect(err.message).to.include("NameTooLong");
    }
  });
});
