mod instructions;
use anchor_lang::prelude::*;
use anchor_spl::
    token_2022::{self}
;

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
}



// ── 错误码 ──────────────────────────────────────────────────────────────────

#[error_code]
pub enum CustomError {
    #[msg("代币名称不能超过 32 个字符")]
    NameTooLong,
    
    #[msg("代币符号不能超过 10 个字符")]
    SymbolTooLong,

    #[msg("URI 不能超过 200 个字符")]
    UriTooLong,

    #[msg("铸造/转账数量必须大于 0")]
    ZeroAmount,

    #[msg("不是授权的铸币者")]
    UnauthorizedMinter,
    
}
