import { STROOPS_PER_XLM, STELLAR_DECIMALS } from './constants';
import { ValidationError } from './errors';

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
export function fromStroops(stroops: bigint, options: { maxDecimals?: number } = {}): string {
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

/**
 * High-precision scale factor (10^18) used for intermediate fixed-point division
 * to prevent floating-point truncation when converting between fiat and 7-decimal stroops.
 */
const FIXED_POINT_SCALE = 1_000_000_000_000_000_000n;

/**
 * Parses an arbitrary decimal string or number into a scaled BigInt with `targetDecimals` precision.
 */
export function parseScaledDecimal(amount: string | number, targetDecimals: number): bigint {
  const str = String(amount).trim();
  if (!/^\d+(\.\d+)?$/.test(str)) {
    throw new ValidationError(`Invalid positive decimal format: "${amount}"`, 'amount');
  }

  const [whole = '0', fraction = ''] = str.split('.');
  const scaleFactor = 10n ** BigInt(targetDecimals);

  if (fraction.length <= targetDecimals) {
    const paddedFraction = fraction.padEnd(targetDecimals, '0');
    return BigInt(whole) * scaleFactor + BigInt(paddedFraction || '0');
  }

  const truncatedFraction = fraction.slice(0, targetDecimals);
  return BigInt(whole) * scaleFactor + BigInt(truncatedFraction || '0');
}

/**
 * Converts a fiat-denominated price or target amount into 7-decimal Soroban token stroops
 * using lossless BigInt fixed-point arithmetic (`fiatAmount / priceFiatPerToken`).
 *
 * @param fiatAmount - Target fiat amount (e.g., "100.00" USD or "150000" NGN)
 * @param priceFiatPerToken - Fiat price per 1 whole token (e.g., "1.00" USD/USDC or "0.125" USD/XLM)
 * @param rounding - Rounding mode for fractional stroops ('ceil' | 'floor' | 'half_up', default 'half_up')
 */
export function fiatToTokenStroops(
  fiatAmount: string | number,
  priceFiatPerToken: string | number,
  rounding: 'ceil' | 'floor' | 'half_up' = 'half_up'
): bigint {
  const scaledFiat = parseScaledDecimal(fiatAmount, 18);
  const scaledPrice = parseScaledDecimal(priceFiatPerToken, 18);

  if (scaledPrice <= 0n) {
    throw new ValidationError(
      'Token fiat price must be strictly greater than zero',
      'priceFiatPerToken'
    );
  }

  const numerator = scaledFiat * STROOPS_PER_XLM;
  const quotient = numerator / scaledPrice;
  const remainder = numerator % scaledPrice;

  if (remainder === 0n || rounding === 'floor') {
    return quotient;
  }

  if (rounding === 'ceil') {
    return quotient + 1n;
  }

  return remainder * 2n >= scaledPrice ? quotient + 1n : quotient;
}

/**
 * Converts a 7-decimal Soroban token stroop balance into a formatted fiat string
 * using lossless BigInt arithmetic (`(stroops * priceFiatPerToken) / 10^7`).
 */
export function tokenStroopsToFiat(
  stroops: bigint,
  priceFiatPerToken: string | number,
  maxDecimals: number = 2
): string {
  if (stroops < 0n) {
    throw new ValidationError('Stroop amount cannot be negative', 'stroops');
  }
  const scaledPrice = parseScaledDecimal(priceFiatPerToken, 18);
  if (scaledPrice <= 0n) {
    throw new ValidationError(
      'Token fiat price must be strictly greater than zero',
      'priceFiatPerToken'
    );
  }

  const numerator = stroops * scaledPrice;
  const divisor = FIXED_POINT_SCALE;
  const fiatStroops = (numerator + divisor / 2n) / divisor;
  return fromStroops(fiatStroops, { maxDecimals });
}

/**
 * Evaluates whether an actual quoted price or stroop amount stays within the configured
 * basis-point slippage tolerance (`maxSlippageBps`, where 100 bps = 1%).
 */
export function applySlippageGuard(
  expectedStroops: bigint,
  actualStroops: bigint,
  maxSlippageBps: number,
  direction: 'max_spend' | 'min_receive' = 'max_spend'
): { withinSlippage: boolean; slippageBps: number; limitStroops: bigint } {
  if (expectedStroops <= 0n) {
    throw new ValidationError('Expected stroops must be strictly positive', 'expectedStroops');
  }
  if (actualStroops <= 0n) {
    throw new ValidationError('Actual stroops must be strictly positive', 'actualStroops');
  }
  if (maxSlippageBps < 0 || maxSlippageBps > 10_000) {
    throw new ValidationError('maxSlippageBps must be between 0 and 10000', 'maxSlippageBps');
  }

  const bps = BigInt(maxSlippageBps);
  if (direction === 'max_spend') {
    const limitStroops = (expectedStroops * (10_000n + bps)) / 10_000n;
    const diff = actualStroops > expectedStroops ? actualStroops - expectedStroops : 0n;
    const slippageBps = Number((diff * 10_000n) / expectedStroops);
    return {
      withinSlippage: actualStroops <= limitStroops,
      slippageBps,
      limitStroops,
    };
  }

  const limitStroops = (expectedStroops * (10_000n - bps)) / 10_000n;
  const diff = expectedStroops > actualStroops ? expectedStroops - actualStroops : 0n;
  const slippageBps = Number((diff * 10_000n) / expectedStroops);
  return {
    withinSlippage: actualStroops >= limitStroops,
    slippageBps,
    limitStroops,
  };
}
