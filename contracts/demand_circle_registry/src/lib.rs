#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    CircleNotFound = 4,
    InvalidQuantity = 5,
    InvalidPrice = 6,
    InvalidDeadline = 7,
    InvalidTitle = 8,
    InvalidState = 9,
    DeadlineNotPassed = 10,
    DeadlinePassed = 11,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CircleStatus {
    Open = 0,
    Closed = 1,
    Expired = 2,
    Cancelled = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DemandCircle {
    pub id: u64,
    pub creator: Address,
    pub title: String,
    pub metadata_uri: String,
    pub target_quantity: u32,
    pub target_price_stroops: i128,
    pub deadline: u64,
    pub created_at: u64,
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
        if duration_seconds < 60 || duration_seconds > 31_536_000 {
            return Err(Error::InvalidDeadline);
        }
        if title.len() == 0 || title.len() > 64 {
            return Err(Error::InvalidTitle);
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
            .get(&DataKey::Circle(circle_id))
            .ok_or(Error::CircleNotFound)
    }

    /// Returns the total count of registered demand circles.
    pub fn get_circle_count(env: Env) -> u64 {
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
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }
}

#[cfg(test)]
mod test;
