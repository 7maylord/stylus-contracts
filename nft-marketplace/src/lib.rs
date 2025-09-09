extern crate alloc;

use stylus_sdk::{
    prelude::*,
    alloy_sol_types::sol,
};
use alloy_primitives::{U256, Address};

sol_storage! {
    #[entrypoint]
    pub struct NFTMarketplace {
        address owner;
        uint256 fee_percentage;
        uint256 item_count;
        mapping(uint256 => MarketItem) market_items;
        mapping(uint256 => bool) sold_items;
    }

    pub struct MarketItem {
        uint256 item_id;
        address nft_contract;
        uint256 token_id;
        address seller;
        address owner;
        uint256 price;
        bool sold;
    }
}

#[public]
impl NFTMarketplace {
    /// Initialize marketplace
    pub fn new(&mut self, fee_percentage: U256) -> Result<(), Vec<u8>> {
        if fee_percentage > U256::from(1000) { // Max 10%
            return Err("Fee too high".as_bytes().to_vec());
        }
        
        self.owner.set(self.vm().msg_sender());
        self.fee_percentage.set(fee_percentage);
        self.item_count.set(U256::from(0));
        
        Ok(())
    }

    /// List NFT for sale
    pub fn create_market_item(
        &mut self,
        nft_contract: Address,
        token_id: U256,
        price: U256,
    ) -> Result<U256, Vec<u8>> {
        if price <= U256::from(0) {
            return Err("Price must be greater than zero".as_bytes().to_vec());
        }
        
        let item_id = self.item_count.get() + U256::from(1);
        let sender = self.vm().msg_sender();
        
        let mut market_item = self.market_items.setter(item_id);
        market_item.item_id.set(item_id);
        market_item.nft_contract.set(nft_contract);
        market_item.token_id.set(token_id);
        market_item.seller.set(sender);
        market_item.owner.set(Address::ZERO);
        market_item.price.set(price);
        market_item.sold.set(false);
        
        self.item_count.set(item_id);
        self.sold_items.setter(item_id).set(false);
        
        log(self.vm(), MarketItemCreated {
            item_id,
            nft_contract,
            token_id,
            seller: sender,
            price,
        });
        
        Ok(item_id)
    }

    /// Buy NFT from marketplace
    pub fn buy_market_item(&mut self, item_id: U256) -> Result<(), Vec<u8>> {
        // First, check conditions and capture needed values
        let (is_sold, price, nft_contract, token_id, seller) = {
            let item = self.market_items.get(item_id);
            (
                item.sold.get(),
                item.price.get(),
                item.nft_contract.get(),
                item.token_id.get(),
                item.seller.get(),
            )
        };
        
        if is_sold {
            return Err("Item already sold".as_bytes().to_vec());
        }
        if self.vm().msg_value() != price {
            return Err("Incorrect payment amount".as_bytes().to_vec());
        }
        
        let fee = (price * self.fee_percentage.get()) / U256::from(10000);
        let _seller_amount = price - fee;
        
        // Now we can safely get the mutable reference
        let buyer = self.vm().msg_sender();
        let mut item_setter = self.market_items.setter(item_id);
        item_setter.owner.set(buyer);
        item_setter.sold.set(true);
        self.sold_items.setter(item_id).set(true);
        
        log(self.vm(), MarketItemSold {
            item_id,
            nft_contract,
            token_id,
            seller,
            buyer,
            price,
        });
        
        Ok(())
    }

    /// Get market item details
    pub fn get_market_item(&self, item_id: U256) -> (U256, Address, U256, Address, Address, U256, bool) {
        let item = self.market_items.get(item_id);
        (
            item.item_id.get(),
            item.nft_contract.get(),
            item.token_id.get(),
            item.seller.get(),
            item.owner.get(),
            item.price.get(),
            item.sold.get(),
        )
    }

    /// Update listing price
    pub fn update_listing_price(&mut self, item_id: U256, new_price: U256) -> Result<(), Vec<u8>> {
        let item = self.market_items.get(item_id);
        
        if item.seller.get() != self.vm().msg_sender() {
            return Err("Only seller can update price".as_bytes().to_vec());
        }
        if item.sold.get() {
            return Err("Item already sold".as_bytes().to_vec());
        }
        if new_price <= U256::from(0) {
            return Err("Price must be greater than zero".as_bytes().to_vec());
        }
        
        let old_price = item.price.get();
        self.market_items.setter(item_id).price.set(new_price);
        
        log(self.vm(), ListingPriceUpdated {
            item_id,
            old_price,
            new_price,
        });
        
        Ok(())
    }

    /// Cancel listing
    pub fn cancel_listing(&mut self, item_id: U256) -> Result<(), Vec<u8>> {
        let item = self.market_items.get(item_id);
        
        if item.seller.get() != self.vm().msg_sender() {
            return Err("Only seller can cancel listing".as_bytes().to_vec());
        }
        if item.sold.get() {
            return Err("Item already sold".as_bytes().to_vec());
        }
        
        // Mark as sold to prevent further sales
        self.market_items.setter(item_id).sold.set(true);
        self.sold_items.setter(item_id).set(true);
        
        log(self.vm(), ListingCancelled {
            item_id,
            seller: self.vm().msg_sender(),
        });
        
        Ok(())
    }

    /// Withdraw marketplace fees (owner only)
    pub fn withdraw_fees(&mut self) -> Result<(), Vec<u8>> {
        if self.vm().msg_sender() != self.owner.get() {
            return Err("Only owner can withdraw fees".as_bytes().to_vec());
        }
        
        // In a real implementation, would check contract balance
        // and transfer ETH to owner using call or transfer
        let balance = U256::from(0); // Placeholder - would get actual contract balance
        
        log(self.vm(), FeesWithdrawn {
            owner: self.vm().msg_sender(),
            amount: balance,
        });
        
        Ok(())
    }

    /// Check if item is sold
    pub fn is_item_sold(&self, item_id: U256) -> bool {
        self.sold_items.get(item_id)
    }

    /// Get marketplace fee percentage
    pub fn get_fee_percentage(&self) -> U256 {
        self.fee_percentage.get()
    }

    /// Check if item exists
    pub fn item_exists(&self, item_id: U256) -> bool {
        item_id <= self.item_count.get() && item_id > U256::from(0)
    }    

    /// Get total items count
    pub fn get_item_count(&self) -> U256 {
        self.item_count.get()
    }
}

sol! {
    event MarketItemCreated(
        uint256 indexed item_id,
        address indexed nft_contract,
        uint256 indexed token_id,
        address seller,
        uint256 price
    );
    event MarketItemSold(
        uint256 indexed item_id,
        address indexed nft_contract,
        uint256 indexed token_id,
        address seller,
        address buyer,
        uint256 price
    );
    event ListingPriceUpdated(uint256 indexed item_id, uint256 old_price, uint256 new_price);
    event ListingCancelled(uint256 indexed item_id, address indexed seller);
    event FeesWithdrawn(address indexed owner, uint256 amount);
}