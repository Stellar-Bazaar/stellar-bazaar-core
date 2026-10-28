#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

/// Minimum ledgers remaining before instance TTL is extended (~1 day at 5s/ledger).
pub const INSTANCE_TTL_THRESHOLD: u32 = 17_280;
/// Amount of ledgers to extend instance TTL to (~30 days at 5s/ledger).
pub const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Contract has already been initialized with an administrative address.
    AlreadyInitialized = 1,
    /// Contract administrative address or state has not been initialized.
    NotInitialized = 2,
    /// Invoker lacks necessary authorization to execute this operation.
    Unauthorized = 3,
    /// Requested demand circle ID was not found in registry storage.
    CircleNotFound = 4,
    /// Target order quantity must be greater than zero.
    InvalidQuantity = 5,
    /// Target unit price must be strictly positive in Stroops.
    InvalidPrice = 6,
    /// Duration must be between 60 seconds and 31,536,000 seconds (1 year).
    InvalidDeadline = 7,
    /// Title cannot be empty and must not exceed 64 bytes.
    InvalidTitle = 8,
    /// Operation cannot be performed given the current lifecycle status of the circle.
    InvalidState = 9,
    /// Expiration cannot be triggered before the block deadline timestamp has passed.
    DeadlineNotPassed = 10,
    /// Action cannot be performed because circle deadline has already elapsed.
    DeadlinePassed = 11,
    /// Metadata URI exceeds the maximum permitted length of 256 bytes or is empty.
    InvalidMetadataUri = 12,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CircleStatus {
    Open = 0,
    Closed = 1,
    Expired = 2,
    Cancelled = 3,
}

/// Authoritative on-chain record for a commercial demand circle on Stellar.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DemandCircle {
    /// Unique sequential identifier assigned on-chain.
    pub id: u64,
    /// Stellar account address that established this demand circle.
    pub creator: Address,
    /// Short human-readable title (up to 64 bytes).
    pub title: String,
    /// Decentralized metadata URI (e.g., IPFS content identifier) containing product specifications (max 256 bytes).
    pub metadata_uri: String,
    /// Minimum aggregate unit volume required to reach collective purchasing quorum.
    pub target_quantity: u32,
    /// Target maximum acceptable unit price in Stroops (10,000,000 Stroops = 1 XLM).
    pub target_price_stroops: i128,
    /// Unix timestamp (in seconds) after which commitments can no longer be accepted.
    pub deadline: u64,
    /// Ledger Unix timestamp at the moment this circle was initialized.
    pub created_at: u64,
    /// Current lifecycle phase of the circle (Open, Closed, Expired, Cancelled).
    pub status: CircleStatus,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    CircleCounter,
    Circle(u64),
}

#[contract]
pub struct DemandCircleRegistryContract;

#[contractimpl]
impl DemandCircleRegistryContract {
    /// Initializes the registry with an administrative address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CircleCounter, &0u64);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        Ok(())
    }

    /// Creates a new demand circle on-chain with validated commercial constraints.
    pub fn create_circle(
        env: Env,
        creator: Address,
        title: String,
        metadata_uri: String,
        target_quantity: u32,
        target_price_stroops: i128,
        duration_seconds: u64,
    ) -> Result<u64, Error> {
        creator.require_auth();

        if target_quantity == 0 {
            return Err(Error::InvalidQuantity);
        }
        if target_price_stroops <= 0 {
            return Err(Error::InvalidPrice);
        }
        if !(60..=31_536_000).contains(&duration_seconds) {
            return Err(Error::InvalidDeadline);
        }
        if title.is_empty() || title.len() > 64 {
            return Err(Error::InvalidTitle);
        }
        if metadata_uri.is_empty() || metadata_uri.len() > 256 {
            return Err(Error::InvalidMetadataUri);
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CircleCounter)
            .unwrap_or(0);
        counter += 1;

        let now = env.ledger().timestamp();
        let deadline = now + duration_seconds;

        let circle = DemandCircle {
            id: counter,
            creator: creator.clone(),
            title,
            metadata_uri,
            target_quantity,
            target_price_stroops,
            deadline,
            created_at: now,
            status: CircleStatus::Open,
        };

        env.storage()
            .instance()
            .set(&DataKey::Circle(counter), &circle);
        env.storage()
            .instance()
            .set(&DataKey::CircleCounter, &counter);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);

        // Emit structured event for indexers
        env.events().publish(
            (symbol_short!("circle"), symbol_short!("created")),
            (counter, creator, target_quantity, target_price_stroops),
        );

        Ok(counter)
    }

    /// Reads an individual demand circle record by its ID.
    pub fn get_circle(env: Env, circle_id: u64) -> Result<DemandCircle, Error> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)
    }

    /// Returns the total count of registered demand circles.
    pub fn get_circle_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .instance()
            .get(&DataKey::CircleCounter)
            .unwrap_or(0)
    }

    /// Allows the creator to explicitly close an open demand circle.
    pub fn close_circle(env: Env, caller: Address, circle_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.creator != caller {
            return Err(Error::Unauthorized);
        }

        if circle.status != CircleStatus::Open {
            return Err(Error::InvalidState);
        }

        circle.status = CircleStatus::Closed;
        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);

        env.events().publish(
            (symbol_short!("circle"), symbol_short!("closed")),
            (circle_id, caller),
        );

        Ok(())
    }

    /// Transitions an open demand circle to Expired status if its deadline has elapsed.
    pub fn expire_circle(env: Env, circle_id: u64) -> Result<(), Error> {
        let mut circle: DemandCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)?;

        if circle.status != CircleStatus::Open {
            return Err(Error::InvalidState);
        }

        if env.ledger().timestamp() <= circle.deadline {
            return Err(Error::DeadlineNotPassed);
        }

        circle.status = CircleStatus::Expired;
        env.storage()
            .instance()
            .set(&DataKey::Circle(circle_id), &circle);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);

        env.events().publish(
            (symbol_short!("circle"), symbol_short!("expired")),
            circle_id,
        );

        Ok(())
    }

    /// Returns the registry administrator address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }
}

#[cfg(test)]
mod test;
