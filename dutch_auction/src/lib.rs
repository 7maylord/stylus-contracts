

#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::sol,
    block, call, contract, msg,
    prelude::*,
};

// ERC721 interface for NFT transfers
sol_interface! {
    interface IERC721 {
        function transferFrom(address from, address to, uint256 tokenId) external;
        function ownerOf(uint256 tokenId) external view returns (address);
        function getApproved(uint256 tokenId) external view returns (address);
        function isApprovedForAll(address owner, address operator) external view returns (bool);
    }
}

// Custom error types
sol! {
    error AuctionNotActive();
    error AuctionAlreadyEnded();
    error OnlySeller();
    error InvalidPrice();
    error PaymentFailed();
    error ZeroAddress();
    error InvalidDuration();
    error NFTTransferFailed();
    error NotNFTOwner();
    error NotApproved();
    error AuctionNotStarted();
}

#[derive(SolidityError)]
pub enum DutchAuctionError {
    AuctionNotActive(AuctionNotActive),
    AuctionAlreadyEnded(AuctionAlreadyEnded),
    OnlySeller(OnlySeller),
    InvalidPrice(InvalidPrice),
    PaymentFailed(PaymentFailed),
    ZeroAddress(ZeroAddress),
    InvalidDuration(InvalidDuration),
    NFTTransferFailed(NFTTransferFailed),
    NotNFTOwner(NotNFTOwner),
    NotApproved(NotApproved),
    AuctionNotStarted(AuctionNotStarted),
}

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
    /// Initialize Dutch auction with NFT verification
    pub fn new(
        &mut self,
        seller: Address,
        nft_contract: Address,
        token_id: U256,
        starting_price: U256,
        ending_price: U256,
        duration: U256,
    ) -> Result<(), DutchAuctionError> {
        if seller == Address::ZERO || nft_contract == Address::ZERO {
            return Err(DutchAuctionError::ZeroAddress(ZeroAddress {}));
        }
        
        if duration == U256::ZERO {
            return Err(DutchAuctionError::InvalidDuration(InvalidDuration {}));
        }

        if starting_price <= ending_price {
            return Err(DutchAuctionError::InvalidPrice(InvalidPrice {}));
        }

        // Set contract state first
        self.seller.set(seller);
        self.nft_contract.set(nft_contract);
        self.token_id.set(token_id);
        self.starting_price.set(starting_price);
        self.ending_price.set(ending_price);
        self.duration.set(duration);
        self.start_time.set(U256::from(block::timestamp()));
        self.ended.set(false);
        self.winner.set(Address::ZERO);
        self.final_price.set(U256::ZERO);

        // Verify NFT ownership and approval
        self.verify_nft_authorization(seller)?;

        Ok(())
    }

    /// Get current price of the auction
    pub fn get_current_price(&self) -> Result<U256, DutchAuctionError> {
        if self.ended.get() {
            return Err(DutchAuctionError::AuctionAlreadyEnded(AuctionAlreadyEnded {}));
        }

        let current_time = U256::from(block::timestamp());
        let start_time = self.start_time.get();
        let duration = self.duration.get();
        let starting_price = self.starting_price.get();
        let ending_price = self.ending_price.get();

        if current_time < start_time {
            return Ok(starting_price);
        }

        let elapsed_time = current_time - start_time;
        
        if elapsed_time >= duration {
            return Ok(ending_price);
        }

        // Calculate current price: starting_price - (price_drop * elapsed_time / duration)
        let price_drop = starting_price - ending_price;
        let price_reduction = (price_drop * elapsed_time) / duration;
        
        Ok(starting_price - price_reduction)
    }

    /// Purchase the item at current price
    pub fn buy(&mut self) -> Result<(), DutchAuctionError> {
        if self.ended.get() {
            return Err(DutchAuctionError::AuctionAlreadyEnded(AuctionAlreadyEnded {}));
        }

        let current_price = self.get_current_price()?;
        let payment = msg::value();
        let buyer = msg::sender();
        let seller = self.seller.get();

        if payment < current_price {
            return Err(DutchAuctionError::InvalidPrice(InvalidPrice {}));
        }

        self.ended.set(true);
        self.winner.set(buyer);
        self.final_price.set(current_price);

        if current_price > U256::ZERO {
            self.transfer_payment(seller, current_price)?;
        }
        
        self.transfer_nft(seller, buyer)?;
        
        let excess = payment - current_price;
        if excess > U256::ZERO {
            self.refund_excess(buyer, excess)?;
        }
        
        Ok(())
    }

    /// Verify NFT ownership and approval before auction start
    fn verify_nft_authorization(&mut self, seller: Address) -> Result<(), DutchAuctionError> {
        let nft_contract = IERC721::new(self.nft_contract.get());
        let token_id = self.token_id.get();

        // Check if seller owns the NFT
        let owner_result = nft_contract.owner_of(call::Call::new_in(self), token_id);
        match owner_result {
            Ok(owner) => {
                if owner != seller {
                    return Err(DutchAuctionError::NotNFTOwner(NotNFTOwner {}));
                }
            }
            Err(_) => return Err(DutchAuctionError::NFTTransferFailed(NFTTransferFailed {})),
        }

        // Check if this contract is approved to transfer the NFT
        let contract_address = contract::address();
        let approved_result = nft_contract.get_approved(call::Call::new_in(self), token_id);
        let approved_for_all_result = nft_contract.is_approved_for_all(call::Call::new_in(self), seller, contract_address);

        let is_approved = match approved_result {
            Ok(approved) => approved == contract_address,
            Err(_) => false,
        };

        let is_approved_for_all = match approved_for_all_result {
            Ok(approved) => approved,
            Err(_) => false,
        };

        if !is_approved && !is_approved_for_all {
            return Err(DutchAuctionError::NotApproved(NotApproved {}));
        }

        Ok(())
    }

    /// Transfer NFT from seller to buyer 
    fn transfer_nft(&mut self, from: Address, to: Address) -> Result<(), DutchAuctionError> {
        let nft_contract = IERC721::new(self.nft_contract.get());
        let token_id = self.token_id.get();

        // Attempt to transfer the NFT
        let result = nft_contract.transfer_from(call::Call::new_in(self), from, to, token_id);
        
        if result.is_err() {
            return Err(DutchAuctionError::NFTTransferFailed(NFTTransferFailed {}));
        }

        Ok(())
    }

    /// Transfer payment to seller
    fn transfer_payment(&self, to: Address, amount: U256) -> Result<(), DutchAuctionError> {
        if to == Address::ZERO {
            return Err(DutchAuctionError::ZeroAddress(ZeroAddress {}));
        }
        
        if amount == U256::ZERO {
            return Err(DutchAuctionError::InvalidPrice(InvalidPrice {}));
        }

        // Transfer ETH to the seller
        let result = call::transfer_eth(to, amount);
        if result.is_err() {
            return Err(DutchAuctionError::PaymentFailed(PaymentFailed {}));
        }

        Ok(())
    }

    /// Refund excess payment to buyer
    fn refund_excess(&self, to: Address, amount: U256) -> Result<(), DutchAuctionError> {
        if to == Address::ZERO {
            return Err(DutchAuctionError::ZeroAddress(ZeroAddress {}));
        }
        
        if amount == U256::ZERO {
            return Err(DutchAuctionError::InvalidPrice(InvalidPrice {}));
        }

        // Refund excess ETH to the buyer
        let result = call::transfer_eth(to, amount);
        if result.is_err() {
            return Err(DutchAuctionError::PaymentFailed(PaymentFailed {}));
        }

        Ok(())
    }

    /// Stop the auction (only seller)
    pub fn stop_auction(&mut self) -> Result<(), DutchAuctionError> {
        if msg::sender() != self.seller.get() {
            return Err(DutchAuctionError::OnlySeller(OnlySeller {}));
        }

        if self.ended.get() {
            return Err(DutchAuctionError::AuctionAlreadyEnded(AuctionAlreadyEnded {}));
        }

        self.ended.set(true);
        Ok(())
    }

    /// Check if auction is active
    pub fn is_active(&self) -> bool {
        !self.ended.get()
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

    // View functions
    pub fn seller(&self) -> Address {
        self.seller.get()
    }

    pub fn nft_contract(&self) -> Address {
        self.nft_contract.get()
    }

    pub fn token_id(&self) -> U256 {
        self.token_id.get()
    }

    pub fn starting_price(&self) -> U256 {
        self.starting_price.get()
    }

    pub fn ending_price(&self) -> U256 {
        self.ending_price.get()
    }

    pub fn duration(&self) -> U256 {
        self.duration.get()
    }

    pub fn start_time(&self) -> U256 {
        self.start_time.get()
    }

    pub fn ended(&self) -> bool {
        self.ended.get()
    }

    pub fn winner(&self) -> Address {
        self.winner.get()
    }

    pub fn final_price(&self) -> U256 {
        self.final_price.get()
    }
}