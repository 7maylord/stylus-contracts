//!
//! Dutch Auction Factory Contract
//!
//! This contract creates and manages Dutch auction instances.
//! Each auction is deployed as a separate contract instance.

// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{U256, Address, B256}, 
    prelude::*, 
    deploy::RawDeploy
};
use alloy_sol_types::sol;

// Import the compiled dutch auction WASM bytecode at compile time
static DUTCH_AUCTION_WASM: &[u8] = include_bytes!("../dutch_auction.wasm");

sol_storage! {
    #[entrypoint]
    pub struct DutchAuctionFactory {
        uint256 auction_count;
        mapping(uint256 => address) auctions;
        address owner;
    }
}

#[public]
impl DutchAuctionFactory {
    /// Initialize the factory
    pub fn new(&mut self) -> Result<(), Vec<u8>> {
        self.auction_count.set(U256::from(0));
        self.owner.set(self.vm().msg_sender());
        Ok(())
    }

    /// Create and deploy a new Dutch auction contract
    pub fn create_auction(
        &mut self,
        nft_contract: Address,
        token_id: U256,
        starting_price: U256,
        ending_price: U256,
        duration: U256,
    ) -> Result<Address, Vec<u8>> {
        if nft_contract == Address::ZERO {
            return Err("Invalid NFT contract".as_bytes().to_vec());
        }
        
        if starting_price <= ending_price {
            return Err("Starting price must be higher than ending price".as_bytes().to_vec());
        }
        
        if duration == U256::from(0) {
            return Err("Duration must be greater than zero".as_bytes().to_vec());
        }

        let auction_id = self.auction_count.get() + U256::from(1);
        let sender = self.vm().msg_sender();
        
        // Use embedded bytecode
        let bytecode = DUTCH_AUCTION_WASM;
        
        // Create salt from auction parameters for deterministic addresses
        let mut salt_data = Vec::new();
        salt_data.extend_from_slice(&auction_id.as_le_bytes());
        salt_data.extend_from_slice(sender.as_slice());
        salt_data.extend_from_slice(nft_contract.as_slice());
        salt_data.extend_from_slice(&token_id.as_le_bytes());
        
        let salt = B256::from_slice(&self.vm().native_keccak256(&salt_data)[0..32]);

        // Deploy the auction contract using RawDeploy with CREATE2
        let auction_address = unsafe {
            RawDeploy::new()
                .salt(salt)
                .deploy(&bytecode, U256::from(0))
                .map_err(|e| {
                    let mut err = "Failed to deploy auction contract: ".as_bytes().to_vec();
                    err.extend_from_slice(&e);
                    err
                })?
        };
        
        // Store the deployed auction address
        self.auction_count.set(auction_id);
        self.auctions.setter(auction_id).set(auction_address);
        
        log(self.vm(), AuctionCreated {
            auction_id,
            creator: sender,
            nft_contract,
            token_id,
            starting_price,
            ending_price,
            duration,
            auction_address,
        });
        
        Ok(auction_address)
    }

    /// Get auction address by ID
    pub fn get_auction(&self, auction_id: U256) -> Address {
        self.auctions.get(auction_id)
    }

    /// Get total number of auctions created
    pub fn get_auction_count(&self) -> U256 {
        self.auction_count.get()
    }

    /// Get factory owner
    pub fn get_owner(&self) -> Address {
        self.owner.get()
    }

    /// Get embedded auction bytecode length
    pub fn get_bytecode_length(&self) -> U256 {
        U256::from(DUTCH_AUCTION_WASM.len())
    }
}

sol! {
    event AuctionCreated(
        uint256 indexed auction_id,
        address indexed creator,
        address indexed nft_contract,
        uint256 token_id,
        uint256 starting_price,
        uint256 ending_price,
        uint256 duration,
        address auction_address
    );
}