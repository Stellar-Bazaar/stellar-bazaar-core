#![no_std]

/// Number of Stroops in one whole Stellar Lumen (XLM).
pub const STROOPS_PER_XLM: i128 = 10_000_000;

/// Maximum basis points (100.00% = 10,000 bps).
pub const BPS_DENOMINATOR: i128 = 10_000;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MathError {
    Overflow,
    DivisionByZero,
    InvalidBasisPoints,
}

/// Converts whole XLM tokens to Stroops (1 XLM = 10,000,000 Stroops).
#[inline]
pub fn xlm_to_stroops(xlm: i128) -> Result<i128, MathError> {
    xlm.checked_mul(STROOPS_PER_XLM).ok_or(MathError::Overflow)
}

/// Calculates the total escrow required for a given order quantity and unit price in Stroops.
#[inline]
pub fn calculate_escrow(quantity: u32, unit_price_stroops: i128) -> Result<i128, MathError> {
    if unit_price_stroops < 0 {
        return Err(MathError::Overflow);
    }
    let q = quantity as i128;
    q.checked_mul(unit_price_stroops).ok_or(MathError::Overflow)
}

/// Calculates a basis point fee on an amount (e.g. 250 bps = 2.50%).
#[inline]
pub fn calculate_fee_bps(amount: i128, bps: u32) -> Result<i128, MathError> {
    if bps > 10_000 {
        return Err(MathError::InvalidBasisPoints);
    }
    let numerator = amount.checked_mul(bps as i128).ok_or(MathError::Overflow)?;
    Ok(numerator / BPS_DENOMINATOR)
}

/// Computes the integer percentage of quorum reached (0 to 100+).
#[inline]
pub fn calculate_quorum_pct(current_volume: u32, target_volume: u32) -> Result<u32, MathError> {
    if target_volume == 0 {
        return Err(MathError::DivisionByZero);
    }
    let current = current_volume as u64;
    let target = target_volume as u64;
    let pct = current
        .checked_mul(100)
        .ok_or(MathError::Overflow)?
        / target;
    Ok(pct as u32)
}

/// Calculates the protocol fee and net payout to seller after deducting basis points fee.
/// Returns (net_payout, fee_amount).
#[inline]
pub fn calculate_net_payout(gross_amount: i128, fee_bps: u32) -> Result<(i128, i128), MathError> {
    if gross_amount < 0 {
        return Err(MathError::Overflow);
    }
    let fee = calculate_fee_bps(gross_amount, fee_bps)?;
    let net = gross_amount.checked_sub(fee).ok_or(MathError::Overflow)?;
    Ok((net, fee))
}

/// Calculates the surplus rebate amount per buyer when accepted offer price is lower than target price.
#[inline]
pub fn calculate_surplus_rebate(
    target_unit_price: i128,
    accepted_unit_price: i128,
    quantity: u32,
) -> Result<i128, MathError> {
    if accepted_unit_price > target_unit_price {
        return Err(MathError::Overflow);
    }
    let diff = target_unit_price
        .checked_sub(accepted_unit_price)
        .ok_or(MathError::Overflow)?;
    let q = quantity as i128;
    q.checked_mul(diff).ok_or(MathError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xlm_to_stroops_conversion() {
        assert_eq!(xlm_to_stroops(1).unwrap(), 10_000_000);
        assert_eq!(xlm_to_stroops(5).unwrap(), 50_000_000);
        assert_eq!(xlm_to_stroops(0).unwrap(), 0);
    }

    #[test]
    fn test_calculate_escrow() {
        assert_eq!(calculate_escrow(10, 5_000_000).unwrap(), 50_000_000);
        assert_eq!(calculate_escrow(0, 5_000_000).unwrap(), 0);
        assert!(calculate_escrow(u32::MAX, i128::MAX).is_err());
    }

    #[test]
    fn test_calculate_fee_bps() {
        // 250 bps = 2.5% on 100 XLM (1,000,000,000 stroops) = 25,000,000 stroops (2.5 XLM)
        let amount = 1_000_000_000;
        let fee = calculate_fee_bps(amount, 250).unwrap();
        assert_eq!(fee, 25_000_000);

        assert!(calculate_fee_bps(amount, 10_001).is_err());
    }

    #[test]
    fn test_calculate_quorum_pct() {
        assert_eq!(calculate_quorum_pct(50, 100).unwrap(), 50);
        assert_eq!(calculate_quorum_pct(100, 100).unwrap(), 100);
        assert_eq!(calculate_quorum_pct(150, 100).unwrap(), 150);
        assert_eq!(calculate_quorum_pct(0, 100).unwrap(), 0);
        assert!(calculate_quorum_pct(50, 0).is_err());
    }

    #[test]
    fn test_calculate_net_payout() {
        // 100 XLM gross (1_000_000_000 stroops), 100 bps (1.00%) fee
        let (net, fee) = calculate_net_payout(1_000_000_000, 100).unwrap();
        assert_eq!(fee, 10_000_000); // 1 XLM
        assert_eq!(net, 990_000_000); // 99 XLM

        // Zero fee
        let (net0, fee0) = calculate_net_payout(500_000_000, 0).unwrap();
        assert_eq!(fee0, 0);
        assert_eq!(net0, 500_000_000);

        // Maximum fee (10,000 bps = 100%)
        let (net_max, fee_max) = calculate_net_payout(100_000_000, 10_000).unwrap();
        assert_eq!(fee_max, 100_000_000);
        assert_eq!(net_max, 0);

        // Negative gross amount should fail
        assert!(calculate_net_payout(-100, 100).is_err());
        // Fee bps exceeding 10_000 should fail
        assert!(calculate_net_payout(100, 10_001).is_err());
    }

    #[test]
    fn test_calculate_surplus_rebate() {
        // Target: 10 XLM (100_000_000 stroops), Accepted: 9 XLM (90_000_000 stroops)
        // Quantity: 5 units. Rebate per unit: 10_000_000 stroops. Total: 50_000_000 stroops (5 XLM)
        let rebate = calculate_surplus_rebate(100_000_000, 90_000_000, 5).unwrap();
        assert_eq!(rebate, 50_000_000);

        // Equal prices: rebate is 0
        let zero_rebate = calculate_surplus_rebate(100_000_000, 100_000_000, 10).unwrap();
        assert_eq!(zero_rebate, 0);

        // Zero quantity: rebate is 0
        let zero_qty = calculate_surplus_rebate(100_000_000, 80_000_000, 0).unwrap();
        assert_eq!(zero_qty, 0);

        // Accepted price higher than target price should fail
        assert!(calculate_surplus_rebate(90_000_000, 100_000_000, 5).is_err());
    }
}
