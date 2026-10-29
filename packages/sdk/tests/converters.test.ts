import { describe, it, expect } from 'vitest';
import {
  toStroops,
  fromStroops,
  calculateDiscountBps,
  calculateRebate,
} from '../src/converters';

describe('SDK Converters', () => {
  it('converts decimal amounts to stroops accurately', () => {
    expect(toStroops('1')).toBe(10_000_000n);
    expect(toStroops('0.5')).toBe(5_000_000n);
    expect(toStroops('0.0000001')).toBe(1n);
    expect(toStroops('12.3456789')).toBe(123_456_789n);
    expect(toStroops(10)).toBe(100_000_000n);
  });

  it('converts stroops to decimal strings without scientific notation', () => {
    expect(fromStroops(10_000_000n)).toBe('1');
    expect(fromStroops(5_000_000n)).toBe('0.5');
    expect(fromStroops(1n)).toBe('0.0000001');
    expect(fromStroops(123_456_789n)).toBe('12.3456789');
    expect(fromStroops(123_456_789n, { maxDecimals: 2 })).toBe('12.34');
  });

  it('calculates basis points discount', () => {
    expect(calculateDiscountBps(10_000_000n, 8_000_000n)).toBe(2000); // 20%
    expect(calculateDiscountBps(10_000_000n, 9_500_000n)).toBe(500); // 5%
    expect(calculateDiscountBps(10_000_000n, 10_000_000n)).toBe(0);
    expect(calculateDiscountBps(10_000_000n, 12_000_000n)).toBe(0);
  });

  it('calculates buyer surplus rebate accurately', () => {
    // 5 units with 2 XLM savings per unit = 10 XLM rebate = 100_000_000 stroops
    const rebate = calculateRebate(100_000_000n, 80_000_000n, 5);
    expect(rebate).toBe(100_000_000n);
  });
});
