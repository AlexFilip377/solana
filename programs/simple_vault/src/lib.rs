use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("BLMkvADf7fefKXUjYf5buSnUXuwJJhMVq5LJeYS6HfxT"); // Подставьте ваш ключ из anchor keys list

#[program]
pub mod simple_vault {
    use super::*;

    /// Создать vault для конкретного SPL-токена.
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault_state;
        vault.admin = ctx.accounts.admin.key();
        vault.token_mint = ctx.accounts.token_mint.key();
        vault.vault_token_account = ctx.accounts.vault_token_account.key();
        vault.share_mint = ctx.accounts.share_mint.key();
        vault.paused = false; // [Задание 3]: Изначально vault активен (не на паузе)
        vault.bump = ctx.bumps.vault_state;

        msg!("Vault initialized for mint: {}", vault.token_mint);
        Ok(())
    }

    /// Создать личную «карточку» пользователя в vault.
    pub fn create_user_card(ctx: Context<CreateUserCard>) -> Result<()> {
        // [Задание 3]: Проверяем, что vault не поставлен на паузу
        require!(!ctx.accounts.vault_state.paused, VaultError::VaultPaused);

        let card = &mut ctx.accounts.user_card;
        card.vault = ctx.accounts.vault_state.key();
        card.owner = ctx.accounts.user.key();
        // [Задание 1]: Сохраняем unix timestamp создания карточки
        card.created_at = Clock::get()?.unix_timestamp;
        card.bump = ctx.bumps.user_card;

        msg!("User card created for: {} at timestamp {}", card.owner, card.created_at);
        Ok(())
    }

    /// [Задание 2]: Инструкция для закрытия карточки пользователя и возврата ренты
    pub fn close_user_card(_ctx: Context<CloseUserCard>) -> Result<()> {
        msg!("User card successfully closed and rent refunded.");
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Accounts
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = admin,
        space = VaultState::LEN,
        seeds = [b"vault", token_mint.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = admin,
        token::mint = token_mint,
        token::authority = vault_state,
        seeds = [b"vault_tokens", token_mint.key().as_ref()],
        bump,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = vault_state,
        seeds = [b"share_mint", token_mint.key().as_ref()],
        bump,
    )]
    pub share_mint: Account<'info, Mint>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateUserCard<'info> {
    #[account(
        seeds = [b"vault", vault_state.token_mint.as_ref()],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = user,
        space = UserCard::LEN,
        seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_card: Account<'info, UserCard>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// [Задание 2]: Контекст для закрытия карточки
#[derive(Accounts)]
pub struct CloseUserCard<'info> {
    /// `close = user` указывает Anchor вернуть всю замороженную ренту (SOL) на аккаунт user
    #[account(
        mut,
        close = user,
        seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()],
        bump = user_card.bump,
        constraint = user_card.owner == user.key(), // <-- Заменили has_one на явную проверку
    )]
    pub user_card: Account<'info, UserCard>,

    pub vault_state: Account<'info, VaultState>,

    #[account(mut)]
    pub user: Signer<'info>,
}

// ─────────────────────────────────────────────────────────────────────────────
// State & Errors
// ─────────────────────────────────────────────────────────────────────────────

#[account]
#[derive(Default)]
pub struct VaultState {
    pub admin: Pubkey,               // 32
    pub token_mint: Pubkey,          // 32
    pub vault_token_account: Pubkey, // 32
    pub share_mint: Pubkey,          // 32
    pub paused: bool,                // 1  — [Задание 3]: флаг паузы
    pub bump: u8,                    // 1
}

impl VaultState {
    // 8 (discriminator) + 32 + 32 + 32 + 32 + 1 + 1 = 138 байт
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 1 + 1;
}

#[account]
#[derive(Default)]
pub struct UserCard {
    pub vault: Pubkey,      // 32
    pub owner: Pubkey,      // 32
    pub created_at: i64,    // 8  — [Задание 1]: unix timestamp
    pub bump: u8,           // 1
}

impl UserCard {
    // 8 (discriminator) + 32 + 32 + 8 + 1 = 81 байт
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1;
}

// [Задание 3]: Кастомная ошибка при попытке создать карточку на паузе
#[error_code]
pub enum VaultError {
    #[msg("Vault is currently paused.")]
    VaultPaused,
}