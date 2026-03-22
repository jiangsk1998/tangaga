use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{self, Token2022}, token_interface::TokenAccount,
};

use crate::error::CustomError;

pub fn handle(ctx: Context<CloseAccount>) -> Result<()> {
    // 关闭账户前的检查
    require!(
        ctx.accounts.token_account.amount == 0,
        CustomError::InsufficientFunds
    );
    require!(
        ctx.accounts.token_account.owner == ctx.accounts.owner.key(),
        CustomError::NotOwnerOfToken
    );

    // 实际上 Anchor 的 close = owner 已经处理了 lamports 退还
    // 这里调用 CPI 是为了执行 SPL Token 的清理逻辑
    token_2022::close_account(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token_2022::CloseAccount {
            account: ctx.accounts.token_account.to_account_info(),
            destination: ctx.accounts.owner.to_account_info(), //退给谁
            authority: ctx.accounts.owner.to_account_info(),
        },
    ))?;
    Ok(())

    // 🗣️ 面试导向：框架的“黑盒”逻辑
    // 面试题：Anchor 的 close 属性是在什么时候生效的？如果 handler 报错了，账户还会被关闭吗？

    // 专业回答：
    // “close 属性是在指令生命周期的 Exit（退出）阶段生效的。
    // 它的执行前提是 handler 必须返回 Ok(())。如果业务逻辑触发了 return Err(...)，整个交易会回滚，Anchor 的自动关闭逻辑也就不会触发。这种设计保证了原子性：只有业务成功清算了余额，租金回收才会发生，防止了在逻辑失败时意外销毁账户导致资产丢失。”
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint=token_account.owner==owner.key() @ CustomError::NotOwnerOfToken,
        constraint=token_account.amount==0 @ CustomError::ZeroAmount,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}
