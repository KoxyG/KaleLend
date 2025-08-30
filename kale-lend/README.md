# KALE Lending Platform

A comprehensive DeFi lending platform built on Stellar that enables users to stake KALE tokens for yield and borrow KALE using KALE as collateral, with real-time price monitoring via Reflector oracle.

## 🚀 Features

### **Core Functionality**
- **KALE Staking** - Stake KALE tokens to earn yield with configurable APY
- **KALE Borrowing** - Borrow KALE using KALE as collateral with interest rates
- **Price-Based Adjustments** - Automatically adjust staking based on KALE price movements
- **Real-time Price Monitoring** - Uses Reflector oracle for accurate KALE pricing
- **Yield Distribution** - Automated reward distribution and interest calculations
- **Collateral Management** - Liquidation protection and collateral ratio monitoring

### **Smart Contract Functions**

#### **Platform Management**
- `initialize()` - Set up platform with admin, KALE token, Reflector oracle, and rates
- `update_platform_config()` - Modify APY rates, fees, and platform parameters
- `get_platform_state()` - Retrieve current platform configuration and statistics

#### **Staking Operations**
- `stake_kale()` - Stake KALE tokens with auto-adjustment settings
- `claim_staking_rewards()` - Claim accumulated staking rewards
- `check_price_adjustments()` - Monitor and adjust stakes based on price movements
- `get_staking_position()` - View user's staking position and earnings

#### **Borrowing Operations**
- `borrow_kale()` - Borrow KALE using KALE as collateral
- `repay_borrowed_kale()` - Repay borrowed KALE with interest
- `get_borrowing_position()` - View user's borrowing position and interest owed

#### **Price & Analytics**
- `get_current_kale_price()` - Get real-time KALE price from Reflector oracle
- `get_yield_pool()` - View platform yield distribution and fee collection

## 🏗️ Architecture

### **Data Structures**

#### **StakingPosition**
```rust
pub struct StakingPosition {
    pub user: Address,                    // User's address
    pub kale_amount: i128,                // Amount of KALE staked
    pub start_time: u64,                  // When staking began
    pub last_claim_time: u64,             // Last reward claim time
    pub auto_adjust_enabled: bool,        // Price-based auto-adjustment
    pub price_threshold: i128,            // Price change threshold (basis points)
    pub last_adjustment_price: i128,      // Price at last adjustment
    pub total_earned: i128,               // Total rewards earned
}
```

#### **BorrowingPosition**
```rust
pub struct BorrowingPosition {
    pub user: Address,                    // User's address
    pub borrowed_amount: i128,            // Amount of KALE borrowed
    pub collateral_amount: i128,          // Amount of KALE as collateral
    pub borrow_time: u64,                 // When borrowing began
    pub interest_rate: i128,              // Annual interest rate (basis points)
    pub last_payment_time: u64,           // Last interest payment time
    pub total_interest_paid: i128,        // Total interest paid
    pub is_active: bool,                  // Whether position is active
}
```

#### **PlatformState**
```rust
pub struct PlatformState {
    pub admin: Address,                   // Platform administrator
    pub kale_token: Address,              // KALE token contract address
    pub reflector_oracle: Address,        // Reflector oracle address
    pub total_staked: i128,               // Total KALE staked across platform
    pub total_borrowed: i128,             // Total KALE borrowed across platform
    pub total_collateral: i128,           // Total KALE used as collateral
    pub staking_apy: i128,                // Annual staking yield rate (basis points)
    pub borrowing_apy: i128,              // Annual borrowing interest rate (basis points)
    pub current_kale_price: i128,         // Current KALE price from oracle
    pub last_price_update: u64,           // Last price update timestamp
    pub platform_fee_rate: i128,          // Platform fee rate (basis points)
    pub liquidation_threshold: i128,      // Collateral ratio threshold
    pub is_active: bool,                  // Platform activation status
}
```

#### **YieldPool**
```rust
pub struct YieldPool {
    pub total_rewards_distributed: i128,  // Total rewards distributed
    pub staking_rewards: i128,            // Rewards from staking
    pub borrowing_fees: i128,             // Fees from borrowing
    pub platform_fees: i128,              // Platform fee collection
    pub last_distribution_time: u64,      // Last distribution timestamp
}
```

### **Price Sources**
- **Stellar**: Reflector Stellar Pubnet oracle (real-time KALE pricing)
- **Oracle Address**: `CAVLP5DH2GJPZMVO7IJY4CVOD5MWEFTJFVPD2YY2FQXOQHRGHK4D6HLP`

## 🔧 Setup & Deployment

### **Prerequisites**
- Rust toolchain (nightly)
- Soroban CLI
- Stellar testnet account

### **Installation**
```bash
# Clone the repository
git clone <repository-url>
cd kale-lend/contracts/kale-lend

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test
```

### **Deployment**
```bash
# Deploy to Stellar testnet
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/kale_lend.wasm --source <your-account>

# Initialize the platform
soroban contract invoke --id <contract-id> -- initialize \
  --admin <admin-address> \
  --kale-token <kale-token-address> \
  --reflector-oracle CAVLP5DH2GJPZMVO7IJY4CVOD5MWEFTJFVPD2YY2FQXOQHRGHK4D6HLP \
  --staking-apy 1000 \
  --borrowing-apy 1500 \
  --platform-fee-rate 200 \
  --liquidation-threshold 15000
```

## 📊 Usage Examples

### **Stake KALE Tokens**
```bash
soroban contract invoke --id <contract-id> -- stake_kale \
  --user <user-address> \
  --amount 10000 \
  --auto-adjust-enabled true \
  --price-threshold-percent 5
```

### **Claim Staking Rewards**
```bash
soroban contract invoke --id <contract-id> -- claim_staking_rewards \
  --user <user-address>
```

### **Borrow KALE**
```bash
soroban contract invoke --id <contract-id> -- borrow_kale \
  --user <user-address> \
  --collateral-amount 20000 \
  --borrow-amount 10000
```

### **Repay Borrowed KALE**
```bash
soroban contract invoke --id <contract-id> -- repay_borrowed_kale \
  --user <user-address> \
  --repay-amount 5000
```

### **Check Price Adjustments**
```bash
soroban contract invoke --id <contract-id> -- check_price_adjustments \
  --user <user-address>
```

### **Get Current KALE Price**
```bash
soroban contract invoke --id <contract-id> -- get_current_kale_price
```

### **View Platform Statistics**
```bash
soroban contract invoke --id <contract-id> -- get_platform_state
soroban contract invoke --id <contract-id> -- get_yield_pool
```

## 🎯 Lending Strategy

### **Staking Mechanics**
1. **Deposit KALE** - Users stake KALE tokens into the platform
2. **Earn Yield** - Receive rewards based on staking APY and time staked
3. **Auto-Adjust** - Stakes automatically adjust based on KALE price movements
4. **Claim Rewards** - Users can claim accumulated rewards at any time

### **Borrowing Mechanics**
1. **Provide Collateral** - Users deposit KALE as collateral
2. **Borrow KALE** - Borrow up to collateral ratio limit
3. **Pay Interest** - Accrue interest on borrowed amount
4. **Repay & Reclaim** - Repay borrowed amount to reclaim collateral

### **Price-Based Adjustments**
```
When KALE price changes by threshold amount:
- Price Increase: Increase stake by 10% of price increase
- Price Decrease: Decrease stake by 10% of price decrease
- Threshold: Configurable (e.g., 5% price change)
```

### **Risk Management**
- **Collateral Ratio**: Minimum 150% collateral to borrow ratio
- **Liquidation Protection**: Automatic liquidation if ratio falls below threshold
- **Interest Accrual**: Real-time interest calculation and payment tracking
- **Platform Fees**: Configurable fee structure for sustainability

## 🔒 Security Features

### **Access Control**
- Admin-only platform configuration updates
- Platform activation/deactivation controls
- Emergency stop functionality

### **Collateral Safety**
- Minimum collateral ratio enforcement
- Liquidation threshold monitoring
- Interest rate management
- Slippage protection

### **Error Handling**
- Graceful failure handling
- Position tracking and validation
- Comprehensive error logging
- Safe math operations

## 📈 Performance Metrics

### **Key Performance Indicators**
- **Total Value Locked (TVL)**: Total KALE staked and collateralized
- **Staking APY**: Annual yield rate for stakers
- **Borrowing APY**: Annual interest rate for borrowers
- **Platform Utilization**: Ratio of borrowed to staked amounts
- **Price Adjustment Frequency**: How often auto-adjustments occur

### **Monitoring Dashboard**
- Real-time TVL tracking
- Staking and borrowing statistics
- Price movement alerts
- Yield distribution analytics

## 🔗 Reflector Oracle Integration

### **Real-Time Price Feeds**
Our lending platform leverages Reflector's Stellar Pubnet oracle for accurate KALE pricing:

#### **Stellar Pubnet Oracle** (`CAVLP5DH2GJPZMVO7IJY4CVOD5MWEFTJFVPD2YY2FQXOQHRGHK4D6HLP`)
- **Purpose**: Real-time KALE prices on Stellar network
- **Data Source**: Stellar Classic DEX and liquidity pools
- **Base Symbol**: USDCcentre.io
- **Decimals**: 14
- **Sampling**: 5 minutes
- **Retention**: 24 hours

### **Price Integration**
- **Automatic Updates**: KALE price updates every 5 minutes
- **Price-Based Adjustments**: Staking positions adjust based on price movements
- **Collateral Valuation**: Real-time collateral value calculation
- **Risk Assessment**: Continuous monitoring of collateral ratios

## 🚀 Future Enhancements

### **Phase 2 Features**
- **Multi-Asset Support**: Extend to other Stellar tokens
- **Flash Loans**: Capital-efficient borrowing mechanisms
- **Governance**: DAO-based platform management
- **Advanced Analytics**: Machine learning for yield optimization

### **Phase 3 Features**
- **Cross-Chain Integration**: Bridge to other blockchains
- **Liquidity Mining**: Additional reward mechanisms
- **Insurance**: DeFi insurance integration
- **Mobile App**: Real-time monitoring and control

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

For questions and support:
- Create an issue on GitHub
- Join our Discord community
- Email: support@kale-lending.com

---

**Built with ❤️ for the KALE ecosystem**
