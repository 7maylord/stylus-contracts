// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::{U256, Address}, prelude::*};
use alloy_sol_types::sol;


sol_storage! {
    #[entrypoint]
    pub struct DutchAuction {
        address seller;
        address nft_contract;
        uint256 token_id;
        uint256 starting_price;
        uint256 ending_price;
        uint256 duration;
        uint256 start_time;
        bool ended;
        address winner;
        uint256 final_price;
    }
}

#[public]
impl DutchAuction {
    /// Initialize empty Dutch auction (called by factory)  
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        // Initialize with empty/zero values
        self.seller.set(Address::ZERO);
        self.nft_contract.set(Address::ZERO);
        self.token_id.set(U256::from(0));
        self.starting_price.set(U256::from(0));
        self.ending_price.set(U256::from(0));
        self.duration.set(U256::from(0));
        self.start_time.set(U256::from(0));
        self.ended.set(false);
        self.winner.set(Address::ZERO);
        self.final_price.set(U256::from(0));
        
        Ok(())
    }

    /// Initialize auction with parameters (called by factory after deployment)
    pub fn initialize(
        &mut self,
        nft_contract: Address,
        token_id: U256,
        starting_price: U256,
        ending_price: U256,
        duration: U256,
    ) -> Result<(), Vec<u8>> {
        // Only allow initialization once
        if self.seller.get() != Address::ZERO {
            return Err("Already initialized".as_bytes().to_vec());
        }

        if nft_contract == Address::ZERO {
            return Err("Invalid NFT contract".as_bytes().to_vec());
        }
        
        if starting_price <= ending_price {
            return Err("Starting price must be higher than ending price".as_bytes().to_vec());
        }
        
        if duration == U256::from(0) {
            return Err("Duration must be greater than zero".as_bytes().to_vec());
        }
        
        let sender = self.vm().msg_sender();
        let timestamp = U256::from(self.vm().block_timestamp());
        
        self.seller.set(sender);
        self.nft_contract.set(nft_contract);
        self.token_id.set(token_id);
        self.starting_price.set(starting_price);
        self.ending_price.set(ending_price);
        self.duration.set(duration);
        self.start_time.set(timestamp);
        
        log(self.vm(), AuctionCreated {
            seller: sender,
            nft_contract,
            token_id,
            starting_price,
            ending_price,
            duration,
            start_time: timestamp,
        });
        
        Ok(())
    }

    /// Get current price of the auction
    pub fn get_current_price(&self) -> U256 {
        if self.ended.get() {
            return self.final_price.get();
        }
        
        let current_time = U256::from(self.vm().block_timestamp());
        let start_time = self.start_time.get();
        let duration = self.duration.get();
        
        if current_time >= start_time + duration {
            // Auction has ended, return ending price
            return self.ending_price.get();
        }
        
        let elapsed = current_time - start_time;
        let starting_price = self.starting_price.get();
        let ending_price = self.ending_price.get();
        
        // Linear price decrease over time
        let price_decrease = ((starting_price - ending_price) * elapsed) / duration;
        starting_price - price_decrease
    }

    /// Buy NFT at current price
    pub fn buy(&mut self) -> Result<(), Vec<u8>> {
        if self.ended.get() {
            return Err("Auction has ended".as_bytes().to_vec());
        }
        
        let sender = self.vm().msg_sender();
        let value = self.vm().msg_value();
        
        if sender == self.seller.get() {
            return Err("Seller cannot buy their own NFT".as_bytes().to_vec());
        }
        
        let current_price = self.get_current_price();
        
        if value < current_price {
            return Err("Insufficient payment".as_bytes().to_vec());
        }
        
        // End the auction
        self.ended.set(true);
        self.winner.set(sender);
        self.final_price.set(current_price);
        
        // Refund excess payment
        let excess = value - current_price;
        if excess > U256::from(0) {
            // In real implementation, would refund excess to buyer
        }
        
        // Transfer payment to seller (in real implementation)
        // Transfer NFT to buyer (in real implementation)
        
        log(self.vm(), AuctionEnded {
            winner: sender,
            final_price: current_price,
            end_time: U256::from(self.vm().block_timestamp()),
        });
        
        Ok(())
    }

    /// Emergency stop auction (seller only)
    pub fn emergency_stop(&mut self) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        
        if sender != self.seller.get() {
            return Err("Only seller can stop auction".as_bytes().to_vec());
        }
        
        if self.ended.get() {
            return Err("Auction already ended".as_bytes().to_vec());
        }
        
        self.ended.set(true);
        
        log(self.vm(), AuctionStopped {
            seller: sender,
            stop_time: U256::from(self.vm().block_timestamp()),
        });
        
        Ok(())
    }

    /// Check if auction has ended
    pub fn has_ended(&self) -> bool {
        if self.ended.get() {
            return true;
        }
        
        let current_time = U256::from(self.vm().block_timestamp());
        let end_time = self.start_time.get() + self.duration.get();
        current_time >= end_time
    }

    /// Get time remaining in auction
    pub fn get_time_remaining(&self) -> U256 {
        if self.ended.get() {
            return U256::from(0);
        }
        
        let current_time = U256::from(self.vm().block_timestamp());
        let end_time = self.start_time.get() + self.duration.get();
        
        if current_time >= end_time {
            return U256::from(0);
        }
        
        end_time - current_time
    }

    /// Get auction details
    pub fn get_auction_details(&self) -> (Address, Address, U256, U256, U256, U256, U256, bool, Address, U256) {
        (
            self.seller.get(),
            self.nft_contract.get(),
            self.token_id.get(),
            self.starting_price.get(),
            self.ending_price.get(),
            self.duration.get(),
            self.start_time.get(),
            self.ended.get(),
            self.winner.get(),
            self.final_price.get(),
        )
    }

    /// Calculate price at specific timestamp
    pub fn get_price_at_time(&self, timestamp: U256) -> U256 {
        let start_time = self.start_time.get();
        let duration = self.duration.get();
        
        if timestamp < start_time {
            return self.starting_price.get();
        }
        
        if timestamp >= start_time + duration {
            return self.ending_price.get();
        }
        
        let elapsed = timestamp - start_time;
        let starting_price = self.starting_price.get();
        let ending_price = self.ending_price.get();
        
        let price_decrease = ((starting_price - ending_price) * elapsed) / duration;
        starting_price - price_decrease
    }

    /// Extend auction duration (seller only, before any bids)
    pub fn extend_auction(&mut self, additional_time: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        
        if sender != self.seller.get() {
            return Err("Only seller can extend auction".as_bytes().to_vec());
        }
        
        if self.ended.get() {
            return Err("Cannot extend ended auction".as_bytes().to_vec());
        }
        
        if self.winner.get() != Address::ZERO {
            return Err("Cannot extend auction with bids".as_bytes().to_vec());
        }
        
        let new_duration = self.duration.get() + additional_time;
        self.duration.set(new_duration);
        
        log(self.vm(), AuctionExtended {
            additional_time,
            new_duration,
        });
        
        Ok(())
    }

    /// Update ending price (seller only, before any bids)
    pub fn update_ending_price(&mut self, new_ending_price: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        
        if sender != self.seller.get() {
            return Err("Only seller can update ending price".as_bytes().to_vec());
        }
        
        if self.ended.get() {
            return Err("Cannot update ended auction".as_bytes().to_vec());
        }
        
        if self.winner.get() != Address::ZERO {
            return Err("Cannot update auction with bids".as_bytes().to_vec());
        }
        
        if new_ending_price >= self.starting_price.get() {
            return Err("Ending price must be less than starting price".as_bytes().to_vec());
        }
        
        let old_ending_price = self.ending_price.get();
        self.ending_price.set(new_ending_price);
        
        log(self.vm(), EndingPriceUpdated {
            old_ending_price,
            new_ending_price,
        });
        
        Ok(())
    }

    /// Get seller
    pub fn get_seller(&self) -> Address {
        self.seller.get()
    }

    /// Get winner
    pub fn get_winner(&self) -> Address {
        self.winner.get()
    }

    /// Get final price
    pub fn get_final_price(&self) -> U256 {
        self.final_price.get()
    }
}

sol! {
    event AuctionCreated(
        address indexed seller,
        address indexed nft_contract,
        uint256 indexed token_id,
        uint256 starting_price,
        uint256 ending_price,
        uint256 duration,
        uint256 start_time
    );
    event AuctionEnded(address indexed winner, uint256 final_price, uint256 end_time);
    event AuctionStopped(address indexed seller, uint256 stop_time);
    event AuctionExtended(uint256 additional_time, uint256 new_duration);
    event EndingPriceUpdated(uint256 old_ending_price, uint256 new_ending_price);
}