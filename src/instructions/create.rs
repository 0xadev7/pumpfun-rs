//! Instruction for creating new tokens with bonding curves
//!
//! This module provides the functionality to create new tokens with associated bonding curves.
//! It includes the instruction data structure and helper function to build the Solana instruction.

use crate::{constants, PumpFun};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
use spl_associated_token_account::get_associated_token_address;

/// Instruction data for creating a new token
///
/// # Fields
///
/// * `name` - Name of the token to be created
/// * `symbol` - Symbol/ticker of the token to be created
/// * `uri` - Metadata URI containing token information (image, description, etc.)
/// * `creator` - Public key of the token creator
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Create {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
}

impl Create {
    /// Instruction discriminator used to identify this instruction
    pub const DISCRIMINATOR: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];

    /// Serializes the instruction data with the appropriate discriminator
    ///
    /// # Returns
    ///
    /// Byte vector containing the serialized instruction data
    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&Self::DISCRIMINATOR);
        self.serialize(&mut data).unwrap();
        data
    }
}

/// Creates an instruction to create a new token with bonding curve
///
/// Creates a new SPL token with an associated bonding curve that determines its price.
/// The token will have metadata and be tradable according to the bonding curve formula.
///
/// # Arguments
///
/// * `payer` - Keypair that will pay for account creation and transaction fees
/// * `mint` - Keypair for the new token mint account that will be created
/// * `args` - Create instruction data containing token name, symbol, metadata URI, and creator
///
/// # Returns
///
/// Returns a Solana instruction that when executed will create the token and its accounts
///
/// # Account Requirements
///
/// The instruction requires the following accounts in this order:
/// 1. Mint account (signer, writable)
/// 2. Mint authority PDA (readonly)
/// 3. Bonding curve PDA (writable)
/// 4. Bonding curve token account (writable)
/// 5. Global configuration PDA (readonly)
/// 6. MPL Token Metadata program (readonly)
/// 7. Metadata PDA (writable)
/// 8. Payer account (signer, writable)
/// 9. System program (readonly)
/// 10. Token program (readonly)
/// 11. Associated token program (readonly)
/// 12. Rent sysvar (readonly)
/// 13. Event authority (readonly)
/// 14. Pump.fun program ID (readonly)
pub fn create(payer: &Keypair, mint: &Keypair, args: Create) -> Instruction {
    let bonding_curve: Pubkey = PumpFun::get_bonding_curve_pda(&mint.pubkey()).unwrap();
    Instruction::new_with_bytes(
        constants::accounts::PUMPFUN,
        &args.data(),
        vec![
            AccountMeta::new(mint.pubkey(), true),
            AccountMeta::new(PumpFun::get_mint_authority_pda(), false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(
                get_associated_token_address(&bonding_curve, &mint.pubkey()),
                false,
            ),
            AccountMeta::new_readonly(PumpFun::get_global_pda(), false),
            AccountMeta::new_readonly(constants::accounts::MPL_TOKEN_METADATA, false),
            AccountMeta::new(PumpFun::get_metadata_pda(&mint.pubkey()), false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(constants::accounts::SYSTEM_PROGRAM, false),
            AccountMeta::new_readonly(constants::accounts::TOKEN_PROGRAM, false),
            AccountMeta::new_readonly(constants::accounts::ASSOCIATED_TOKEN_PROGRAM, false),
            AccountMeta::new_readonly(constants::accounts::RENT, false),
            AccountMeta::new_readonly(constants::accounts::EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(constants::accounts::PUMPFUN, false),
        ],
    )
}

/// Instruction data for creating a new token with Token 2022 (create_v2)
///
/// # Fields
///
/// * `name` - Name of the token to be created
/// * `symbol` - Symbol/ticker of the token to be created
/// * `uri` - Metadata URI containing token information (image, description, etc.)
/// * `creator` - Public key of the token creator
/// * `is_mayhem_mode` - Whether to enable mayhem mode for this token
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct CreateV2 {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
    pub is_mayhem_mode: bool,
}

impl CreateV2 {
    /// Instruction discriminator used to identify this instruction
    pub const DISCRIMINATOR: [u8; 8] = [214, 144, 76, 236, 95, 139, 49, 180];

    /// Serializes the instruction data with the appropriate discriminator
    ///
    /// # Returns
    ///
    /// Byte vector containing the serialized instruction data
    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&Self::DISCRIMINATOR);
        self.serialize(&mut data).unwrap();
        data
    }
}

/// Creates an instruction to create a new Token 2022 token with bonding curve (create_v2)
///
/// Creates a new SPL Token 2022 token with an associated bonding curve that determines its price.
/// This version uses Token 2022 program instead of the standard Token program and supports
/// mayhem mode functionality.
///
/// # Arguments
///
/// * `payer` - Keypair that will pay for account creation and transaction fees
/// * `mint` - Keypair for the new token mint account that will be created
/// * `args` - CreateV2 instruction data containing token name, symbol, metadata URI, creator, and mayhem mode flag
///
/// # Returns
///
/// Returns a Solana instruction that when executed will create the Token 2022 token and its accounts
///
/// # Account Requirements
///
/// The instruction requires the following accounts in this order:
/// 1. Mint account (signer, writable)
/// 2. Mint authority PDA (readonly)
/// 3. Bonding curve PDA (writable)
/// 4. Bonding curve token account (writable)
/// 5. Global configuration PDA (readonly)
/// 6. Payer account (signer, writable)
/// 7. System program (readonly)
/// 8. Token 2022 program (readonly)
/// 9. Associated token program (readonly)
/// 10. Mayhem program ID (writable)
/// 11. Global params PDA (readonly)
/// 12. SOL vault PDA (writable)
/// 13. Mayhem state PDA (writable)
/// 14. Mayhem token vault (writable)
/// 15. Event authority (readonly)
/// 16. Pump.fun program ID (readonly)
pub fn create_v2(payer: &Keypair, mint: &Keypair, args: CreateV2) -> Instruction {
    let bonding_curve: Pubkey = PumpFun::get_bonding_curve_pda(&mint.pubkey()).unwrap();
    let mayhem_program = constants::accounts::MAYHEM_PROGRAM;
    let global_params = PumpFun::get_global_params_pda();
    let sol_vault = PumpFun::get_sol_vault_pda();
    let mayhem_state = PumpFun::get_mayhem_state_pda(&mint.pubkey());
    let mayhem_token_vault = PumpFun::get_token_vault_pda(&mint.pubkey());

    // Derive associated_bonding_curve PDA with Token 2022 program ID
    // The PDA seeds are: [bonding_curve, token_program, mint]
    // For create_v2, we must use TOKEN_2022_PROGRAM instead of TOKEN_PROGRAM
    let associated_bonding_curve = PumpFun::get_associated_token_address_with_program(
        &bonding_curve,
        &mint.pubkey(),
        &constants::accounts::TOKEN_2022_PROGRAM,
    );

    Instruction::new_with_bytes(
        constants::accounts::PUMPFUN,
        &args.data(),
        vec![
            AccountMeta::new(mint.pubkey(), true),
            AccountMeta::new(PumpFun::get_mint_authority_pda(), false),
            AccountMeta::new(bonding_curve, false), // writable in IDL, but AccountMeta::new already makes it writable
            AccountMeta::new(
                associated_bonding_curve,
                false, // writable in IDL, but AccountMeta::new already makes it writable
            ),
            AccountMeta::new_readonly(PumpFun::get_global_pda(), false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(constants::accounts::SYSTEM_PROGRAM, false),
            AccountMeta::new_readonly(constants::accounts::TOKEN_2022_PROGRAM, false),
            AccountMeta::new_readonly(constants::accounts::ASSOCIATED_TOKEN_PROGRAM, false),
            AccountMeta::new(mayhem_program, false), // writable in IDL, but AccountMeta::new already makes it writable
            AccountMeta::new_readonly(global_params, false),
            AccountMeta::new(sol_vault, false), // writable in IDL, but AccountMeta::new already makes it writable
            AccountMeta::new(mayhem_state, false), // writable in IDL, but AccountMeta::new already makes it writable
            AccountMeta::new(mayhem_token_vault, false), // writable in IDL, but AccountMeta::new already makes it writable
            AccountMeta::new_readonly(constants::accounts::EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(constants::accounts::PUMPFUN, false),
        ],
    )
}