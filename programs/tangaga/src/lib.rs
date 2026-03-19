use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{self, Token2022},
    token_2022_extensions::{
        metadata_pointer::{self, MetadataPointerInitialize},
        token_metadata::{self, TokenMetadataInitialize},
    },
    token_interface::TokenAccount,
};

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
}

// ── 账户结构体 ──────────────────────────────────────────────────────────────

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
