import { NetworkProfile } from './types';
import { NETWORKS, CONTRACT_ADDRESSES } from './constants';

export interface ClientConfig {
  network?: 'testnet' | 'futurenet' | 'local';
  dealEngineAddress?: string;
  circleRegistryAddress?: string;
  customRpcUrl?: string;
}

export interface ContractInvocation {
  contractAddress: string;
  method: string;
  args: any[];
}

export class StellarBazaarClient {
  private network: NetworkProfile;
  private dealEngineAddress: string;
  private circleRegistryAddress: string;

  constructor(config: ClientConfig = {}) {
    const netKey = config.network || 'testnet';
    const baseNet = NETWORKS[netKey] || NETWORKS.testnet;

    this.network = {
      ...baseNet,
      rpcUrl: config.customRpcUrl || baseNet.rpcUrl,
    };

    const presetAddresses = CONTRACT_ADDRESSES[netKey as keyof typeof CONTRACT_ADDRESSES] || CONTRACT_ADDRESSES.testnet;
    this.dealEngineAddress = config.dealEngineAddress || presetAddresses.dealEngine;
    this.circleRegistryAddress = config.circleRegistryAddress || presetAddresses.circleRegistry;
  }

  public getNetwork(): NetworkProfile {
    return { ...this.network };
  }

  public getDealEngineAddress(): string {
    return this.dealEngineAddress;
  }

  public getCircleRegistryAddress(): string {
    return this.circleRegistryAddress;
  }

  public buildCreateCircleInvocation(
    creator: string,
    tokenAddress: string,
    targetUnitPriceStroops: bigint,
    minVolume: number,
    maxVolume: number,
    durationSeconds: number
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'create_circle',
      args: [
        creator,
        tokenAddress,
        targetUnitPriceStroops,
        minVolume,
        maxVolume,
        durationSeconds,
      ],
    };
  }

  public buildCommitDemandInvocation(
    buyer: string,
    circleId: string | number,
    volume: number
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'commit_demand',
      args: [buyer, BigInt(circleId), volume],
    };
  }

  public buildSubmitOfferInvocation(
    seller: string,
    circleId: string | number,
    unitPriceStroops: bigint,
    volume: number,
    leadTimeDays: number
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'submit_seller_offer',
      args: [seller, BigInt(circleId), unitPriceStroops, volume, leadTimeDays],
    };
  }

  public buildAcceptOfferInvocation(
    creator: string,
    circleId: string | number,
    offerId: string | number
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'accept_seller_offer',
      args: [creator, BigInt(circleId), BigInt(offerId)],
    };
  }

  public buildInitiateInspectionInvocation(
    creator: string,
    circleId: string | number,
    durationSeconds: number
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'initiate_inspection',
      args: [creator, BigInt(circleId), durationSeconds],
    };
  }

  public buildConfirmInspectionInvocation(
    creator: string,
    circleId: string | number
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'confirm_inspection_and_settle',
      args: [creator, BigInt(circleId)],
    };
  }

  public buildClaimSurplusRebateInvocation(
    buyer: string,
    circleId: string | number
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'claim_surplus_rebate',
      args: [buyer, BigInt(circleId)],
    };
  }

  public buildDisputeDealInvocation(
    buyer: string,
    circleId: string | number,
    reason: string
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'dispute_deal',
      args: [buyer, BigInt(circleId), reason],
    };
  }

  public buildResolveDisputeInvocation(
    admin: string,
    circleId: string | number,
    refundToBuyers: boolean
  ): ContractInvocation {
    return {
      contractAddress: this.dealEngineAddress,
      method: 'resolve_dispute',
      args: [admin, BigInt(circleId), refundToBuyers],
    };
  }
}
