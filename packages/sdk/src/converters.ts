import { STROOPS_PER_XLM, STELLAR_DECIMALS } from './constants';

/**
 * Converts a human-readable decimal amount string or number to stroops (BigInt).
 * Lossless with exact 7-decimal place scaling.
 */
export function toStroops(amount: string | number): bigint {
  const str = typeof amount === 'number' ? amount.toFixed(STELLAR_DECIMALS) : amount.trim();
  if (!str || str === '0') return 0n;

  const parts = str.split('.');
  const wholePart = parts[0] || '0';
  let fractionPart = parts[1] || '';

  // Pad or truncate fraction to 7 decimals
  if (fractionPart.length > STELLAR_DECIMALS) {
    fractionPart = fractionPart.slice(0, STELLAR_DECIMALS);
  } else {
    fractionPart = fractionPart.padEnd(STELLAR_DECIMALS, '0');
  }

  const whole = BigInt(wholePart) * STROOPS_PER_XLM;
  const fraction = BigInt(fractionPart);
  return whole + fraction;
}

/**
 * Converts integer stroops to human-readable decimal string.
 */
export function fromStroops(
  stroops: bigint,
  options: { maxDecimals?: number } = {}
): string {
  const isNegative = stroops < 0n;
  const abs = isNegative ? -stroops : stroops;

  const whole = abs / STROOPS_PER_XLM;
  const fraction = abs % STROOPS_PER_XLM;

  const fractionStr = fraction.toString().padStart(STELLAR_DECIMALS, '0');
  let trimmedFraction = fractionStr.replace(/0+$/, '');

  const maxDecimals = options.maxDecimals ?? STELLAR_DECIMALS;
  if (trimmedFraction.length > maxDecimals) {
    trimmedFraction = trimmedFraction.slice(0, maxDecimals);
  }

  const sign = isNegative ? '-' : '';
  if (trimmedFraction.length === 0) {
    return `${sign}${whole.toString()}`;
  }

  return `${sign}${whole.toString()}.${trimmedFraction}`;
}

/**
 * Calculates discount in basis points (1 bps = 0.01%).
 */
export function calculateDiscountBps(target: bigint, offer: bigint): number {
  if (target <= 0n || offer >= target) return 0;
  const delta = target - offer;
  return Number((delta * 10_000n) / target);
}

/**
 * Calculates buyer surplus rebate from target and accepted wholesale unit prices.
 */
export function calculateRebate(
  targetPriceStroops: bigint,
  acceptedPriceStroops: bigint,
  units: number
): bigint {
  if (units <= 0 || acceptedPriceStroops >= targetPriceStroops) return 0n;
  const delta = targetPriceStroops - acceptedPriceStroops;
  return delta * BigInt(units);
}
