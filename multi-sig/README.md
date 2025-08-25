# Multi-Signature Wallet Contract

A secure multi-signature wallet implementation using Arbitrum Stylus, written in Rust. This contract allows multiple owners to collectively manage funds and execute transactions that require a minimum number of confirmations.

## Features

- **Multi-owner Management**: Support for multiple wallet owners
- **Configurable Threshold**: Set required number of confirmations for transaction execution
- **Transaction Queue**: Submit, confirm, and execute transactions securely
- **Access Control**: Only owners can submit and confirm transactions
- **Event Logging**: Complete transaction history via events

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
    --endpoint <youRPCurl> \
    --private-key <yourPrivateKey> \
    --constructor-args '[["address1", "address2", "address3"], "2"]'
```

### Constructor Parameters

The contract requires initialization with:
- `owners`: Array of owner addresses (cannot be empty)
- `required`: Number of required confirmations (must be > 0 and <= owners.length)

```solidity
constructor(address[] memory owners, uint256 required)
```

### Contract Size & Cost

- **Contract Size**: ~21.6 KiB
- **Deployment Cost**: ~0.000124 ETH
- **WASM Size**: ~81.3 KiB

## Usage

### Core Functions

#### Submit Transaction
```rust
submit_transaction(to: Address, value: U256, data: Vec<u8>) -> Result<U256, Vec<u8>>
```

#### Confirm Transaction
```rust
confirm_transaction(tx_id: U256) -> Result<(), Vec<u8>>
```

#### Execute Transaction
```rust
execute_transaction(tx_id: U256) -> Result<(), Vec<u8>>
```

### View Functions

#### Get Transaction Details
```rust
get_transaction(tx_id: U256) -> (Address, U256, Vec<u8>, bool, U256)
```

#### Check Confirmation Status
```rust
is_confirmed(tx_id: U256, owner: Address) -> bool
get_confirmation_count(tx_id: U256) -> U256
```

#### Wallet Information
```rust
get_required_confirmations() -> U256
get_transaction_count() -> U256
get_owner_count() -> U256
is_owner(addr: Address) -> bool
```

## Events

- `TransactionSubmitted(uint256 indexed transaction_id, address indexed owner, address indexed to, uint256 value)`
- `TransactionConfirmed(uint256 indexed transaction_id, address indexed owner)`
- `TransactionExecuted(uint256 indexed transaction_id)`

## Security Features

- **Access Control**: Owner-only access for sensitive operations
- **Duplicate Prevention**: Prevents owners from confirming the same transaction twice
- **Threshold Enforcement**: Transactions execute only after reaching required confirmations
- **Input Validation**: Comprehensive validation for all parameters

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