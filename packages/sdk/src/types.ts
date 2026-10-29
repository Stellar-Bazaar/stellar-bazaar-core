/**
 * Core domain types and interfaces for the Stellar Bazaar SDK.
 */

export type CircleStatus =
  | 'OPEN'
  | 'QUORUM_REACHED'
  | 'INSPECTION'
  | 'SETTLED'
  | 'CANCELLED'
  | 'DISPUTED';

export type OfferStatus =
  | 'SUBMITTED'
  | 'ACCEPTED'
  | 'REJECTED'
  | 'EXPIRED';

export type NetworkId = 'testnet' | 'futurenet' | 'local';

export interface NetworkProfile {
  id: NetworkId;
  name: string;
  rpcUrl: string;
  horizonUrl: string;
  networkPassphrase: string;
}

export interface DemandCircle {
  id: string;
  creator: string;
  tokenAddress: string;
  targetUnitPriceStroops: bigint;
  minVolume: number;
  maxVolume: number;
  currentVolume: number;
  deadline: number;
  status: CircleStatus;
  acceptedOfferId: string | null;
  createdAt: number;
}

export interface Commitment {
  circleId: string;
  buyerAddress: string;
  volume: number;
  escrowAmountStroops: bigint;
  committedAt: number;
  refunded: boolean;
  claimedRebate: boolean;
}

export interface SellerOffer {
  id: string;
  circleId: string;
  sellerAddress: string;
  pricePerUnitStroops: bigint;
  volume: number;
  leadTimeDays: number;
  status: OfferStatus;
  createdAt: number;
}

export interface SellerReputation {
  sellerAddress: string;
  successfulDeals: number;
  totalVolumeSettled: number;
  totalAmountSettledStroops: bigint;
  disputedOrRefundedDeals: number;
}
