#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, symbol_short, token, vec,
    Address, Env, String, Vec,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ExternalCircleRecord {
    pub id: u64,
    pub creator: Address,
    pub title: String,
    pub metadata_uri: String,
    pub target_quantity: u32,
    pub target_price_stroops: i128,
    pub deadline: u64,
    pub created_at: u64,
    pub status: u32,
}

#[contractclient(name = "DemandCircleRegistryClient")]
pub trait DemandCircleRegistryInterface {
    fn get_circle(env: Env, circle_id: u64) -> ExternalCircleRecord;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    CircleNotFound = 4,
    CircleAlreadyClosed = 5,
    CircleNotSettled = 6,
    InvalidAmount = 7,
    InvalidVolume = 8,
    DeadlinePassed = 9,
    DeadlineNotPassed = 10,
    QuorumNotReached = 11,
    OfferNotFound = 12,
    OfferMismatch = 13,
    AlreadyCommitted = 14,
    CommitmentNotFound = 15,
    AlreadyRefunded = 16,
    VolumeExceeded = 17,
    RegistryError = 18,
    PriceExceedsTarget = 19,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CircleStatus {
    Open = 0,
    QuorumReached = 1,
    Settled = 2,
    Cancelled = 3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum OfferStatus {
    Submitted = 0,
    Accepted = 1,
    Rejected = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DemandCircle {
    pub id: u64,
    pub creator: Address,
    pub token: Address,
    pub target_unit_price: i128,
    pub min_volume: u32,
    pub max_volume: u32,
    pub current_volume: u32,
    pub total_escrow: i128,
    pub deadline: u64,
    pub status: CircleStatus,
    pub registry_address: Option<Address>,
    pub registry_circle_id: Option<u64>,
    pub accepted_offer_id: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Commitment {
    pub buyer: Address,
    pub quantity: u32,
    pub amount: i128,
    pub refunded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SellerOffer {
    pub id: u64,
    pub seller: Address,
    pub circle_id: u64,
    pub unit_price: i128,
    pub volume: u32,
    pub lead_time_days: u32,
    pub status: OfferStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SellerReputation {
    pub seller: Address,
    pub successful_deals: u32,
    pub total_volume_settled: u64,
    pub total_amount_settled: i128,
    pub disputed_or_refunded_deals: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    CircleCounter,
    OfferCounter,
    Circle(u64),
    CircleOffers(u64),
    Commitment(u64, Address),
    Offer(u64),
    SellerReputation(Address),
}

#[contract]
pub struct BazaarDealEngineContract;

#[contractimpl]
impl BazaarDealEngineContract {
    /// Initialize the contract with an administrative address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CircleCounter, &0u64);
        env.storage().instance().set(&DataKey::OfferCounter, &0u64);
        Ok(())
    }

    /// Create a new buyer demand circle with volume thresholds, target unit price, and reservation deadline.
    pub fn create_circle(
        env: Env,
        creator: Address,
        token: Address,
        target_unit_price: i128,
        min_volume: u32,
        max_volume: u32,
        duration_seconds: u64,
    ) -> Result<u64, Error> {
        creator.require_auth();

        if target_unit_price <= 0 {
            return Err(Error::InvalidAmount);
        }
        if min_volume == 0 || max_volume < min_volume {
            return Err(Error::InvalidVolume);
        }

        let mut circle_counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CircleCounter)
            .unwrap_or(0);
        circle_counter += 1;

        let deadline = env.ledger().timestamp() + duration_seconds;

        let circle = DemandCircle {
            id: circle_counter,
            creator: creator.clone(),
            token,
            target_unit_price,
            min_volume,
            max_volume,
            current_volume: 0,
            total_escrow: 0,
            deadline,
            status: CircleStatus::Open,
            registry_address: None,
            registry_circle_id: None,
            accepted_offer_id: None,
        };

        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_counter), &circle);
        env.storage()
            .instance()
            .set(&DataKey::CircleCounter, &circle_counter);
        let empty_offers: Vec<u64> = vec![&env];
        env.storage()
            .instance()
            .set(&DataKey::CircleOffers(circle_counter), &empty_offers);

        env.events().publish(
            (symbol_short!("circle"), symbol_short!("created")),
            (circle_counter, creator),
        );

        Ok(circle_counter)
    }

    /// Genuine Inter-Contract Invocation:
    /// Register a deal by querying and verifying against an authoritative on-chain DemandCircleRegistry contract.
    pub fn create_deal_from_registry(
        env: Env,
        creator: Address,
        registry_address: Address,
        registry_circle_id: u64,
        token: Address,
        min_volume: u32,
        max_volume: u32,
    ) -> Result<u64, Error> {
        creator.require_auth();

        // Cross-contract call to DemandCircleRegistry
        let registry_client = DemandCircleRegistryClient::new(&env, &registry_address);
        let registry_circle = registry_client.get_circle(&registry_circle_id);

        if registry_circle.creator != creator {
            return Err(Error::Unauthorized);
        }

        if registry_circle.status != 0 {
            return Err(Error::CircleAlreadyClosed);
        }

        if env.ledger().timestamp() > registry_circle.deadline {
            return Err(Error::DeadlinePassed);
        }

        if min_volume == 0 || max_volume < min_volume {
            return Err(Error::InvalidVolume);
        }

        let mut circle_counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CircleCounter)
            .unwrap_or(0);
        circle_counter += 1;

        let circle = DemandCircle {
            id: circle_counter,
            creator: creator.clone(),
            token,
            target_unit_price: registry_circle.target_price_stroops,
            min_volume,
            max_volume,
            current_volume: 0,
            total_escrow: 0,
            deadline: registry_circle.deadline,
            status: CircleStatus::Open,
            registry_address: Some(registry_address.clone()),
            registry_circle_id: Some(registry_circle_id),
            accepted_offer_id: None,
        };

        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_counter), &circle);
        env.storage()
            .instance()
            .set(&DataKey::CircleCounter, &circle_counter);
        let empty_offers: Vec<u64> = vec![&env];
        env.storage()
            .instance()
            .set(&DataKey::CircleOffers(circle_counter), &empty_offers);

        env.events().publish(
            (symbol_short!("x_circle"), symbol_short!("linked")),
            (circle_counter, registry_address, registry_circle_id),
        );

        Ok(circle_counter)
    }

    /// Commit capital to a demand circle, transferring escrowed tokens to contract custody.
    pub fn commit_demand(
        env: Env,
        buyer: Address,
        circle_id: u64,
        quantity: u32,
    ) -> Result<(), Error> {
        buyer.require_auth();

        if quantity == 0 {
            return Err(Error::InvalidVolume);
        }

        let mut circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status != CircleStatus::Open && circle.status != CircleStatus::QuorumReached {
            return Err(Error::CircleAlreadyClosed);
        }

        if env.ledger().timestamp() > circle.deadline {
            return Err(Error::DeadlinePassed);
        }

        if circle.current_volume + quantity > circle.max_volume {
            return Err(Error::VolumeExceeded);
        }

        let commitment_key = DataKey::Commitment(circle_id, buyer.clone());
        if env.storage().instance().has(&commitment_key) {
            return Err(Error::AlreadyCommitted);
        }

        let total_cost = (quantity as i128) * circle.target_unit_price;

        // Escrow transfer from buyer to contract address
        let contract_address = env.current_contract_address();
        token::Client::new(&env, &circle.token).transfer(&buyer, &contract_address, &total_cost);

        circle.current_volume += quantity;
        circle.total_escrow += total_cost;

        if circle.current_volume >= circle.min_volume {
            circle.status = CircleStatus::QuorumReached;
        }

        let commitment = Commitment {
            buyer: buyer.clone(),
            quantity,
            amount: total_cost,
            refunded: false,
        };

        env.storage().instance().set(&commitment_key, &commitment);
        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_id), &circle);

        env.events().publish(
            (symbol_short!("demand"), symbol_short!("commit")),
            (circle_id, buyer, quantity, total_cost),
        );

        Ok(())
    }

    /// Submit a competitive seller quote/offer for an active demand circle.
    pub fn submit_seller_offer(
        env: Env,
        seller: Address,
        circle_id: u64,
        unit_price: i128,
        volume: u32,
        lead_time_days: u32,
    ) -> Result<u64, Error> {
        seller.require_auth();

        if unit_price <= 0 {
            return Err(Error::InvalidAmount);
        }
        if volume == 0 {
            return Err(Error::InvalidVolume);
        }

        let circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status == CircleStatus::Settled || circle.status == CircleStatus::Cancelled {
            return Err(Error::CircleAlreadyClosed);
        }

        if env.ledger().timestamp() > circle.deadline {
            return Err(Error::DeadlinePassed);
        }

        if unit_price > circle.target_unit_price {
            return Err(Error::PriceExceedsTarget);
        }

        let mut offer_counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::OfferCounter)
            .unwrap_or(0);
        offer_counter += 1;

        let offer = SellerOffer {
            id: offer_counter,
            seller: seller.clone(),
            circle_id,
            unit_price,
            volume,
            lead_time_days,
            status: OfferStatus::Submitted,
        };

        env.storage()
            .instance()
            .set(&DataKey::Offer(offer_counter), &offer);
        env.storage()
            .instance()
            .set(&DataKey::OfferCounter, &offer_counter);

        // Append to circle offers list
        let mut circle_offers: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::CircleOffers(circle_id))
            .unwrap_or_else(|| vec![&env]);
        circle_offers.push_back(offer_counter);
        env.storage()
            .instance()
            .set(&DataKey::CircleOffers(circle_id), &circle_offers);

        env.events().publish(
            (symbol_short!("offer"), symbol_short!("submit")),
            (circle_id, offer_counter, seller),
        );

        Ok(offer_counter)
    }

    /// Accept a seller offer for an active circle that reached quorum.
    pub fn accept_seller_offer(
        env: Env,
        caller: Address,
        circle_id: u64,
        offer_id: u64,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.creator != caller {
            return Err(Error::Unauthorized);
        }

        if circle.status != CircleStatus::QuorumReached && circle.status != CircleStatus::Open {
            return Err(Error::CircleAlreadyClosed);
        }

        let mut offer: SellerOffer = env
            .storage()
            .instance()
            .get(&DataKey::Offer(offer_id))
            .ok_or(Error::OfferNotFound)?;

        if offer.circle_id != circle_id {
            return Err(Error::OfferMismatch);
        }

        if offer.status != OfferStatus::Submitted {
            return Err(Error::CircleAlreadyClosed);
        }

        offer.status = OfferStatus::Accepted;
        circle.accepted_offer_id = Some(offer_id);

        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage()
            .instance()
            .set(&DataKey::Offer(offer_id), &offer);

        env.events().publish(
            (symbol_short!("offer"), symbol_short!("accept")),
            (circle_id, offer_id, caller),
        );

        Ok(())
    }

    /// Settle a demand circle with a winning seller offer, releasing escrowed capital and recording verifiable reputation.
    pub fn settle_deal(env: Env, circle_id: u64, winning_offer_id: u64) -> Result<(), Error> {
        let mut circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status != CircleStatus::QuorumReached {
            return Err(Error::QuorumNotReached);
        }

        let mut offer: SellerOffer = env
            .storage()
            .instance()
            .get(&DataKey::Offer(winning_offer_id))
            .ok_or(Error::OfferNotFound)?;

        if offer.circle_id != circle_id {
            return Err(Error::OfferMismatch);
        }

        if offer.volume < circle.current_volume {
            return Err(Error::InvalidVolume);
        }

        // Calculate payout: volume * winning offer unit price
        let payout = (circle.current_volume as i128) * offer.unit_price;
        if payout > circle.total_escrow {
            return Err(Error::InvalidAmount);
        }

        // Transfer escrowed funds from contract to winning seller
        token::Client::new(&env, &circle.token).transfer(
            &env.current_contract_address(),
            &offer.seller,
            &payout,
        );

        offer.status = OfferStatus::Accepted;
        circle.status = CircleStatus::Settled;
        circle.accepted_offer_id = Some(winning_offer_id);

        // Update on-chain verifiable seller reputation
        let rep_key = DataKey::SellerReputation(offer.seller.clone());
        let mut rep: SellerReputation = env
            .storage()
            .instance()
            .get(&rep_key)
            .unwrap_or(SellerReputation {
                seller: offer.seller.clone(),
                successful_deals: 0,
                total_volume_settled: 0,
                total_amount_settled: 0,
                disputed_or_refunded_deals: 0,
            });
        rep.successful_deals += 1;
        rep.total_volume_settled += circle.current_volume as u64;
        rep.total_amount_settled += payout;
        env.storage().instance().set(&rep_key, &rep);

        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage()
            .instance()
            .set(&DataKey::Offer(winning_offer_id), &offer);

        env.events().publish(
            (symbol_short!("deal"), symbol_short!("settled")),
            (circle_id, winning_offer_id, payout),
        );

        Ok(())
    }

    /// Claim refund for a committed buyer if circle expired without settlement or was cancelled.
    pub fn claim_refund(env: Env, buyer: Address, circle_id: u64) -> Result<(), Error> {
        buyer.require_auth();

        let circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        let is_expired_unsettled =
            env.ledger().timestamp() > circle.deadline && circle.status != CircleStatus::Settled;
        let is_cancelled = circle.status == CircleStatus::Cancelled;

        if !is_expired_unsettled && !is_cancelled {
            return Err(Error::CircleAlreadyClosed);
        }

        let commitment_key = DataKey::Commitment(circle_id, buyer.clone());
        let mut commitment: Commitment = env
            .storage()
            .instance()
            .get(&commitment_key)
            .ok_or(Error::CommitmentNotFound)?;

        if commitment.refunded {
            return Err(Error::AlreadyRefunded);
        }

        // Return exact escrowed tokens to buyer
        token::Client::new(&env, &circle.token).transfer(
            &env.current_contract_address(),
            &buyer,
            &commitment.amount,
        );

        commitment.refunded = true;
        env.storage().instance().set(&commitment_key, &commitment);

        env.events().publish(
            (symbol_short!("refund"), symbol_short!("claimed")),
            (circle_id, buyer, commitment.amount),
        );

        Ok(())
    }

    /// Creator cancellation of an open circle before settlement, releasing commitments for refund.
    pub fn cancel_circle(env: Env, caller: Address, circle_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.creator != caller {
            return Err(Error::Unauthorized);
        }

        if circle.status == CircleStatus::Settled {
            return Err(Error::CircleAlreadyClosed);
        }

        circle.status = CircleStatus::Cancelled;
        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_id), &circle);

        env.events().publish(
            (symbol_short!("circle"), symbol_short!("cancel")),
            (circle_id, caller),
        );

        Ok(())
    }

    /// Query a demand circle by ID.
    pub fn get_circle(env: Env, circle_id: u64) -> Result<DemandCircle, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)
    }

    /// Total registered circles count.
    pub fn get_circle_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::CircleCounter)
            .unwrap_or(0)
    }

    /// Query a buyer's commitment for a given circle.
    pub fn get_commitment(env: Env, circle_id: u64, buyer: Address) -> Result<Commitment, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Commitment(circle_id, buyer))
            .ok_or(Error::CommitmentNotFound)
    }

    /// Query a seller offer by ID.
    pub fn get_offer(env: Env, offer_id: u64) -> Result<SellerOffer, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Offer(offer_id))
            .ok_or(Error::OfferNotFound)
    }

    /// Query all offer IDs submitted for a given circle.
    pub fn get_circle_offers(env: Env, circle_id: u64) -> Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::CircleOffers(circle_id))
            .unwrap_or_else(|| vec![&env])
    }

    /// Query verifiable transaction-outcome-derived seller reputation.
    pub fn get_seller_reputation(env: Env, seller: Address) -> SellerReputation {
        env.storage()
            .instance()
            .get(&DataKey::SellerReputation(seller.clone()))
            .unwrap_or(SellerReputation {
                seller,
                successful_deals: 0,
                total_volume_settled: 0,
                total_amount_settled: 0,
                disputed_or_refunded_deals: 0,
            })
    }
}

#[cfg(test)]
mod test;
