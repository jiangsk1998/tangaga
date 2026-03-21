// ── 错误码 ──────────────────────────────────────────────────────────────────

use anchor_lang::error_code;

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

    #[msg("不是代币的所有者")]
    NotOwnerOfToken,

    #[msg("余额不足，无法完成转账")]
    InsufficientFunds,
    
    #[msg("转入账户的 mint 与预期不匹配")]
    MintMismatch,
}
