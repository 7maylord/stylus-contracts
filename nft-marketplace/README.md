# NFT Marketplace

A decentralized NFT marketplace smart contract written in Rust for Arbitrum Stylus. This contract enables users to list, buy, and manage NFT sales with built-in fee collection and marketplace functionality.

## Features

- **NFT Listing**: Create marketplace listings for any NFT contract
- **Secure Buying**: Safe purchase mechanism with payment validation
- **Fee Collection**: Configurable marketplace fees with owner withdrawal
- **Price Management**: Update listing prices and cancel listings
- **Ownership Tracking**: Track sellers, buyers, and listing status
- **Event Logging**: Comprehensive event emission for marketplace activities
- **Gas Efficient**: Optimized for low transaction costs on Arbitrum

## Contract Structure

The marketplace manages the following key components:

- **Market Items**: NFT listings with pricing, ownership, and status information
- **Fee System**: Percentage-based fees with secure collection mechanism  
- **Ownership Controls**: Marketplace owner functions for fee management
- **Sales Tracking**: Complete transaction history and item status

## Main Functions

### Core Marketplace Functions
- `new(fee_percentage)` - Initialize marketplace with fee structure
- `create_market_item(nft_contract, token_id, price)` - List NFT for sale
- `buy_market_item(item_id)` - Purchase listed NFT
- `update_listing_price(item_id, new_price)` - Modify listing price
- `cancel_listing(item_id)` - Remove NFT from marketplace

### View Functions
- `get_market_item(item_id)` - Retrieve complete listing information
- `item_exists(item_id)` - Check if listing exists
- `is_item_sold(item_id)` - Check if item has been sold
- `get_fee_percentage()` - Get current marketplace fee
- `get_item_count()` - Get total number of listings

### Owner Functions
- `withdraw_fees()` - Withdraw collected marketplace fees

## Quick Start 

Install [Rust](https://www.rust-lang.org/tools/install), and then install the Stylus CLI tool with Cargo

```bash
cargo install --force cargo-stylus ```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```
rustup target add wasm32-unknown-unknown
```

You should now have it available as a Cargo subcommand:

```bash
cargo stylus --help
```

### Testnet Information

All testnet information, including faucets and RPC endpoints can be found [here](https://docs.arbitrum.io/stylus/reference/testnet-information).

### ABI Export

You can export the Solidity ABI for your program by using the `cargo stylus` tool as follows:

```bash
cargo stylus export-abi
```

Exporting ABIs uses a feature that is enabled by default in your Cargo.toml:

```toml
[features]
export-abi = ["stylus-sdk/export-abi"]
```

## Deploying

You can use the `cargo stylus` command to also deploy your program to the Stylus testnet. We can use the tool to first check
our program compiles to valid WASM for Stylus and will succeed a deployment onchain without transacting. By default, this will use the Stylus testnet public RPC endpoint. See here for [Stylus testnet information](https://docs.arbitrum.io/stylus/reference/testnet-information)

```bash
cargo stylus check
```

If successful, then run

```bash
 cargo stylus deploy \
    --endpoint <rpcurl> \
    --private-key <yourprivatekey> \
    --constructor-args 500
```

Platform fee Calculation
500 = 5%
250 = 2.5%
1000 = 10% highest


## Security Features

- **Payment Validation**: Ensures correct payment amount matches listing price
- **Ownership Verification**: Validates seller permissions for price updates and cancellations
- **Fee Protection**: Secure fee calculation and withdrawal mechanisms
- **State Consistency**: Prevents double spending and invalid state transitions
- **Access Control**: Owner-only functions for marketplace administration
- **Input Validation**: Comprehensive parameter validation and error handling
- **Reentrancy Protection**: Safe external call patterns and state updates

## Events

The contract emits the following events for transparency and marketplace monitoring:

- `MarketItemCreated(item_id, nft_contract, token_id, seller, price)` - New listing created
- `MarketItemSold(item_id, nft_contract, token_id, seller, buyer, price)` - NFT purchased
- `ListingPriceUpdated(item_id, old_price, new_price)` - Price modification
- `ListingCancelled(item_id, seller)` - Listing removed from marketplace
- `FeesWithdrawn(owner, amount)` - Marketplace fees collected

## Build Options

By default, the cargo stylus tool will build your project for WASM using sensible optimizations, but you can control how this gets compiled by seeing the full README for [cargo stylus](https://github.com/OffchainLabs/cargo-stylus). If you wish to optimize the size of your compiled WASM, see the different options available [here](https://github.com/OffchainLabs/cargo-stylus/blob/main/OPTIMIZING_BINARIES.md).

## Development and Testing

### Expanding Macros

The [stylus-sdk](https://github.com/OffchainLabs/stylus-sdk-rs) uses helpful macros that expand into pure Rust code. To see what the NFT Marketplace contract expands into, use `cargo expand`:

First, run `cargo install cargo-expand` if you don't have it, then:

```
cargo expand --all-features --release --target=<YOUR_ARCHITECTURE>
```

Where you can find `YOUR_ARCHITECTURE` by running `rustc -vV | grep host`. For M1 Apple computers, for example, this is `aarch64-apple-darwin`.

### Testing Scenarios

The marketplace includes comprehensive validation and error handling. Consider testing:

- **Listing Management**: Create, update, and cancel NFT listings
- **Purchase Flow**: Buy NFTs with correct payment validation
- **Fee Calculation**: Verify marketplace fee collection and distribution
- **Access Control**: Test seller-only and owner-only functions
- **Edge Cases**: Invalid payments, sold items, non-existent listings
- **Event Emission**: Verify all marketplace events are properly logged

### Use Cases

This NFT Marketplace is suitable for:

- **Digital Art Platforms**: Trade original artwork and collectibles
- **Gaming Assets**: Exchange in-game items and characters
- **Music & Media**: Distribute and monetize creative content
- **Domain Names**: Trade ENS domains and other naming assets
- **Utility NFTs**: Marketplace for membership tokens and access rights
- **Cross-chain Trading**: Bridge NFTs across different blockchain networks

## License

This project is fully open source, including an Apache-2.0 or MIT license at your choosing under your own copyright.
