import { ContractInvocation } from './client';

export interface ResourceBudget {
  cpuInstructions: number;
  memoryBytes: number;
  minResourceFeeStroops: bigint;
}

export interface PreparedTransaction {
  sourceAccount: string;
  contractInvocation: ContractInvocation;
  baseFeeStroops: bigint;
  resourceFeeStroops: bigint;
  totalFeeStroops: bigint;
  timeoutSeconds: number;
  estimatedCpuInstructions: number;
}

export class TransactionBuilder {
  private invocation: ContractInvocation;
  private sourceAccount: string;
  private baseFeeStroops: bigint = 100n; // Default minimum Stellar base fee
  private resourceBudget: ResourceBudget = {
    cpuInstructions: 1_500_000,
    memoryBytes: 500_000,
    minResourceFeeStroops: 20_000n,
  };
  private timeoutSeconds: number = 300;

  constructor(sourceAccount: string, invocation: ContractInvocation) {
    this.sourceAccount = sourceAccount;
    this.invocation = invocation;
  }

  public setBaseFee(feeStroops: bigint): this {
    if (feeStroops > 0n) {
      this.baseFeeStroops = feeStroops;
    }
    return this;
  }

  public setResourceBudget(budget: Partial<ResourceBudget>): this {
    this.resourceBudget = {
      ...this.resourceBudget,
      ...budget,
    };
    return this;
  }

  public setTimeout(seconds: number): this {
    if (seconds > 0) {
      this.timeoutSeconds = seconds;
    }
    return this;
  }

  public build(): PreparedTransaction {
    const resourceFee = this.resourceBudget.minResourceFeeStroops;
    const totalFee = this.baseFeeStroops + resourceFee;

    return {
      sourceAccount: this.sourceAccount,
      contractInvocation: this.invocation,
      baseFeeStroops: this.baseFeeStroops,
      resourceFeeStroops: resourceFee,
      totalFeeStroops: totalFee,
      timeoutSeconds: this.timeoutSeconds,
      estimatedCpuInstructions: this.resourceBudget.cpuInstructions,
    };
  }
}

