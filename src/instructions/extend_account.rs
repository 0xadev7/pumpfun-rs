//! Instruction for extending program-owned accounts
//!
//! This module provides the functionality to extend the size of program-owned accounts.
//! It includes the instruction data structure and helper function to build the Solana instruction.

use crate::constants;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};

/// Creates an instruction to extend a program-owned account
///
/// Extends the size of a program-owned account. This is typically used to increase
/// the size of bonding curve accounts when they need to store additional data.
///
/// # Arguments
///
/// * `payer` - Keypair that will pay for the account extension
/// * `account` - Public key of the account to extend
///
/// # Returns
///
/// Returns a Solana instruction that when executed will extend the account
///
/// # Account Requirements
///
/// The instruction requires the following accounts in this order:
/// 1. Account to extend (writable)
/// 2. Payer account (signer)
/// 3. System program (readonly)
/// 4. Event authority (readonly)
/// 5. Pump.fun program ID (readonly)
pub fn extend_account(payer: &Keypair, account: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        constants::accounts::PUMPFUN,
        &[234, 102, 194, 203, 150, 72, 62, 229], // extend_account discriminator
        vec![
            AccountMeta::new(*account, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(constants::accounts::SYSTEM_PROGRAM, false),
            AccountMeta::new_readonly(constants::accounts::EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(constants::accounts::PUMPFUN, false),
        ],
    )
}

