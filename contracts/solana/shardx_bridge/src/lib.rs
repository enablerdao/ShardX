//! ShardX Bridge Program for Solana
//!
//! This program enables cross-chain transfers between ShardX and Solana.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::{
    instruction as token_instruction,
    state::{Account as TokenAccount, Mint},
};
use spl_associated_token_account::instruction as associated_token_instruction;
use std::convert::TryInto;

/// プログラムのエントリーポイント
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// 命令処理
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 命令タイプを解析
    let instruction = BridgeInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    match instruction {
        BridgeInstruction::Initialize => process_initialize(program_id, accounts),
        BridgeInstruction::Deposit { amount, to_shardx_address } => {
            process_deposit(program_id, accounts, amount, to_shardx_address)
        },
        BridgeInstruction::Withdraw { amount, to_solana_address } => {
            process_withdraw(program_id, accounts, amount, to_solana_address)
        },
        BridgeInstruction::AddValidator { validator } => {
            process_add_validator(program_id, accounts, validator)
        },
        BridgeInstruction::RemoveValidator { validator } => {
            process_remove_validator(program_id, accounts, validator)
        },
        BridgeInstruction::AddSupportedToken { mint } => {
            process_add_supported_token(program_id, accounts, mint)
        },
        BridgeInstruction::RemoveSupportedToken { mint } => {
            process_remove_supported_token(program_id, accounts, mint)
        },
    }
}

/// ブリッジ命令
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum BridgeInstruction {
    /// ブリッジを初期化
    Initialize,
    /// トークンをデポジット
    Deposit {
        /// 金額
        amount: u64,
        /// ShardX宛先アドレス
        to_shardx_address: String,
    },
    /// トークンを引き出し
    Withdraw {
        /// 金額
        amount: u64,
        /// Solana宛先アドレス
        to_solana_address: Pubkey,
    },
    /// バリデータを追加
    AddValidator {
        /// バリデータのアドレス
        validator: Pubkey,
    },
    /// バリデータを削除
    RemoveValidator {
        /// バリデータのアドレス
        validator: Pubkey,
    },
    /// サポートするトークンを追加
    AddSupportedToken {
        /// トークンのミントアドレス
        mint: Pubkey,
    },
    /// サポートするトークンを削除
    RemoveSupportedToken {
        /// トークンのミントアドレス
        mint: Pubkey,
    },
}

/// ブリッジ設定
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct BridgeConfig {
    /// 初期化済みかどうか
    pub is_initialized: bool,
    /// 管理者
    pub admin: Pubkey,
    /// バリデータ数
    pub validator_count: u8,
    /// 必要な承認数
    pub required_confirmations: u8,
    /// サポートされているトークン数
    pub supported_token_count: u8,
}

/// バリデータ情報
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ValidatorInfo {
    /// バリデータのアドレス
    pub address: Pubkey,
    /// アクティブかどうか
    pub is_active: bool,
}

/// サポートされているトークン情報
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct SupportedTokenInfo {
    /// トークンのミントアドレス
    pub mint: Pubkey,
    /// アクティブかどうか
    pub is_active: bool,
}

/// ブリッジを初期化
fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // アカウント情報を取得
    let admin_info = next_account_info(account_info_iter)?;
    let bridge_config_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    
    // 署名を確認
    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // ブリッジ設定アカウントを作成
    let rent = Rent::get()?;
    let space = std::mem::size_of::<BridgeConfig>();
    let lamports = rent.minimum_balance(space);
    
    // ブリッジ設定アカウントを作成
    invoke(
        &system_instruction::create_account(
            admin_info.key,
            bridge_config_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            admin_info.clone(),
            bridge_config_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // ブリッジ設定を初期化
    let bridge_config = BridgeConfig {
        is_initialized: true,
        admin: *admin_info.key,
        validator_count: 0,
        required_confirmations: 1, // デフォルトは1
        supported_token_count: 0,
    };
    
    // ブリッジ設定を保存
    bridge_config.serialize(&mut *bridge_config_info.data.borrow_mut())?;
    
    msg!("Bridge initialized");
    
    Ok(())
}

/// トークンをデポジット
fn process_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    to_shardx_address: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // アカウント情報を取得
    let sender_info = next_account_info(account_info_iter)?;
    let sender_token_account_info = next_account_info(account_info_iter)?;
    let bridge_token_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    // 署名を確認
    if !sender_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // トークンがサポートされているか確認
    // TODO: サポートされているトークンのリストを確認
    
    // トークンを転送
    invoke(
        &token_instruction::transfer(
            token_program_info.key,
            sender_token_account_info.key,
            bridge_token_account_info.key,
            sender_info.key,
            &[sender_info.key],
            amount,
        )?,
        &[
            sender_token_account_info.clone(),
            bridge_token_account_info.clone(),
            sender_info.clone(),
            token_program_info.clone(),
        ],
    )?;
    
    // デポジットイベントをログに記録
    msg!("Deposit: {} tokens to ShardX address {}", amount, to_shardx_address);
    
    Ok(())
}

/// トークンを引き出し
fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    to_solana_address: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // アカウント情報を取得
    let validator_info = next_account_info(account_info_iter)?;
    let bridge_token_account_info = next_account_info(account_info_iter)?;
    let recipient_token_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let associated_token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    
    // 署名を確認
    if !validator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // バリデータが登録されているか確認
    // TODO: バリデータのリストを確認
    
    // 受取人のトークンアカウントが存在しない場合は作成
    if recipient_token_account_info.data_is_empty() {
        invoke(
            &associated_token_instruction::create_associated_token_account(
                validator_info.key,
                &to_solana_address,
                mint_info.key,
            ),
            &[
                validator_info.clone(),
                recipient_token_account_info.clone(),
                to_solana_address.clone(),
                mint_info.clone(),
                system_program_info.clone(),
                token_program_info.clone(),
                associated_token_program_info.clone(),
            ],
        )?;
    }
    
    // ブリッジのPDAを取得
    let (bridge_authority, bridge_authority_bump) = Pubkey::find_program_address(
        &[b"bridge_authority"],
        program_id,
    );
    
    // トークンを転送
    invoke_signed(
        &token_instruction::transfer(
            token_program_info.key,
            bridge_token_account_info.key,
            recipient_token_account_info.key,
            &bridge_authority,
            &[],
            amount,
        )?,
        &[
            bridge_token_account_info.clone(),
            recipient_token_account_info.clone(),
            token_program_info.clone(),
        ],
        &[&[b"bridge_authority", &[bridge_authority_bump]]],
    )?;
    
    // 引き出しイベントをログに記録
    msg!("Withdraw: {} tokens to Solana address {}", amount, to_solana_address);
    
    Ok(())
}

/// バリデータを追加
fn process_add_validator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    validator: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // アカウント情報を取得
    let admin_info = next_account_info(account_info_iter)?;
    let bridge_config_info = next_account_info(account_info_iter)?;
    let validator_list_info = next_account_info(account_info_iter)?;
    
    // 署名を確認
    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // ブリッジ設定を取得
    let mut bridge_config = BridgeConfig::try_from_slice(&bridge_config_info.data.borrow())?;
    
    // 管理者かどうか確認
    if bridge_config.admin != *admin_info.key {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // バリデータリストを取得
    let mut validator_list = Vec::<ValidatorInfo>::try_from_slice(&validator_list_info.data.borrow())?;
    
    // バリデータが既に存在するか確認
    for validator_info in &validator_list {
        if validator_info.address == validator && validator_info.is_active {
            return Err(ProgramError::InvalidArgument);
        }
    }
    
    // バリデータを追加
    validator_list.push(ValidatorInfo {
        address: validator,
        is_active: true,
    });
    
    // バリデータ数を更新
    bridge_config.validator_count = validator_list.len() as u8;
    
    // ブリッジ設定を保存
    bridge_config.serialize(&mut *bridge_config_info.data.borrow_mut())?;
    
    // バリデータリストを保存
    validator_list.serialize(&mut *validator_list_info.data.borrow_mut())?;
    
    msg!("Validator added: {}", validator);
    
    Ok(())
}

/// バリデータを削除
fn process_remove_validator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    validator: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // アカウント情報を取得
    let admin_info = next_account_info(account_info_iter)?;
    let bridge_config_info = next_account_info(account_info_iter)?;
    let validator_list_info = next_account_info(account_info_iter)?;
    
    // 署名を確認
    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // ブリッジ設定を取得
    let mut bridge_config = BridgeConfig::try_from_slice(&bridge_config_info.data.borrow())?;
    
    // 管理者かどうか確認
    if bridge_config.admin != *admin_info.key {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // バリデータリストを取得
    let mut validator_list = Vec::<ValidatorInfo>::try_from_slice(&validator_list_info.data.borrow())?;
    
    // バリデータを削除（非アクティブにする）
    let mut found = false;
    for validator_info in &mut validator_list {
        if validator_info.address == validator {
            validator_info.is_active = false;
            found = true;
            break;
        }
    }
    
    if !found {
        return Err(ProgramError::InvalidArgument);
    }
    
    // アクティブなバリデータ数を更新
    let active_count = validator_list.iter().filter(|v| v.is_active).count() as u8;
    bridge_config.validator_count = active_count;
    
    // 必要な承認数が多すぎる場合は調整
    if bridge_config.required_confirmations > active_count {
        bridge_config.required_confirmations = active_count;
    }
    
    // ブリッジ設定を保存
    bridge_config.serialize(&mut *bridge_config_info.data.borrow_mut())?;
    
    // バリデータリストを保存
    validator_list.serialize(&mut *validator_list_info.data.borrow_mut())?;
    
    msg!("Validator removed: {}", validator);
    
    Ok(())
}

/// サポートするトークンを追加
fn process_add_supported_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    mint: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // アカウント情報を取得
    let admin_info = next_account_info(account_info_iter)?;
    let bridge_config_info = next_account_info(account_info_iter)?;
    let token_list_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    
    // 署名を確認
    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // ブリッジ設定を取得
    let mut bridge_config = BridgeConfig::try_from_slice(&bridge_config_info.data.borrow())?;
    
    // 管理者かどうか確認
    if bridge_config.admin != *admin_info.key {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // ミントアドレスが有効かどうか確認
    if mint != *mint_info.key {
        return Err(ProgramError::InvalidArgument);
    }
    
    // トークンリストを取得
    let mut token_list = Vec::<SupportedTokenInfo>::try_from_slice(&token_list_info.data.borrow())?;
    
    // トークンが既に存在するか確認
    for token_info in &token_list {
        if token_info.mint == mint && token_info.is_active {
            return Err(ProgramError::InvalidArgument);
        }
    }
    
    // トークンを追加
    token_list.push(SupportedTokenInfo {
        mint,
        is_active: true,
    });
    
    // サポートされているトークン数を更新
    bridge_config.supported_token_count = token_list.len() as u8;
    
    // ブリッジ設定を保存
    bridge_config.serialize(&mut *bridge_config_info.data.borrow_mut())?;
    
    // トークンリストを保存
    token_list.serialize(&mut *token_list_info.data.borrow_mut())?;
    
    msg!("Supported token added: {}", mint);
    
    Ok(())
}

/// サポートするトークンを削除
fn process_remove_supported_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    mint: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // アカウント情報を取得
    let admin_info = next_account_info(account_info_iter)?;
    let bridge_config_info = next_account_info(account_info_iter)?;
    let token_list_info = next_account_info(account_info_iter)?;
    
    // 署名を確認
    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // ブリッジ設定を取得
    let mut bridge_config = BridgeConfig::try_from_slice(&bridge_config_info.data.borrow())?;
    
    // 管理者かどうか確認
    if bridge_config.admin != *admin_info.key {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // トークンリストを取得
    let mut token_list = Vec::<SupportedTokenInfo>::try_from_slice(&token_list_info.data.borrow())?;
    
    // トークンを削除（非アクティブにする）
    let mut found = false;
    for token_info in &mut token_list {
        if token_info.mint == mint {
            token_info.is_active = false;
            found = true;
            break;
        }
    }
    
    if !found {
        return Err(ProgramError::InvalidArgument);
    }
    
    // アクティブなトークン数を更新
    let active_count = token_list.iter().filter(|t| t.is_active).count() as u8;
    bridge_config.supported_token_count = active_count;
    
    // ブリッジ設定を保存
    bridge_config.serialize(&mut *bridge_config_info.data.borrow_mut())?;
    
    // トークンリストを保存
    token_list.serialize(&mut *token_list_info.data.borrow_mut())?;
    
    msg!("Supported token removed: {}", mint);
    
    Ok(())
}

/// エラー定義
#[derive(thiserror::Error, Debug)]
pub enum BridgeError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    #[error("Not initialized")]
    NotInitialized,
    
    #[error("Already initialized")]
    AlreadyInitialized,
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Invalid validator")]
    InvalidValidator,
    
    #[error("Insufficient confirmations")]
    InsufficientConfirmations,
}

impl From<BridgeError> for ProgramError {
    fn from(e: BridgeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}