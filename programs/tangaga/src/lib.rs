use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account::error::AssociatedTokenAccountError;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::ID as METADATA_PROGRAM_ID;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("HxN1Z596K82cxETveoBQ8h8mQXxaVDmKwzgXR68Ykkbt");

#[program]
pub mod tangaga {
    use super::*;
    use anchor_lang::solana_program::program::invoke;
    use anchor_spl::metadata::mpl_token_metadata::instructions::{
        CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs,
    };
    use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
    use anchor_spl::token;
    use anchor_spl::token::MintTo;

    /// 我们的合约需要实现 3 个指令：
    ///
    /// 1. create_token    — 创建 Mint Account + Metadata Account
    /// → 一个交易同时完成"定义代币 + 添加元信息"
    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
        decimals: u8,
    ) -> Result<()> {
        // ========== 参数校验 ==========
        // Metaplex 对字段长度有限制，超出会导致交易失败
        require!(name.len() <= 32, CustomError::NameTooLong);
        require!(symbol.len() <= 10, CustomError::SymbolTooLong);
        require!(uri.len() <= 200, CustomError::UriTooLong);

        // ========== 步骤 1：Mint Account 已由 Anchor 自动创建 ==========
        // #[account(init, ...)] 约束在 CreateToken struct 中
        // → Anchor 在执行这个函数之前，已经完成了：
        //   1. 创建新账户（system_program.create_account）
        //   2. 初始化为 Mint（token_program.initialize_mint）
        // → 到这里，Mint Account 已经存在且已初始化
        msg!("Mint Account 创建成功: {}", ctx.accounts.mint.key());

        let mint_key = ctx.accounts.mint.key();

        let metadata_seeds = &[b"metadata", METADATA_PROGRAM_ID.as_ref(), mint_key.as_ref()];

        let (meta_pda, _bump) = Pubkey::find_program_address(
            metadata_seeds,
            &Pubkey::new_from_array(METADATA_PROGRAM_ID.to_bytes()),
        );

        require_keys_eq!(
            ctx.accounts.metadata.key(),
            meta_pda,
            CustomError::InvalidMetadata
        );

        //// 构建 Metaplex 的 CreateMetadataAccountV3 指令
        let metadata_account_v3 = CreateMetadataAccountV3 {
            metadata: ctx.accounts.metadata.key(),
            mint: ctx.accounts.mint.key(),
            mint_authority: ctx.accounts.authority.key(),
            payer: ctx.accounts.authority.key(),
            update_authority: (ctx.accounts.authority.key(), true),
            system_program: ctx.accounts.system_program.key(),
            rent: Some(ctx.accounts.rent.key()),
        };

        //指令数据
        let data_v2 = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0, //普通代币设为0 NFT才需要
            creators: None,
            collection: None,
            uses: None,
        };

        //完整指令
        let instruction = metadata_account_v3.instruction(CreateMetadataAccountV3InstructionArgs {
            data: data_v2,
            is_mutable: true,
            collection_details: None,
        });
        let account_infos = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        invoke(&instruction, &account_infos)?;
        msg!("Metadata Account 创建成功");

        msg!("代币创建完成！Mint:{},", ctx.accounts.mint.key());

        Ok(())
    }

    ///
    /// 2. mint_to_wallet  — 铸造代币到指定钱包的 ATA
    /// → ATA 不存在则自动创建（init_if_needed）
    pub fn mint_to_wallet(ctx: Context<MintToWallet>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::ZeroAmount);
        // ========== ATA 已由 Anchor 自动创建（init_if_needed） ==========
        // 如果目标钱包还没有这个代币的 ATA，Anchor 会自动创建
        // 如果已经有了，就跳过创建步骤

        // ========== CPI 调用 Token Program 铸造代币 ==========

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.destination_ata.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_context =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        token::mint_to(cpi_context, amount)?; //

        msg!(
            "铸造成功！{} 代币 → {}",
            amount,
            ctx.accounts.destination_ata.key()
        );

        Ok(())
    }
    ///
    /// 3. transfer_tokens — 从一个钱包转代币到另一个钱包
    /// → 演示代币转账的完整流程

    pub fn transfer_tokens(ctx: Context<TransferTokens>,amount:u64) -> Result<()> {

        require!(amount > 0, CustomError::ZeroAmount);

        let trans_accounts = anchor_spl::token::Transfer{
            from: ctx.accounts.from_ata.to_account_info(),
            to: ctx.accounts.to_ata.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), trans_accounts);

        token::transfer(cpi_context,amount)?;

        msg!(
            "转账成功！{} → {} (数量: {})",
            ctx.accounts.from_ata.key(),
            ctx.accounts.to_ata.key(),
            amount
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {

    pub mint: Account<'info, Mint>,

    /// 目标 ATA — 代币铸造到这里
    /// init_if_needed: 如果 ATA 不存在就自动创建
    /// associated_token::mint: 关联到哪个 Mint
    /// associated_token::authority: ATA 归谁所有
    #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = owner
    )]
    pub from_ata: Account<'info, TokenAccount>,

    #[account(
    init_if_needed,
    payer=owner,
    associated_token::mint = mint,
    associated_token::authority =to_wallet
    )]
    pub to_ata: Account<'info, TokenAccount>,

    /// 目标钱包地址 — ATA 的 owner（不需要签名，因为铸币不需要接收方同意）
    ///CHECK: 任何公钥都可以接收代币
    pub to_wallet: UncheckedAccount<'info>,

    pub system_program:Program<'info,System>,

    pub associated_token_program:Program<'info,AssociatedToken>,

    ///发送方钱包
    #[account(mut)]
    pub owner:Signer<'info>,


    pub token_program: Program<'info, Token>,

}

#[derive(Accounts)]
pub struct MintToWallet<'info> {
    #[account(mut)] //supply会增加
    pub mint: Account<'info, Mint>,

    /// 目标 ATA — 代币铸造到这里
    /// init_if_needed: 如果 ATA 不存在就自动创建
    /// associated_token::mint: 关联到哪个 Mint
    /// associated_token::authority: ATA 归谁所有
    #[account(
    init_if_needed,
    payer=authority,
    associated_token::mint = mint,
    associated_token::authority = destination_wallet
    )]
    pub destination_ata: Account<'info, TokenAccount>,

    /// 目标钱包地址 — ATA 的 owner（不需要签名，因为铸币不需要接收方同意）
    ///CHECK: 任何公钥都可以接收代币
    pub destination_wallet: UncheckedAccount<'info>,

    /// mint_authority — 必须是 Mint 的 mint_authority（签名者）
    #[account(mut,
        constraint=authority.key()==mint.mint_authority.unwrap() @  CustomError::UnauthorizedMinter
    )]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    ///if_need需要
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(name: String,
        symbol: String,
        uri: String,
        decimals: u8,)]
pub struct CreateToken<'info> {
    #[account(
        init,
        payer=authority,
        mint::decimals=decimals,
        mint::authority = authority.key()
    )]
    pub mint: Account<'info, Mint>,

    /// Metadata Account — 由 Metaplex 管理的 PDA
    /// 我们不用 Anchor 的 init（因为 Metaplex 自己创建），所以用 UncheckedAccount
    /// CHECK: 地址在指令逻辑中通过 PDA 推导验证
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,

    /// Metaplex Token Metadata Program
    /// CHECK: 我们验证它的地址等于 Metaplex 程序 ID
    #[account(
        constraint=token_metadata_program.key() == METADATA_PROGRAM_ID @ CustomError::InvalidMetadataProgram
    )]
    pub token_metadata_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Initialize {}

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

    #[msg("余额不足")]
    InsufficientBalance,

    #[msg("不是授权的铸币者")]
    UnauthorizedMinter,

    #[msg("Metadata 账户地址不匹配")]
    InvalidMetadata,

    #[msg("Token Metadata 程序地址不正确")]
    InvalidMetadataProgram,
}
