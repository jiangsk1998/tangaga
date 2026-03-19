use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::{self, Token2022}, token_interface::TokenAccount};

use crate::CustomError;




    /// 2. mint_to_wallet — 铸造代币到指定钱包的 ATA
    pub fn mint_to_wallet(ctx: Context<MintToWallet>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::ZeroAmount);

        token_2022::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token_2022::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.destination_ata.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!("铸造成功！{} 代币 → {}", amount, ctx.accounts.destination_ata.key());
        Ok(())
    }

#[derive(Accounts)]
pub struct MintToWallet<'info> {
    /// CHECK: Token-2022 mint（含 metadata extension，不能用 Account<Mint>）
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = destination_wallet,
        associated_token::token_program = token_program,
    )]
    pub destination_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: 任何公钥都可以接收代币
    pub destination_wallet: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}