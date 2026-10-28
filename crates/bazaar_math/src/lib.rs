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
}
