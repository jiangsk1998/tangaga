use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_2022;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::MetadataPointerInitialize;
use anchor_spl::token_interface::TokenMetadataInitialize;
use anchor_spl::token_interface::metadata_pointer;
use anchor_spl::token_interface::token_metadata;
use crate::CustomError;


    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
        decimals: u8,
    ) -> Result<()> {
        require!(name.len() <= 32, CustomError::NameTooLong);
        require!(symbol.len() <= 10, CustomError::SymbolTooLong);
        require!(uri.len() <= 200, CustomError::UriTooLong);

        // ── 步骤 1：计算账户空间和 lamports ────────────────────────────
        // create_account 大小 = mint + MetadataPointer（InitializeMint2 要求精确匹配）
        // lamports = mint + MetadataPointer + Metadata 的 rent（预存，供 realloc 使用）
        use anchor_spl::token_2022::spl_token_2022::{
            extension::ExtensionType,
            state::Mint as MintState,
        };
        use anchor_spl::token_2022_extensions::spl_token_metadata_interface::state::TokenMetadata;

        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            ..Default::default()
        };

        let base_mint_size =
            ExtensionType::try_calculate_account_len::<MintState>(&[
                ExtensionType::MetadataPointer,
            ])
            .unwrap();

        let metadata_size = token_metadata.tlv_size_of().unwrap();
        let full_size = base_mint_size.checked_add(metadata_size).unwrap();

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(full_size);
        let mint_size = base_mint_size;

        // ── 步骤 2：创建账户 ────────────────────────────────────────────
        system_program::create_account(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::CreateAccount {
                    from: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                },
            ),
            lamports,
            mint_size as u64,
            &Token2022::id(),
        )?;

        // ── 步骤 3：初始化 MetadataPointer extension ────────────────────
        metadata_pointer::metadata_pointer_initialize(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MetadataPointerInitialize {
                    token_program_id: ctx.accounts.token_program.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            Some(ctx.accounts.authority.key()),
            Some(ctx.accounts.mint.key()),
        )?;

        // ── 步骤 4：初始化 Mint ─────────────────────────────────────────
        token_2022::initialize_mint2(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token_2022::InitializeMint2 {
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            decimals,
            &ctx.accounts.authority.key(),
            None,
        )?;

        // ── 步骤 5：初始化 Metadata extension ──────────────────────────
        token_metadata::token_metadata_initialize(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TokenMetadataInitialize {
                    program_id: ctx.accounts.token_program.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    metadata: ctx.accounts.mint.to_account_info(), // self-referential
                    mint_authority: ctx.accounts.authority.to_account_info(),
                    update_authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            name,
            symbol,
            uri,
        )?;

        msg!("Token-2022 Mint 创建成功: {}", ctx.accounts.mint.key());
        Ok(())
    }


#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String, decimals: u8)]
pub struct CreateToken<'info> {
    /// CHECK: 在指令逻辑中手动 create_account + initialize
    #[account(mut)]
    pub mint: Signer<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}