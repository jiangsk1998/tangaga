use anchor_lang::prelude::{program_option::COption, *};
use anchor_spl::{
    token::Mint,
    token_2022::{self, Token2022}, token_interface::TokenAccount,
};

use crate::error::CustomError;

pub fn handle(ctx: Context<DelegateTransfer>, amount: u64, decimals: u8) -> Result<()> {
    require!(amount > 0, CustomError::ZeroAmount);
    require!(
        ctx.accounts.from_ata.amount >= amount,
        CustomError::InsufficientFunds
    );

    let ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token_2022::TransferChecked {
            from: ctx.accounts.from_ata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to_ata.to_account_info(),
            authority: ctx.accounts.delegate.to_account_info(),
        },
    );

    token_2022::transfer_checked(ctx, amount, decimals)?;

    msg!("委托转账成功: {} 单位", amount);

    Ok(())
}

#[derive(Accounts)]
pub struct DelegateTransfer<'info> {
    #[account(mut)]
    pub delegate: Signer<'info>,

    #[account(mut,
        //校验是否是被授权人
        constraint=from_ata.delegate==COption::Some(delegate.key()) @ CustomError::NotOwnerOfToken
    )]
    pub from_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = delegate,
        associated_token::mint = mint,
        associated_token::authority = to_owner,
        associated_token::token_program = token_program,
    )]
    pub to_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Token-2022 mint
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    pub to_owner: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,

    pub token_program: Program<'info, Token2022>,
}
