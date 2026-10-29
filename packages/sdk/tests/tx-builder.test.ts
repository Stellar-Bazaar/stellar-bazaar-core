import { describe, it, expect } from 'vitest';
import { TransactionBuilder } from '../src/tx-builder';
import { ContractInvocation } from '../src/client';

describe('TransactionBuilder', () => {
  const dummyInvocation: ContractInvocation = {
    contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
    method: 'commit_to_circle',
    args: ['1', 'GBUYER123', '50000000', '10000000'],
  };

  it('builds prepared transaction with defaults', () => {
    const builder = new TransactionBuilder('GBUYER123', dummyInvocation);
    const tx = builder.build();

    expect(tx.sourceAccount).toBe('GBUYER123');
    expect(tx.contractInvocation.method).toBe('commit_to_circle');
    expect(tx.baseFeeStroops).toBe(100n);
    expect(tx.resourceFeeStroops).toBe(20_000n);
    expect(tx.totalFeeStroops).toBe(20_100n);
    expect(tx.timeoutSeconds).toBe(300);
    expect(tx.estimatedCpuInstructions).toBe(1_500_000);
  });

  it('customizes base fee and timeout', () => {
    const builder = new TransactionBuilder('GBUYER123', dummyInvocation)
      .setBaseFee(500n)
      .setTimeout(600);

    const tx = builder.build();
    expect(tx.baseFeeStroops).toBe(500n);
    expect(tx.timeoutSeconds).toBe(600);
    expect(tx.totalFeeStroops).toBe(20_500n);
  });

  it('customizes resource budget metrics', () => {
    const builder = new TransactionBuilder('GBUYER123', dummyInvocation)
      .setResourceBudget({
        cpuInstructions: 3_000_000,
        memoryBytes: 1_200_000,
        minResourceFeeStroops: 45_000n,
      });

    const tx = builder.build();
    expect(tx.estimatedCpuInstructions).toBe(3_000_000);
    expect(tx.resourceFeeStroops).toBe(45_000n);
    expect(tx.totalFeeStroops).toBe(45_100n);
  });

  it('ignores non-positive base fee and timeout modifications', () => {
    const builder = new TransactionBuilder('GBUYER123', dummyInvocation)
      .setBaseFee(0n)
      .setTimeout(0);

    const tx = builder.build();
    expect(tx.baseFeeStroops).toBe(100n);
    expect(tx.timeoutSeconds).toBe(300);
  });
});
