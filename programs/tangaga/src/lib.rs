mod error;
mod instructions;
use anchor_lang::prelude::*;
// use anchor_spl::token_2022::{self};

use instructions::*;

declare_id!("FZLKFcWiZbyyPqpyoG1uA1APveJC6Ex5e93wmaf63L9C");

#[program]
pub mod tangaga {

    use super::*;

    /// 1. create_token — 创建 Token-2022 Mint，内嵌 MetadataPointer + Metadata extension
    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
        decimals: u8,
    ) -> Result<()> {
        instructions::create_token(ctx, name, symbol, uri, decimals)
    }

    /// 2. mint_to_wallet — 铸造代币到指定钱包的 ATA
    pub fn mint_to_wallet(ctx: Context<MintToWallet>, amount: u64) -> Result<()> {
        instructions::mint_to_wallet(ctx, amount)
    }

    /// 3. transfer_tokens — 从一个钱包转代币到另一个钱包
    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        instructions::transfer_tokens(ctx, amount)
    }

    //     场景：你在 DEX 上挂了一个卖单，卖 100 USDC

    //     问题：DEX 合约怎么拿到你的 100 USDC？
    //       → 你不能直接把私钥给 DEX
    //       → 你不能预先把币转给 DEX（万一没成交呢？）

    //     解决方案：授权（Approve）
    //       1. 你授权 DEX 可以从你的账户转走最多 100 USDC
    //       2. DEX 在成交时调用 transfer，但 authority 是你的账户的 delegate
    //       3. 如果没成交，你可以 revoke 取消授权

    //     类比：
    //       就像你给支付宝设置了"每月自动扣款上限 500 元"
    //       支付宝可以在限额内代你付款，但不能超额
    /// 4. 实现授权指令（approve/revoke）：A 授权给 C，
    pub fn approve(ctx: Context<ApproveDelegate>, amount: u64) -> Result<()> {
        instructions::approve::handle(ctx, amount)
    }

    pub fn revoke(ctx: Context<RevokeDelegate>) -> Result<()> {
        instructions::revoke::handle(ctx)
    }

    //5.C 可以代 A 转账
    pub fn delegate(ctx: Context<DelegateTransfer>, amount: u64, decimals: u8) -> Result<()> {
        instructions::delegate_transfer::handle(ctx, amount, decimals)
    }

    //     销毁与增发相反：
    //       1. 从持有者 ATA 扣除 amount
    //       2. 从 Mint Account 扣除 supply

    //     任何人都可以销毁自己持有的代币（不需要 mint_authority）
    //     这相当于"永久性减少供应量"

    //     常见场景：
    //       - 项目方回购销毁（通缩模型）
    //       - 游戏内道具消耗
    //       - 交易手续费销毁
    // 6. 实现销毁（burn）：永久减少代币供应量
    pub fn burn(ctx: Context<BurnToken>, amount: u64) -> Result<()> {
        instructions::burn_token::handle(ctx, amount)
    }

    //      创建 Token Account 需要支付租金（rent）：
    //        - Token Account 大小：165 bytes
    //        - 租金：约 0.00203928 SOL（可退还）

    //      场景：你创建了一个 ATA 准备接收某代币，但交易取消了
    //            这个空 ATA 还在占用你的 SOL 租金

    //      解决方案：Close Account
    //        1. 将 Token Account 的 lamports 转回给所有者
    //        2. 账户数据被清零
    //        3. 回收租金

    //      安全前提：
    //        - 账户余额必须为 0（先转出或销毁所有代币）
    //        - 只有 owner 可以关闭
    // 7. 实现关闭账户（close_account）：回收创建 Token Account 的租金
    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        instructions::close_account::handle(ctx)
    }
}
