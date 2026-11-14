#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, symbol_short, token, vec,
    Address, Env, String, Symbol, Vec,
};

/// Minimum ledgers remaining before instance TTL is extended (~1 day at 5s/ledger).
pub const INSTANCE_TTL_THRESHOLD: u32 = 17_280;
/// Amount of ledgers to extend instance TTL to (~30 days at 5s/ledger).
pub const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

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
    DuplicateOffer = 20,
    NotUnderInspection = 21,
    DisputeAlreadyOpen = 22,
    DisputeNotOpen = 23,
    DisputeUnauthorized = 24,
    InvalidFeeBps = 25,
    NoSurplusRebate = 26,
    AlreadyClaimedRebate = 27,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CircleStatus {
    Open = 0,
    QuorumReached = 1,
    Settled = 2,
    Cancelled = 3,
    Inspection = 4,
    Disputed = 5,
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
    pub inspection_deadline: Option<u64>,
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
    pub seller_bond_stroops: i128,
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
    Treasury,
    ProtocolFeeBps,
    RebateClaimed(u64, Address),
    SellerBondBps,
    EscrowedSellerBond(u64),
    SlashedSellerBond(u64),
    SlashedBondDistributed(u64),
    CircleRefundedVolume(u64),
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
        env.storage().instance().set(&DataKey::Treasury, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ProtocolFeeBps, &100u32);
        env.storage().instance().set(&DataKey::CircleCounter, &0u64);
        env.storage().instance().set(&DataKey::OfferCounter, &0u64);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
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
            inspection_deadline: None,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle_counter), &circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_counter),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .instance()
            .set(&DataKey::CircleCounter, &circle_counter);
        let empty_offers: Vec<u64> = vec![&env];
        env.storage()
            .persistent()
            .set(&DataKey::CircleOffers(circle_counter), &empty_offers);
        env.storage().persistent().extend_ttl(
            &DataKey::CircleOffers(circle_counter),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);

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
            inspection_deadline: None,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle_counter), &circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_counter),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .instance()
            .set(&DataKey::CircleCounter, &circle_counter);
        let empty_offers: Vec<u64> = vec![&env];
        env.storage()
            .persistent()
            .set(&DataKey::CircleOffers(circle_counter), &empty_offers);
        env.storage().persistent().extend_ttl(
            &DataKey::CircleOffers(circle_counter),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);

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
            .persistent()
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
        if env.storage().persistent().has(&commitment_key) {
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

        env.storage().persistent().set(&commitment_key, &commitment);
        env.storage().persistent().extend_ttl(
            &commitment_key,
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

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
            .persistent()
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

        // Validate that this seller has not already submitted an active offer for this circle
        let existing_offers: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CircleOffers(circle_id))
            .unwrap_or_else(|| vec![&env]);
        for off_id in existing_offers.iter() {
            if let Some(existing) = env
                .storage()
                .persistent()
                .get::<DataKey, SellerOffer>(&DataKey::Offer(off_id))
            {
                if existing.seller == seller && existing.status == OfferStatus::Submitted {
                    return Err(Error::DuplicateOffer);
                }
            }
        }

        let mut offer_counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::OfferCounter)
            .unwrap_or(0);
        offer_counter += 1;

        let bond_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SellerBondBps)
            .unwrap_or(0);
        let total_offer_value = unit_price * (volume as i128);
        let seller_bond_stroops = (total_offer_value * (bond_bps as i128)) / 10_000;

        let offer = SellerOffer {
            id: offer_counter,
            seller: seller.clone(),
            circle_id,
            unit_price,
            volume,
            lead_time_days,
            seller_bond_stroops,
            status: OfferStatus::Submitted,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Offer(offer_counter), &offer);
        env.storage().persistent().extend_ttl(
            &DataKey::Offer(offer_counter),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .instance()
            .set(&DataKey::OfferCounter, &offer_counter);

        // Append to circle offers list
        let mut circle_offers: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::CircleOffers(circle_id))
            .unwrap_or_else(|| vec![&env]);
        circle_offers.push_back(offer_counter);
        env.storage()
            .persistent()
            .set(&DataKey::CircleOffers(circle_id), &circle_offers);
        env.storage().persistent().extend_ttl(
            &DataKey::CircleOffers(circle_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        env.events().publish(
            (symbol_short!("offer"), symbol_short!("submit")),
            (circle_id, offer_counter, seller),
        );

        Ok(offer_counter)
    }

    /// Submit a competitive seller quote with an explicit performance bond amount in stroops.
    pub fn submit_seller_offer_with_bond(
        env: Env,
        seller: Address,
        circle_id: u64,
        unit_price: i128,
        volume: u32,
        lead_time_days: u32,
        seller_bond_stroops: i128,
    ) -> Result<u64, Error> {
        if seller_bond_stroops < 0 {
            return Err(Error::InvalidAmount);
        }
        let offer_id = Self::submit_seller_offer(
            env.clone(),
            seller,
            circle_id,
            unit_price,
            volume,
            lead_time_days,
        )?;
        let mut offer: SellerOffer = env
            .storage()
            .persistent()
            .get(&DataKey::Offer(offer_id))
            .ok_or(Error::OfferNotFound)?;
        offer.seller_bond_stroops = seller_bond_stroops;
        env.storage()
            .persistent()
            .set(&DataKey::Offer(offer_id), &offer);
        env.storage().persistent().extend_ttl(
            &DataKey::Offer(offer_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        Ok(offer_id)
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
            .persistent()
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
            .persistent()
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

        let bond_amount = offer.seller_bond_stroops;
        if bond_amount > 0 {
            let bond_key = DataKey::EscrowedSellerBond(circle_id);
            env.storage().persistent().set(&bond_key, &bond_amount);
            env.storage().persistent().extend_ttl(
                &bond_key,
                INSTANCE_TTL_THRESHOLD,
                INSTANCE_TTL_EXTEND_TO,
            );
        }

        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .persistent()
            .set(&DataKey::Offer(offer_id), &offer);
        env.storage().persistent().extend_ttl(
            &DataKey::Offer(offer_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        if bond_amount > 0 {
            offer.seller.require_auth();
            let contract_address = env.current_contract_address();
            token::Client::new(&env, &circle.token).transfer(
                &offer.seller,
                &contract_address,
                &bond_amount,
            );
            env.events().publish(
                (symbol_short!("bond"), symbol_short!("locked")),
                (circle_id, offer_id, offer.seller.clone(), bond_amount),
            );
        }

        env.events().publish(
            (symbol_short!("offer"), symbol_short!("accept")),
            (circle_id, offer_id, caller),
        );

        Ok(())
    }

    /// Initiates a commercial delivery and inspection period for an accepted deal.
    pub fn initiate_inspection(
        env: Env,
        caller: Address,
        circle_id: u64,
        inspection_duration_seconds: u64,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.creator != caller {
            return Err(Error::Unauthorized);
        }

        if circle.status != CircleStatus::QuorumReached {
            return Err(Error::QuorumNotReached);
        }

        if circle.accepted_offer_id.is_none() {
            return Err(Error::OfferNotFound);
        }

        circle.status = CircleStatus::Inspection;
        let inspection_deadline = env.ledger().timestamp() + inspection_duration_seconds;
        circle.inspection_deadline = Some(inspection_deadline);

        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        env.events().publish(
            (symbol_short!("inspect"), symbol_short!("start")),
            (circle_id, caller, inspection_deadline),
        );

        Ok(())
    }

    /// Confirms delivery and releases escrow settlement to the seller following inspection.
    pub fn confirm_inspection_and_settle(
        env: Env,
        caller: Address,
        circle_id: u64,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.creator != caller {
            return Err(Error::Unauthorized);
        }

        if circle.status != CircleStatus::Inspection {
            return Err(Error::NotUnderInspection);
        }

        let winning_offer_id = circle.accepted_offer_id.ok_or(Error::OfferNotFound)?;
        let offer: SellerOffer = env
            .storage()
            .persistent()
            .get(&DataKey::Offer(winning_offer_id))
            .ok_or(Error::OfferNotFound)?;

        Self::settle_deal_internal(&env, &mut circle, &offer, winning_offer_id)
    }

    /// Opens a formal commercial dispute during inspection or post-quorum negotiation.
    /// Can be initiated by circle creator or any buyer who committed capital.
    pub fn dispute_deal(
        env: Env,
        caller: Address,
        circle_id: u64,
        reason: Symbol,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status == CircleStatus::Disputed {
            return Err(Error::DisputeAlreadyOpen);
        }

        if circle.status != CircleStatus::Inspection && circle.status != CircleStatus::QuorumReached
        {
            return Err(Error::NotUnderInspection);
        }

        // Caller must be creator or have an active commitment
        let is_creator = circle.creator == caller;
        let is_buyer = env
            .storage()
            .persistent()
            .has(&DataKey::Commitment(circle_id, caller.clone()));

        if !is_creator && !is_buyer {
            return Err(Error::DisputeUnauthorized);
        }

        circle.status = CircleStatus::Disputed;
        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        env.events().publish(
            (symbol_short!("dispute"), symbol_short!("opened")),
            (circle_id, caller, reason),
        );

        Ok(())
    }

    /// Resolves an open dispute through administrative arbitration.
    /// - If refund_to_buyers is true: circle status is set to Cancelled,
    ///   allowing buyers to claim 100% refund, and seller's disputed reputation is updated.
    /// - If refund_to_buyers is false: releases escrow funds to seller via settle_deal.
    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        circle_id: u64,
        refund_to_buyers: bool,
    ) -> Result<(), Error> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if caller != admin {
            return Err(Error::Unauthorized);
        }

        let mut circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status != CircleStatus::Disputed {
            return Err(Error::DisputeNotOpen);
        }

        if refund_to_buyers {
            circle.status = CircleStatus::Cancelled;

            // Slash any escrowed seller performance bond for pro-rata buyer compensation
            let bond_key = DataKey::EscrowedSellerBond(circle_id);
            let escrowed_bond: i128 = env.storage().persistent().get(&bond_key).unwrap_or(0);
            if escrowed_bond > 0 {
                env.storage().persistent().set(&bond_key, &0i128);
                let slashed_key = DataKey::SlashedSellerBond(circle_id);
                env.storage().persistent().set(&slashed_key, &escrowed_bond);
                env.storage().persistent().extend_ttl(
                    &slashed_key,
                    INSTANCE_TTL_THRESHOLD,
                    INSTANCE_TTL_EXTEND_TO,
                );
                env.events().publish(
                    (symbol_short!("bond"), symbol_short!("slashed")),
                    (circle_id, escrowed_bond),
                );
            }

            // Update seller reputation to record dispute/refund
            if let Some(offer_id) = circle.accepted_offer_id {
                if let Some(offer) = env
                    .storage()
                    .persistent()
                    .get::<_, SellerOffer>(&DataKey::Offer(offer_id))
                {
                    let rep_key = DataKey::SellerReputation(offer.seller.clone());
                    let mut rep: SellerReputation = env
                        .storage()
                        .persistent()
                        .get(&rep_key)
                        .unwrap_or(SellerReputation {
                            seller: offer.seller.clone(),
                            successful_deals: 0,
                            total_volume_settled: 0,
                            total_amount_settled: 0,
                            disputed_or_refunded_deals: 0,
                        });
                    rep.disputed_or_refunded_deals += 1;
                    env.storage().persistent().set(&rep_key, &rep);
                    env.storage().persistent().extend_ttl(
                        &rep_key,
                        INSTANCE_TTL_THRESHOLD,
                        INSTANCE_TTL_EXTEND_TO,
                    );
                }
            }

            env.storage()
                .persistent()
                .set(&DataKey::Circle(circle_id), &circle);
            env.storage().persistent().extend_ttl(
                &DataKey::Circle(circle_id),
                INSTANCE_TTL_THRESHOLD,
                INSTANCE_TTL_EXTEND_TO,
            );

            env.events().publish(
                (symbol_short!("dispute"), symbol_short!("resolve")),
                (circle_id, caller, true),
            );

            Ok(())
        } else {
            let winning_offer_id = circle.accepted_offer_id.ok_or(Error::OfferNotFound)?;
            let offer: SellerOffer = env
                .storage()
                .persistent()
                .get(&DataKey::Offer(winning_offer_id))
                .ok_or(Error::OfferNotFound)?;
            Self::settle_deal_internal(&env, &mut circle, &offer, winning_offer_id)
        }
    }

    /// Settle a demand circle with a winning seller offer.
    ///
    /// Follows the Checks-Effects-Interactions (CEI) security pattern:
    /// 1. CHECKS: Validate caller authorization, circle quorum, accepted offer validity, and escrow balance.
    /// 2. EFFECTS: Update on-chain states (circle, offer, seller reputation) before token transfer.
    /// 3. INTERACTIONS: Transfer escrowed SAC tokens to the seller last to eliminate re-entrancy risks.
    pub fn settle_deal(
        env: Env,
        caller: Address,
        circle_id: u64,
        winning_offer_id: u64,
    ) -> Result<(), Error> {
        caller.require_auth();

        // --- 1. CHECKS ---
        let mut circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status == CircleStatus::Disputed {
            return Err(Error::DisputeAlreadyOpen);
        }

        if circle.status != CircleStatus::QuorumReached && circle.status != CircleStatus::Inspection
        {
            return Err(Error::QuorumNotReached);
        }

        let winning_id = circle.accepted_offer_id.ok_or(Error::OfferNotFound)?;
        if winning_id != winning_offer_id {
            return Err(Error::OfferMismatch);
        }

        let offer: SellerOffer = env
            .storage()
            .persistent()
            .get(&DataKey::Offer(winning_offer_id))
            .ok_or(Error::OfferNotFound)?;

        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);

        let is_creator = caller == circle.creator;
        let is_admin = admin.as_ref() == Some(&caller);
        let is_seller_after_inspection = caller == offer.seller
            && circle.status == CircleStatus::Inspection
            && circle.inspection_deadline.is_some()
            && env.ledger().timestamp() >= circle.inspection_deadline.unwrap();

        if !is_creator && !is_admin && !is_seller_after_inspection {
            return Err(Error::Unauthorized);
        }

        Self::settle_deal_internal(&env, &mut circle, &offer, winning_offer_id)
    }

    fn settle_deal_internal(
        env: &Env,
        circle: &mut DemandCircle,
        offer: &SellerOffer,
        winning_offer_id: u64,
    ) -> Result<(), Error> {
        if offer.volume < circle.current_volume {
            return Err(Error::InvalidVolume);
        }

        // Calculate payout: volume * winning offer unit price
        let payout = (circle.current_volume as i128) * offer.unit_price;
        if payout > circle.total_escrow {
            return Err(Error::InvalidAmount);
        }

        let fee_bps = env
            .storage()
            .instance()
            .get(&DataKey::ProtocolFeeBps)
            .unwrap_or(0u32);
        let treasury: Option<Address> = env.storage().instance().get(&DataKey::Treasury);

        let (seller_payout, protocol_fee) =
            bazaar_math::calculate_net_payout(payout, fee_bps).unwrap_or((payout, 0));

        let bond_key = DataKey::EscrowedSellerBond(circle.id);
        let bond_returned: i128 = env.storage().persistent().get(&bond_key).unwrap_or(0);

        // --- 2. EFFECTS (State mutations before external transfer) ---
        if bond_returned > 0 {
            env.storage().persistent().set(&bond_key, &0i128);
            env.storage().persistent().extend_ttl(
                &bond_key,
                INSTANCE_TTL_THRESHOLD,
                INSTANCE_TTL_EXTEND_TO,
            );
        }
        let mut mut_offer = offer.clone();
        mut_offer.status = OfferStatus::Accepted;
        circle.status = CircleStatus::Settled;
        circle.accepted_offer_id = Some(winning_offer_id);

        // Update on-chain verifiable seller reputation
        let rep_key = DataKey::SellerReputation(offer.seller.clone());
        let mut rep: SellerReputation =
            env.storage()
                .persistent()
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
        env.storage().persistent().set(&rep_key, &rep);
        env.storage().persistent().extend_ttl(
            &rep_key,
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle.id), circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle.id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        env.storage()
            .persistent()
            .set(&DataKey::Offer(winning_offer_id), &mut_offer);
        env.storage().persistent().extend_ttl(
            &DataKey::Offer(winning_offer_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        // --- 3. INTERACTIONS (External token transfer performed last) ---
        let token_client = token::Client::new(env, &circle.token);

        if protocol_fee > 0 {
            if let Some(ref treasury_addr) = treasury {
                token_client.transfer(
                    &env.current_contract_address(),
                    treasury_addr,
                    &protocol_fee,
                );
            }
        }

        let total_seller_release = seller_payout + bond_returned;
        token_client.transfer(
            &env.current_contract_address(),
            &offer.seller,
            &total_seller_release,
        );

        if bond_returned > 0 {
            env.events().publish(
                (symbol_short!("bond"), symbol_short!("returned")),
                (circle.id, winning_offer_id, offer.seller.clone(), bond_returned),
            );
        }

        env.events().publish(
            (symbol_short!("deal"), symbol_short!("settled")),
            (circle.id, winning_offer_id, seller_payout),
        );

        Ok(())
    }

    /// Claim refund for a committed buyer if circle expired without settlement or was cancelled.
    ///
    /// Follows the Checks-Effects-Interactions (CEI) security pattern:
    /// 1. CHECKS: Verify expiration or cancellation status, and buyer commitment validity.
    /// 2. EFFECTS: Flag commitment as refunded in storage prior to token transfer.
    /// 3. INTERACTIONS: Transfer escrowed tokens back to the buyer address last.
    pub fn claim_refund(env: Env, buyer: Address, circle_id: u64) -> Result<(), Error> {
        buyer.require_auth();

        // --- 1. CHECKS ---
        let circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        let is_expired_unsettled = env.ledger().timestamp() > circle.deadline
            && (circle.status == CircleStatus::Open
                || circle.status == CircleStatus::QuorumReached);
        let is_cancelled = circle.status == CircleStatus::Cancelled;

        if !is_expired_unsettled && !is_cancelled {
            return Err(Error::CircleAlreadyClosed);
        }

        let commitment_key = DataKey::Commitment(circle_id, buyer.clone());
        let mut commitment: Commitment = env
            .storage()
            .persistent()
            .get(&commitment_key)
            .ok_or(Error::CommitmentNotFound)?;

        if commitment.refunded {
            return Err(Error::AlreadyRefunded);
        }

        // Calculate pro-rata share of any slashed seller bond (with zero-dust final claim)
        let slashed_bond: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::SlashedSellerBond(circle_id))
            .unwrap_or(0);
        let mut bond_compensation: i128 = 0;

        // --- 2. EFFECTS ---
        if is_cancelled && slashed_bond > 0 && circle.current_volume > 0 {
            let vol_key = DataKey::CircleRefundedVolume(circle_id);
            let dist_key = DataKey::SlashedBondDistributed(circle_id);
            let prev_refunded_vol: u32 = env.storage().persistent().get(&vol_key).unwrap_or(0);
            let prev_distributed: i128 = env.storage().persistent().get(&dist_key).unwrap_or(0);
            let next_refunded_vol = prev_refunded_vol + commitment.quantity;

            if next_refunded_vol >= circle.current_volume {
                bond_compensation = (slashed_bond - prev_distributed).max(0);
            } else {
                let buyer_qty = commitment.quantity as i128;
                let total_vol = circle.current_volume as i128;
                bond_compensation = (slashed_bond * buyer_qty) / total_vol;
            }

            let next_distributed = prev_distributed + bond_compensation;
            env.storage().persistent().set(&vol_key, &next_refunded_vol);
            env.storage().persistent().extend_ttl(
                &vol_key,
                INSTANCE_TTL_THRESHOLD,
                INSTANCE_TTL_EXTEND_TO,
            );
            env.storage().persistent().set(&dist_key, &next_distributed);
            env.storage().persistent().extend_ttl(
                &dist_key,
                INSTANCE_TTL_THRESHOLD,
                INSTANCE_TTL_EXTEND_TO,
            );
        }

        commitment.refunded = true;
        env.storage().persistent().set(&commitment_key, &commitment);
        env.storage().persistent().extend_ttl(
            &commitment_key,
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        // --- 3. INTERACTIONS ---
        let total_refund = commitment.amount + bond_compensation;
        token::Client::new(&env, &circle.token).transfer(
            &env.current_contract_address(),
            &buyer,
            &total_refund,
        );

        env.events().publish(
            (symbol_short!("refund"), symbol_short!("claimed")),
            (circle_id, buyer, total_refund),
        );

        Ok(())
    }

    /// Creator cancellation of an open circle before settlement, releasing commitments for refund.
    pub fn cancel_circle(env: Env, caller: Address, circle_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.creator != caller {
            return Err(Error::Unauthorized);
        }

        if circle.status == CircleStatus::Settled || circle.status == CircleStatus::Cancelled {
            return Err(Error::CircleAlreadyClosed);
        }

        if circle.status == CircleStatus::Inspection || circle.status == CircleStatus::Disputed {
            return Err(Error::CircleAlreadyClosed);
        }

        if circle.accepted_offer_id.is_some() {
            return Err(Error::CircleAlreadyClosed);
        }

        circle.status = CircleStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        env.events().publish(
            (symbol_short!("circle"), symbol_short!("cancel")),
            (circle_id, caller),
        );

        Ok(())
    }

    /// Claim surplus discount rebate for a buyer in a settled demand circle.
    /// When the accepted seller quote is lower than the circle target unit price,
    /// committed buyers are entitled to their pro-rata price savings.
    pub fn claim_surplus_rebate(env: Env, buyer: Address, circle_id: u64) -> Result<(), Error> {
        buyer.require_auth();

        // --- 1. CHECKS ---
        let circle: DemandCircle = env
            .storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status != CircleStatus::Settled {
            return Err(Error::CircleNotSettled);
        }

        let rebate_key = DataKey::RebateClaimed(circle_id, buyer.clone());
        if env.storage().persistent().has(&rebate_key) {
            return Err(Error::AlreadyClaimedRebate);
        }

        let commitment_key = DataKey::Commitment(circle_id, buyer.clone());
        let commitment: Commitment = env
            .storage()
            .persistent()
            .get(&commitment_key)
            .ok_or(Error::CommitmentNotFound)?;

        let winning_offer_id = circle.accepted_offer_id.ok_or(Error::OfferNotFound)?;
        let offer: SellerOffer = env
            .storage()
            .persistent()
            .get(&DataKey::Offer(winning_offer_id))
            .ok_or(Error::OfferNotFound)?;

        if offer.unit_price >= circle.target_unit_price {
            return Err(Error::NoSurplusRebate);
        }

        let rebate_amount = bazaar_math::calculate_surplus_rebate(
            circle.target_unit_price,
            offer.unit_price,
            commitment.quantity,
        )
        .map_err(|_| Error::InvalidAmount)?;

        if rebate_amount <= 0 {
            return Err(Error::NoSurplusRebate);
        }

        // --- 2. EFFECTS ---
        env.storage().persistent().set(&rebate_key, &true);
        env.storage().persistent().extend_ttl(
            &rebate_key,
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );

        // --- 3. INTERACTIONS ---
        token::Client::new(&env, &circle.token).transfer(
            &env.current_contract_address(),
            &buyer,
            &rebate_amount,
        );

        env.events().publish(
            (symbol_short!("rebate"), symbol_short!("claimed")),
            (circle_id, buyer, rebate_amount),
        );

        Ok(())
    }

    /// Query a demand circle by ID.
    pub fn get_circle(env: Env, circle_id: u64) -> Result<DemandCircle, Error> {
        env.storage().persistent().extend_ttl(
            &DataKey::Circle(circle_id),
            INSTANCE_TTL_THRESHOLD,
            INSTANCE_TTL_EXTEND_TO,
        );
        env.storage()
            .persistent()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)
    }

    /// Total registered circles count.
    pub fn get_circle_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .instance()
            .get(&DataKey::CircleCounter)
            .unwrap_or(0)
    }

    /// Query a buyer's commitment for a given circle.
    pub fn get_commitment(env: Env, circle_id: u64, buyer: Address) -> Result<Commitment, Error> {
        let key = DataKey::Commitment(circle_id, buyer);
        env.storage()
            .persistent()
            .extend_ttl(&key, INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(Error::CommitmentNotFound)
    }

    /// Query a seller offer by ID.
    pub fn get_offer(env: Env, offer_id: u64) -> Result<SellerOffer, Error> {
        let key = DataKey::Offer(offer_id);
        env.storage()
            .persistent()
            .extend_ttl(&key, INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(Error::OfferNotFound)
    }

    /// Query all offer IDs submitted for a given circle.
    pub fn get_circle_offers(env: Env, circle_id: u64) -> Vec<u64> {
        let key = DataKey::CircleOffers(circle_id);
        env.storage()
            .persistent()
            .extend_ttl(&key, INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| vec![&env])
    }

    /// Query verifiable transaction-outcome-derived seller reputation.
    pub fn get_seller_reputation(env: Env, seller: Address) -> SellerReputation {
        let key = DataKey::SellerReputation(seller.clone());
        env.storage()
            .persistent()
            .extend_ttl(&key, INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(SellerReputation {
                seller,
                successful_deals: 0,
                total_volume_settled: 0,
                total_amount_settled: 0,
                disputed_or_refunded_deals: 0,
            })
    }

    /// Update the protocol fee in basis points (max 1,000 bps / 10%).
    pub fn set_protocol_fee(env: Env, caller: Address, fee_bps: u32) -> Result<(), Error> {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != admin {
            return Err(Error::Unauthorized);
        }
        if fee_bps > 1_000 {
            return Err(Error::InvalidFeeBps);
        }
        env.storage()
            .instance()
            .set(&DataKey::ProtocolFeeBps, &fee_bps);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.events().publish(
            (symbol_short!("fee"), symbol_short!("update")),
            (caller, fee_bps),
        );
        Ok(())
    }

    /// Update the protocol treasury address.
    pub fn set_treasury(env: Env, caller: Address, treasury: Address) -> Result<(), Error> {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != admin {
            return Err(Error::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.events().publish(
            (symbol_short!("treasury"), symbol_short!("update")),
            (caller, treasury),
        );
        Ok(())
    }

    /// Returns the currently active protocol fee in basis points.
    pub fn get_protocol_fee_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ProtocolFeeBps)
            .unwrap_or(0)
    }

    /// Returns the active protocol treasury address if configured.
    pub fn get_treasury(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Treasury)
    }

    /// Configure the default seller security performance bond in basis points (max 5,000 bps / 50%).
    pub fn set_seller_bond_bps(env: Env, caller: Address, bond_bps: u32) -> Result<(), Error> {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != admin {
            return Err(Error::Unauthorized);
        }
        if bond_bps > 5_000 {
            return Err(Error::InvalidFeeBps);
        }
        env.storage()
            .instance()
            .set(&DataKey::SellerBondBps, &bond_bps);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.events().publish(
            (symbol_short!("bond_bps"), symbol_short!("update")),
            (caller, bond_bps),
        );
        Ok(())
    }

    /// Returns the currently configured seller performance bond in basis points.
    pub fn get_seller_bond_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::SellerBondBps)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod e2e_tests;
