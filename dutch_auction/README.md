# Dutch Auction Contract

A Dutch auction implementation using Arbitrum Stylus, written in Rust. In a Dutch auction, the price starts high and decreases linearly over time until a buyer accepts the current price or the auction ends. **Now supports ERC20 token payments!**

## Features

- **ERC20 Token Payments**: Uses any ERC20 token as payment currency instead of ETH
- **Linear Price Decay**: Price decreases linearly from starting to ending price over time  
- **Immediate Settlement**: First buyer to accept the price wins the auction
- **Automatic Transfers**: Direct seller payment via ERC20 transferFrom calls
- **Smart Refund Logic**: Handles excess payment scenarios gracefully
- **Owner Controls**: Seller can update price, extend duration, or cancel before any bids
- **Time-based Pricing**: Real-time price calculation based on elapsed time
- **Event Logging**: Complete auction activity tracking via events

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) toolchain
- [Cargo Stylus](https://github.com/OffchainLabs/cargo-stylus)

### Installation

```bash
cargo install cargo-stylus
rustup target add wasm32-unknown-unknown
```

### Build Commands

#### Check contract validity:
```bash
cargo stylus check
```

#### Build for production:
```bash
cargo build --release
```

#### Export ABI:
```bash
cargo stylus export-abi
```

### Deployment

#### Deploy to Arbitrum Sepolia (testnet):
```bash
cargo stylus deploy \
    --endpoint <yourRPCurl> \
    --private-key <yourPrivateKey>
```


### Constructor Parameters

The contract can be deployed with no constructor parameters and initialized later:

```rust
// Deploy with empty initialization
init() -> Result<(), Vec<u8>>

// Initialize auction with parameters (including ERC20 payment token)
initialize(
    nft_contract: Address,
    token_id: U256,
    payment_token: Address,  // NEW: ERC20 token for payments
    starting_price: U256,
    ending_price: U256,
    duration: U256
) -> Result<(), Vec<u8>>
```

### Contract Size & Cost

- **Contract Size**: ~17.6 KiB
- **Deployment Cost**: ~0.000110 ETH
- **WASM Size**: ~67.9 KiB

## Usage

### Core Functions

#### Get Current Price
```rust
get_current_price() -> U256
```
Returns the current auction price based on elapsed time.

#### Buy NFT
```rust
buy(max_payment: U256) -> Result<(), Vec<u8>>
```
Purchase the NFT using ERC20 tokens. Buyer must approve the contract for token spending first. Handles excess payment with automatic refunds.

#### Update Listing Price
```rust
update_listing_price(new_price: U256) -> Result<(), Vec<u8>>
```
Seller can update the ending price before any bids.

#### Extend Auction
```rust
extend_auction(additional_time: U256) -> Result<(), Vec<u8>>
```
Seller can extend auction duration before any bids.

#### Cancel Listing
```rust
cancel_listing() -> Result<(), Vec<u8>>
```
Seller can cancel the auction before any bids.

#### Emergency Stop
```rust
emergency_stop() -> Result<(), Vec<u8>>
```
Seller can emergency stop the auction.

### View Functions

#### Auction Information
```rust
get_auction_details() -> (Address, Address, U256, Address, U256, U256, U256, U256, bool, Address, U256)
// Returns: (seller, nft_contract, token_id, payment_token, starting_price, ending_price, duration, start_time, ended, winner, final_price)
get_seller() -> Address
get_winner() -> Address
get_final_price() -> U256
get_payment_token() -> Address  // NEW: Get the ERC20 payment token address
```

#### Status Checks
```rust
has_ended() -> bool
get_time_remaining() -> U256
get_price_at_time(timestamp: U256) -> U256
```

## ERC20 Payment Integration

### How It Works

1. **Buyer Approval**: Buyer must approve the auction contract to spend their tokens
   ```solidity
   ERC20Token.approve(auction_contract, max_amount)
   ```

2. **Purchase Flow**:
   - Buyer calls `buy(max_payment)`
   - Contract transfers `current_price` from buyer to seller
   - Any excess payment is handled gracefully
   - Payment and refund events are emitted

3. **Direct Settlement**: 
   - Seller receives payment immediately via `transferFrom`
   - No escrow period required
   - Gas-efficient single transaction

### Events
- `PaymentProcessed(buyer, seller, amount)` - Successful payment
- `RefundFailed(buyer, amount)` - If excess payment refund fails
- `ERC20CallMade(token, selector, success)` - ERC20 call tracking

## Price Calculation

The Dutch auction uses linear price decay:

```
current_price = starting_price - ((starting_price - ending_price) * elapsed_time / duration)
```

- **Starting Price**: Initial high price
- **Ending Price**: Final low price (reserve price)
- **Duration**: Total auction time in seconds
- **Elapsed Time**: Time since auction started

## Events

- `AuctionCreated(address indexed seller, address indexed nft_contract, uint256 indexed token_id, uint256 starting_price, uint256 ending_price, uint256 duration, uint256 start_time)`
- `AuctionEnded(address indexed winner, uint256 final_price, uint256 end_time)`
- `AuctionStopped(address indexed seller, uint256 stop_time)`
- `AuctionExtended(uint256 additional_time, uint256 new_duration)`
- `EndingPriceUpdated(uint256 old_ending_price, uint256 new_ending_price)`

## Security Features

- **Reentrancy Protection**: Utilizes OpenZeppelin's ReentrancyGuard to prevent reentrancy attacks
- **Access Control**: Seller cannot bid on their own auction
- **Price Validation**: Starting price must be greater than ending price
- **Duration Validation**: Auction duration must be greater than 0
- **Single Winner Enforcement**: Only one buyer can win the auction
- **Comprehensive Input Validation**: All parameters are validated before processing

## Factory Integration

This contract is designed to work with the `DutchAuctionFactory`:

1. Factory deploys new auction instances
2. Factory calls `initialize()` with auction parameters
3. Individual auctions operate independently

## Development

### Run Tests
```bash
cargo test
```

### Local Development
```bash
cargo stylus check --endpoint http://localhost:8547
```

## License

This project is licensed under MIT OR Apache-2.0.