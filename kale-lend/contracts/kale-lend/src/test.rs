#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short, Address, Env, Symbol, symbol_short
};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500, // 5% staking APY
        &800, // 8% borrowing APY
        &100, // 1% platform fee
        &15000, // 150% liquidation threshold
    );
    
    // Verify platform state
    let state = KaleLendingPlatformClient::new(&env, &contract_id).get_platform_state().unwrap();
    assert_eq!(state.admin, admin);
    assert_eq!(state.kale_token, kale_token);
    assert_eq!(state.xlm_token, xlm_token);
    assert_eq!(state.reflector_oracle, reflector_oracle);
    assert_eq!(state.staking_apy, 500);
    assert_eq!(state.borrowing_apy, 800);
    assert_eq!(state.platform_fee_rate, 100);
    assert_eq!(state.liquidation_threshold, 15000);
    assert_eq!(state.is_active, true);
}

#[test]
fn test_stake_kale() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    let user = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500,
        &800,
        &100,
        &15000,
    );
    
    // Stake KALE
    KaleLendingPlatformClient::new(&env, &contract_id).stake_kale(
        &user,
        &1000000, // 1 KALE (6 decimals)
        &true, // auto adjust enabled
        &10, // 10% price threshold
    );
    
    // Verify staking position
    let position = KaleLendingPlatformClient::new(&env, &contract_id).get_staking_position(&user).unwrap();
    assert_eq!(position.user, user);
    assert_eq!(position.kale_amount, 1000000);
    assert_eq!(position.auto_adjust_enabled, true);
    assert_eq!(position.price_threshold_percent, 10);
    assert_eq!(position.is_active, true);
}

#[test]
fn test_borrow_kale_with_xlm() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    let user = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500,
        &800,
        &100,
        &15000,
    );
    
    // Borrow KALE using XLM as collateral
    // Assuming XLM price is $0.10 and KALE price is $1.00
    // 1500 XLM collateral (worth $150) for 100 KALE borrow (worth $100)
    KaleLendingPlatformClient::new(&env, &contract_id).borrow_kale_with_xlm(
        &user,
        &1500000, // 1500 XLM collateral (6 decimals)
        &100000, // 100 KALE borrow (6 decimals)
    );
    
    // Verify borrowing position
    let position = KaleLendingPlatformClient::new(&env, &contract_id).get_borrowing_position(&user).unwrap();
    assert_eq!(position.user, user);
    assert_eq!(position.borrowed_amount, 100000);
    assert_eq!(position.collateral_amount, 1500000);
    assert_eq!(position.is_active, true);
}

#[test]
fn test_claim_staking_rewards() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    let user = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500,
        &800,
        &100,
        &15000,
    );
    
    // Stake KALE first
    KaleLendingPlatformClient::new(&env, &contract_id).stake_kale(
        &user,
        &1000000,
        &true,
        &10,
    );
    
    // Claim rewards (will be 0 in test environment due to no time passage)
    let rewards = KaleLendingPlatformClient::new(&env, &contract_id).claim_staking_rewards(&user).unwrap();
    assert_eq!(rewards, 0);
}

#[test]
fn test_repay_borrowed_kale() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    let user = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500,
        &800,
        &100,
        &15000,
    );
    
    // Borrow KALE using XLM first
    KaleLendingPlatformClient::new(&env, &contract_id).borrow_kale_with_xlm(
        &user,
        &1500000,
        &100000,
    );
    
    // Repay borrowed KALE
    let repaid = KaleLendingPlatformClient::new(&env, &contract_id).repay_borrowed_kale(
        &user,
        &100000, // Full repayment
    ).unwrap();
    
    assert_eq!(repaid, 100000);
    
    // Verify position is closed
    let position = KaleLendingPlatformClient::new(&env, &contract_id).get_borrowing_position(&user).unwrap();
    assert_eq!(position.is_active, false);
}

#[test]
fn test_check_price_adjustments() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    let user = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500,
        &800,
        &100,
        &15000,
    );
    
    // Stake KALE with auto-adjust enabled
    KaleLendingPlatformClient::new(&env, &contract_id).stake_kale(
        &user,
        &1000000,
        &true,
        &10,
    );
    
    // Check price adjustments (will fail in test environment due to mock Reflector)
    let result = KaleLendingPlatformClient::new(&env, &contract_id).check_price_adjustments(&user);
    assert!(result.is_err()); // Expected to fail because we can't reach Reflector oracle in tests
}

#[test]
fn test_get_current_kale_price() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500,
        &800,
        &100,
        &15000,
    );
    
    // Get current KALE price (will fail in test environment due to mock Reflector)
    let result = KaleLendingPlatformClient::new(&env, &contract_id).get_current_kale_price();
    assert!(result.is_err()); // Expected to fail because we can't reach Reflector oracle in tests
}

#[test]
fn test_update_platform_config() {
    let env = Env::default();
    let contract_id = env.register_contract(None, KaleLendingPlatform);
    let admin = Address::generate(&env);
    let kale_token = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    let reflector_oracle = Address::generate(&env);
    
    KaleLendingPlatformClient::new(&env, &contract_id).initialize(
        &admin,
        &kale_token,
        &xlm_token,
        &reflector_oracle,
        &500,
        &800,
        &100,
        &15000,
    );
    
    // Update platform configuration
    KaleLendingPlatformClient::new(&env, &contract_id).update_platform_config(
        &Some(600), // New staking APY: 6%
        &Some(900), // New borrowing APY: 9%
        &Some(150), // New platform fee: 1.5%
        &Some(16000), // New liquidation threshold: 160%
        &Some(false), // Deactivate platform
    );
    
    // Verify updated configuration
    let state = KaleLendingPlatformClient::new(&env, &contract_id).get_platform_state().unwrap();
    assert_eq!(state.staking_apy, 600);
    assert_eq!(state.borrowing_apy, 900);
    assert_eq!(state.platform_fee_rate, 150);
    assert_eq!(state.liquidation_threshold, 16000);
    assert_eq!(state.is_active, false);
}
