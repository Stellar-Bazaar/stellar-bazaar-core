import { describe, it, expect } from 'vitest';
import {
  StellarBazaarError,
  NetworkError,
  ValidationError,
  SorobanContractError,
  resolveContractError,
  CONTRACT_ERROR_MAP,
} from '../src/errors';

describe('SDK Error Hierarchy', () => {
  it('instantiates base StellarBazaarError', () => {
    const err = new StellarBazaarError('Root failure');
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(StellarBazaarError);
    expect(err.name).toBe('StellarBazaarError');
    expect(err.message).toBe('Root failure');
  });

  it('instantiates NetworkError with status', () => {
    const netErr = new NetworkError('Gateway timeout', 504);
    expect(netErr).toBeInstanceOf(StellarBazaarError);
    expect(netErr.name).toBe('NetworkError');
    expect(netErr.status).toBe(504);
  });

  it('instantiates ValidationError with field', () => {
    const valErr = new ValidationError('Invalid amount', 'commitStroops');
    expect(valErr).toBeInstanceOf(StellarBazaarError);
    expect(valErr.name).toBe('ValidationError');
    expect(valErr.field).toBe('commitStroops');
  });

  it('resolves mapped SorobanContractError codes 1 through 27', () => {
    expect(Object.keys(CONTRACT_ERROR_MAP).length).toBe(27);

    const errCeiling = resolveContractError(8);
    expect(errCeiling.code).toBe(8);
    expect(errCeiling.errorName).toBe('CeilingPriceExceeded');
    expect(errCeiling.remediation).toContain('Submit an offer at or below the maximum price');

    const errFee = resolveContractError(23);
    expect(errFee.code).toBe(23);
    expect(errFee.errorName).toBe('InvalidFeeBps');
    expect(errFee.message).toContain('exceeds the maximum allowable threshold');

    const errRebate = resolveContractError(18);
    expect(errRebate.code).toBe(18);
    expect(errRebate.errorName).toBe('RebateAlreadyClaimed');
  });

  it('falls back cleanly for unrecognized error codes', () => {
    const unknownErr = resolveContractError(999);
    expect(unknownErr.code).toBe(999);
    expect(unknownErr.errorName).toBe('UnknownError999');
    expect(unknownErr.remediation).toContain('Check contract ledger logs');
  });
});
