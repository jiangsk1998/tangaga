use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{self, Revoke, Token2022}, token_interface::TokenAccount,
};

use crate::error::CustomError;

#[derive(Accounts)]
pub struct RevokeDelegate<'info> {
    //原所有者
    #[account(mut)]
    pub owner: Signer<'info>,

    //要授权的Token Account
    #[account(
        mut,
        constraint=token_account.owner==owner.key() @ CustomError::NotOwnerOfToken
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    //token_program
    pub token_program: Program<'info, Token2022>,
}

pub fn handle(ctx: Context<RevokeDelegate>) -> Result<()> {
    let accounts = Revoke {
        source: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), accounts);

    token_2022::revoke(cpi_context)?;
    msg!("授权已取消");
    Ok(())
}
