use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::{self, Token2022}, token_interface::TokenAccount};

use crate::error::CustomError;


    /// 3. transfer_tokens — 从一个钱包转代币到另一个钱包
    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::ZeroAmount);

        // 从 mint 账户数据中读取 decimals
        use anchor_spl::token_2022::spl_token_2022::state::Mint as MintState;
        use anchor_spl::token_2022::spl_token_2022::extension::StateWithExtensions;
        let mint_data = ctx.accounts.mint.data.borrow();
        let mint_state = StateWithExtensions::<MintState>::unpack(&mint_data)?;
        let decimals = mint_state.base.decimals;
        drop(mint_data);

        token_2022::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token_2022::TransferChecked {
                    from: ctx.accounts.from_ata.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.to_ata.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount,
            decimals,
        )?;

        msg!(
            "转账成功！{} → {} (数量: {})",
            ctx.accounts.from_ata.key(),
            ctx.accounts.to_ata.key(),
            amount
        );
        Ok(())
    }

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    /// CHECK: Token-2022 mint
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub from_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = to_wallet,
        associated_token::token_program = token_program,
    )]
    pub to_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: 接收方钱包
    pub to_wallet: UncheckedAccount<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}