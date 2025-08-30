#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol, 
    Error as SorobanError
};


mod reflector;
use reflector::{ReflectorClient, Asset as ReflectorAsset};

#[contract]
pub struct KaleLendingPlatform;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakingPosition {
    pub user: Address,
    pub kale_amount: i128,
    pub start_time: u64,
    pub last_claim_time: u64,
    pub auto_adjust_enabled: bool,
    pub price_threshold: i128, // e.g., 500 = 5%
    pub last_adjustment_price: i128,
    pub total_earned: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowingPosition {
    pub user: Address,
    pub borrowed_amount: i128,
    pub collateral_amount: i128,
    pub borrow_time: u64,
    pub interest_rate: i128, // in basis points
    pub last_payment_time: u64,
    pub total_interest_paid: i128,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformState {
    pub admin: Address,
    pub kale_token: Address,
    pub xlm_token: Address, // XLM token for collateral
    pub reflector_oracle: Address,
    pub total_staked: i128,
    pub total_borrowed: i128,
    pub total_collateral: i128,
    pub staking_apy: i128, // Annual yield rate in basis points
    pub borrowing_apy: i128, // Annual interest rate in basis points
    pub current_kale_price: i128,
    pub current_xlm_price: i128, // Current XLM price in USD
    pub last_price_update: u64,
    pub platform_fee_rate: i128, // Platform fee in basis points
    pub liquidation_threshold: i128, // Collateral ratio threshold
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YieldPool {
    pub total_rewards_distributed: i128,
    pub staking_rewards: i128,
    pub borrowing_fees: i128,
    pub platform_fees: i128,
    pub last_distribution_time: u64,
}

// Storage keys enum for better organization
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    PlatformState,
    StakingPositions,
    BorrowingPositions,
    YieldPool,
}

impl StorageKey {
    fn to_symbol(&self) -> Symbol {
        match self {
            StorageKey::PlatformState => symbol_short!("STATE"),
            StorageKey::StakingPositions => symbol_short!("STAKES"),
            StorageKey::BorrowingPositions => symbol_short!("BORROWS"),
            StorageKey::YieldPool => symbol_short!("YIELD"),
        }
    }

    // Helper method to get data from storage
    fn get<T>(&self, env: &Env) -> Option<T> 
    where 
        T: soroban_sdk::IntoVal<Env, soroban_sdk::Val> + soroban_sdk::FromVal<Env, soroban_sdk::Val>,
    {
        env.storage().instance().get(&self.to_symbol())
    }

    // Helper method to set data in storage
    fn set<T>(&self, env: &Env, value: &T) 
    where 
        T: soroban_sdk::IntoVal<Env, soroban_sdk::Val> + soroban_sdk::FromVal<Env, soroban_sdk::Val>,
    {
        env.storage().instance().set(&self.to_symbol(), value);
    }

    // Helper method to check if key exists in storage
    fn has(&self, env: &Env) -> bool {
        env.storage().instance().has(&self.to_symbol())
    }
}

// Testnet Reflector Oracle Addresses
const STELLAR_ORACLE: &str = "CAVLP5DH2GJPZMVO7IJY4CVOD5MWEFTJFVPD2YY2FQXOQHRGHK4D6HLP"; // Stellar Pubnet

#[contractimpl]
impl KaleLendingPlatform {
    // Initialize the KALE lending platform
    pub fn initialize(
        env: Env,
        admin: Address,
        kale_token: Address,
        xlm_token: Address, // XLM token for collateral
        reflector_oracle: Address,
        staking_apy: i128,
        borrowing_apy: i128,
        platform_fee_rate: i128,
        liquidation_threshold: i128,
    ) -> Result<(), SorobanError> {
        if StorageKey::PlatformState.has(&env) {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        let state = PlatformState {
            admin,
            kale_token,
            xlm_token,
            reflector_oracle,
            total_staked: 0,
            total_borrowed: 0,
            total_collateral: 0,
            staking_apy,
            borrowing_apy,
            current_kale_price: 0,
            current_xlm_price: 0,
            last_price_update: env.ledger().timestamp(),
            platform_fee_rate,
            liquidation_threshold,
            is_active: true,
        };

        let yield_pool = YieldPool {
            total_rewards_distributed: 0,
            staking_rewards: 0,
            borrowing_fees: 0,
            platform_fees: 0,
            last_distribution_time: env.ledger().timestamp(),
        };

        StorageKey::PlatformState.set(&env, &state);
        StorageKey::YieldPool.set(&env, &yield_pool);
        
        // Initialize empty maps
        let staking_positions: Map<Address, StakingPosition> = Map::new(&env);
        let borrowing_positions: Map<Address, BorrowingPosition> = Map::new(&env);
        StorageKey::StakingPositions.set(&env, &staking_positions);
        StorageKey::BorrowingPositions.set(&env, &borrowing_positions);
        
        Ok(())
    }

    // Stake KALE tokens to earn yield
    pub fn stake_kale(
        env: Env,
        user: Address,
        amount: i128,
        auto_adjust_enabled: bool,
        price_threshold_percent: u32,
    ) -> Result<(), SorobanError> {
        if amount <= 0 {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        let mut state: PlatformState = env.storage().instance().get(&StorageKey::PlatformState.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        if !state.is_active {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        let mut staking_positions: Map<Address, StakingPosition> = env.storage().instance().get(&StorageKey::StakingPositions.to_symbol())
            .unwrap_or(Map::new(&env));

        // Update current KALE price
        let current_price = Self::get_kale_price(&env, &state.reflector_oracle)?;
        state.current_kale_price = current_price;
        state.last_price_update = env.ledger().timestamp();

        // Create or update staking position
        let position = StakingPosition {
            user: user.clone(),
            kale_amount: amount,
            start_time: env.ledger().timestamp(),
            last_claim_time: env.ledger().timestamp(),
            auto_adjust_enabled,
            price_threshold: (price_threshold_percent as i128) * 100, // Convert to basis points
            last_adjustment_price: current_price,
            total_earned: 0,
        };

        staking_positions.set(user, position);
        state.total_staked += amount;

        env.storage().instance().set(&StorageKey::StakingPositions.to_symbol(), &staking_positions);
        env.storage().instance().set(&StorageKey::PlatformState.to_symbol(), &state);

        Ok(())
    }

    // Borrow KALE using XLM as collateral (proper DeFi lending)
    pub fn borrow_kale_with_xlm(
        env: Env,
        user: Address,
        xlm_collateral_amount: i128,
        kale_borrow_amount: i128,
    ) -> Result<(), SorobanError> {
        if xlm_collateral_amount <= 0 || kale_borrow_amount <= 0 {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        let mut state: PlatformState = env.storage().instance().get(&StorageKey::PlatformState.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        if !state.is_active {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        // Get current prices from Reflector oracle
        let kale_price_usd = Self::get_kale_price(&env, &state.reflector_oracle)?;
        let xlm_price_usd = Self::get_xlm_price(&env, &state.reflector_oracle)?;
        
        // Calculate collateral value in USD (assuming 6 decimals for both KALE and XLM)
        let collateral_value_usd = (xlm_collateral_amount * xlm_price_usd) / 1000000;
        let borrow_value_usd = (kale_borrow_amount * kale_price_usd) / 1000000;
        
        // Check collateral ratio (e.g., 150% = 15000 basis points)
        let collateral_ratio = (collateral_value_usd * 10000) / borrow_value_usd;

        if collateral_ratio < state.liquidation_threshold {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        let mut borrowing_positions: Map<Address, BorrowingPosition> = env.storage().instance().get(&StorageKey::BorrowingPositions.to_symbol())
            .unwrap_or(Map::new(&env));

        let position = BorrowingPosition {
            user: user.clone(),
            borrowed_amount: kale_borrow_amount,
            collateral_amount: xlm_collateral_amount,
            borrow_time: env.ledger().timestamp(),
            interest_rate: state.borrowing_apy,
            last_payment_time: env.ledger().timestamp(),
            total_interest_paid: 0,
            is_active: true,
        };

        borrowing_positions.set(user, position);
        state.total_borrowed += kale_borrow_amount;
        state.total_collateral += xlm_collateral_amount;
        state.current_kale_price = kale_price_usd;
        state.current_xlm_price = xlm_price_usd;
        state.last_price_update = env.ledger().timestamp();

        env.storage().instance().set(&StorageKey::BorrowingPositions.to_symbol(), &borrowing_positions);
        env.storage().instance().set(&StorageKey::PlatformState.to_symbol(), &state);

        Ok(())
    }

    // Repay borrowed KALE and release XLM collateral
    pub fn repay_borrowed_kale(
        env: Env,
        user: Address,
        repay_amount: i128,
    ) -> Result<i128, SorobanError> {
        if repay_amount <= 0 {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        let mut state: PlatformState = env.storage().instance().get(&StorageKey::PlatformState.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        let mut borrowing_positions: Map<Address, BorrowingPosition> = env.storage().instance().get(&StorageKey::BorrowingPositions.to_symbol())
            .unwrap_or(Map::new(&env));

        let mut position = borrowing_positions.get(user.clone())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        if !position.is_active {
            return Err(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ));
        }

        // Calculate interest owed
        let time_elapsed = (env.ledger().timestamp() - position.last_payment_time) as i128;
        let interest_owed = (position.borrowed_amount * position.interest_rate * time_elapsed) / (365 * 24 * 60 * 60 * 10000);
        
        let total_repay_needed = position.borrowed_amount + interest_owed;
        let actual_repay = if repay_amount >= total_repay_needed {
            total_repay_needed
        } else {
            repay_amount
        };

        // Update position
        position.borrowed_amount -= actual_repay;
        position.total_interest_paid += interest_owed;
        position.last_payment_time = env.ledger().timestamp();

        // If fully repaid, release XLM collateral
        if position.borrowed_amount <= 0 {
            position.is_active = false;
            state.total_collateral -= position.collateral_amount;
        }

        state.total_borrowed -= actual_repay;

        borrowing_positions.set(user, position);
        env.storage().instance().set(&StorageKey::BorrowingPositions.to_symbol(), &borrowing_positions);
        env.storage().instance().set(&StorageKey::PlatformState.to_symbol(), &state);

        Ok(actual_repay)
    }

    // Claim staking rewards
    pub fn claim_staking_rewards(env: Env, user: Address) -> Result<i128, SorobanError> {
        let state: PlatformState = env.storage().instance().get(&StorageKey::PlatformState.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        let staking_positions: Map<Address, StakingPosition> = env.storage().instance().get(&StorageKey::StakingPositions.to_symbol())
            .unwrap_or(Map::new(&env));

        let mut position = staking_positions.get(user.clone())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        // Calculate rewards based on time staked and amount
        let time_staked = (env.ledger().timestamp() - position.last_claim_time) as i128;
        let rewards = (position.kale_amount * state.staking_apy * time_staked) / (365 * 24 * 60 * 60 * 10000);

        // Update position
        position.last_claim_time = env.ledger().timestamp();
        position.total_earned += rewards;

        // Update storage
        let mut updated_positions = staking_positions;
        updated_positions.set(user, position);
        env.storage().instance().set(&StorageKey::StakingPositions.to_symbol(), &updated_positions);

        // Update yield pool
        let mut yield_pool: YieldPool = env.storage().instance().get(&StorageKey::YieldPool.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;
        
        yield_pool.staking_rewards += rewards;
        yield_pool.total_rewards_distributed += rewards;
        yield_pool.last_distribution_time = env.ledger().timestamp();
        env.storage().instance().set(&StorageKey::YieldPool.to_symbol(), &yield_pool);

        Ok(rewards)
    }

    // Check and adjust staking based on price movements
    pub fn check_price_adjustments(env: Env, user: Address) -> Result<bool, SorobanError> {
        let state: PlatformState = env.storage().instance().get(&StorageKey::PlatformState.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        let mut staking_positions: Map<Address, StakingPosition> = env.storage().instance().get(&StorageKey::StakingPositions.to_symbol())
            .unwrap_or(Map::new(&env));

        let mut position = staking_positions.get(user.clone())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        if !position.auto_adjust_enabled {
            return Ok(false);
        }

        let current_price = Self::get_kale_price(&env, &state.reflector_oracle)?;
        let price_change = if position.last_adjustment_price > 0 {
            ((current_price - position.last_adjustment_price) * 10000) / position.last_adjustment_price
        } else {
            0
        };

        // If price change exceeds threshold, adjust stake
        if price_change.abs() >= position.price_threshold {
            // Adjust stake based on price movement
            let adjustment_factor = if price_change > 0 {
                10000 + (price_change / 10) // Increase stake by 10% of price increase
            } else {
                10000 + (price_change / 10) // Decrease stake by 10% of price decrease
            };

            let new_amount = (position.kale_amount * adjustment_factor) / 10000;
            position.kale_amount = new_amount;
            position.last_adjustment_price = current_price;

            staking_positions.set(user, position);
            env.storage().instance().set(&StorageKey::StakingPositions.to_symbol(), &staking_positions);

            return Ok(true);
        }

        Ok(false)
    }

    // Get current KALE price from Reflector
    pub fn get_current_kale_price(env: Env) -> Result<i128, SorobanError> {
        let state: PlatformState = env.storage().instance().get(&StorageKey::PlatformState.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        Self::get_kale_price(&env, &state.reflector_oracle)
    }

    // Get user's staking position
    pub fn get_staking_position(env: Env, user: Address) -> Result<StakingPosition, SorobanError> {
        let staking_positions: Map<Address, StakingPosition> = env.storage().instance().get(&StorageKey::StakingPositions.to_symbol())
            .unwrap_or(Map::new(&env));

        staking_positions.get(user)
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))
    }

    // Get user's borrowing position
    pub fn get_borrowing_position(env: Env, user: Address) -> Result<BorrowingPosition, SorobanError> {
        let borrowing_positions: Map<Address, BorrowingPosition> = env.storage().instance().get(&StorageKey::BorrowingPositions.to_symbol())
            .unwrap_or(Map::new(&env));

        borrowing_positions.get(user)
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))
    }

    // Get platform state
    pub fn get_platform_state(env: Env) -> Result<PlatformState, SorobanError> {
        StorageKey::PlatformState.get(&env)
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))
    }

    // Get yield pool information
    pub fn get_yield_pool(env: Env) -> Result<YieldPool, SorobanError> {
        StorageKey::YieldPool.get(&env)
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))
    }

    // Admin function to update platform parameters
    pub fn update_platform_config(
        env: Env,
        staking_apy: Option<i128>,
        borrowing_apy: Option<i128>,
        platform_fee_rate: Option<i128>,
        liquidation_threshold: Option<i128>,
        is_active: Option<bool>,
    ) -> Result<(), SorobanError> {
        let mut state: PlatformState = env.storage().instance().get(&StorageKey::PlatformState.to_symbol())
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;

        // Check admin authorization (simplified for MVP)
        
        if let Some(apy) = staking_apy {
            state.staking_apy = apy;
        }
        if let Some(apy) = borrowing_apy {
            state.borrowing_apy = apy;
        }
        if let Some(fee) = platform_fee_rate {
            state.platform_fee_rate = fee;
        }
        if let Some(threshold) = liquidation_threshold {
            state.liquidation_threshold = threshold;
        }
        if let Some(active) = is_active {
            state.is_active = active;
        }

        env.storage().instance().set(&StorageKey::PlatformState.to_symbol(), &state);
        Ok(())
    }

    // Helper function to get KALE price from Reflector oracle
    fn get_kale_price(env: &Env, oracle_address: &Address) -> Result<i128, SorobanError> {
        let reflector_client = ReflectorClient::new(env, oracle_address);
        let kale_asset = ReflectorAsset::Other(symbol_short!("KALE"));
        
        let price_data = reflector_client.lastprice(&kale_asset)
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;
        
        Ok(price_data.price)
    }

    // Helper function to get XLM price from Reflector oracle
    fn get_xlm_price(env: &Env, oracle_address: &Address) -> Result<i128, SorobanError> {
        let reflector_client = ReflectorClient::new(env, oracle_address);
        let xlm_asset = ReflectorAsset::Other(symbol_short!("XLM"));
        
        let price_data = reflector_client.lastprice(&xlm_asset)
            .ok_or(SorobanError::from_type_and_code(
                soroban_sdk::xdr::ScErrorType::Context,
                soroban_sdk::xdr::ScErrorCode::InvalidInput,
            ))?;
        
        Ok(price_data.price)
    }
}

mod test;
