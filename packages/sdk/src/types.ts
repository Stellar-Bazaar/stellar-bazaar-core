/**
 * Core domain types and interfaces for the Stellar Bazaar SDK.
 */

export type CircleStatus =
  'OPEN' | 'QUORUM_REACHED' | 'INSPECTION' | 'SETTLED' | 'CANCELLED' | 'DISPUTED';

export type OfferStatus = 'SUBMITTED' | 'ACCEPTED' | 'REJECTED' | 'EXPIRED';

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

export type Sep0038QuoteContext = 'sep6' | 'sep24' | 'sep31';

export interface Sep0038DeliveryMethod {
  name: string;
  description: string;
}

export interface Sep0038AssetDescriptor {
  asset: string;
  sell_delivery_methods?: Sep0038DeliveryMethod[];
  buy_delivery_methods?: Sep0038DeliveryMethod[];
  country_codes?: string[];
}

export interface Sep0038InfoResponse {
  assets: Sep0038AssetDescriptor[];
}

export interface Sep0038FeeDetails {
  total: string;
  asset: string;
  details?: Array<{
    name: string;
    description?: string;
    amount: string;
  }>;
}

export interface Sep0038IndicativePrice {
  buy_asset?: string;
  sell_asset?: string;
  price: string;
  decimals: number;
}

export interface Sep0038PricesResponse {
  buy_assets?: Sep0038IndicativePrice[];
  sell_assets?: Sep0038IndicativePrice[];
}

export interface Sep0038PriceQuote {
  total_price: string;
  price: string;
  sell_amount: string;
  buy_amount: string;
  fee: Sep0038FeeDetails;
}

export interface Sep0038FirmQuoteRequest {
  context: Sep0038QuoteContext;
  sell_asset: string;
  buy_asset: string;
  sell_amount?: string;
  buy_amount?: string;
  expire_after?: string;
  sell_delivery_method?: string;
  buy_delivery_method?: string;
  country_code?: string;
}

export interface Sep0038FirmQuote extends Sep0038PriceQuote {
  id: string;
  expires_at: string;
  sell_asset: string;
  buy_asset: string;
  sell_delivery_method?: string;
  buy_delivery_method?: string;
}

export interface ReflectorPriceQuote {
  baseAsset: string;
  quoteAsset: string;
  price: bigint | string;
  decimals: number;
  timestamp: number;
}

export interface RfqPriceOracleConfig {
  anchorBaseUrl: string;
  jwtToken?: string;
  maxSlippageBps?: number;
  maxQuoteAgeSeconds?: number;
  minRemainingTtlSeconds?: number;
  fetchFn?: typeof fetch;
}

export interface FiatConversionQuote {
  fiatAmount: string;
  fiatAsset: string;
  tokenAsset: string;
  unitPrice: string;
  totalPrice: string;
  tokenStroops: bigint;
  tokenAmountDecimal: string;
  slippageBps: number;
  quoteId?: string;
  expiresAt?: string;
}
