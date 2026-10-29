export class StellarBazaarError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'StellarBazaarError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export class NetworkError extends StellarBazaarError {
  public status?: number;
  constructor(message: string, status?: number) {
    super(message);
    this.name = 'NetworkError';
    this.status = status;
  }
}

export class ValidationError extends StellarBazaarError {
  public field?: string;
  constructor(message: string, field?: string) {
    super(message);
    this.name = 'ValidationError';
    this.field = field;
  }
}

export interface ContractErrorInfo {
  name: string;
  description: string;
  remediation: string;
}

export const CONTRACT_ERROR_MAP: Record<number, ContractErrorInfo> = {
  1: {
    name: 'AlreadyInitialized',
    description: 'Contract instance has already been initialized.',
    remediation: 'Do not invoke initialize more than once.',
  },
  2: {
    name: 'NotInitialized',
    description: 'Contract instance has not been initialized with admin & treasury.',
    remediation: 'Deploy and initialize the contract prior to trading.',
  },
  3: {
    name: 'Unauthorized',
    description: 'The caller lacks required authorization for this operation.',
    remediation: 'Ensure the calling account signs with valid keypair.',
  },
  4: {
    name: 'CircleNotFound',
    description: 'The requested buying circle ID does not exist in ledger storage.',
    remediation: 'Verify circle ID before querying or committing.',
  },
  5: {
    name: 'CircleNotActive',
    description: 'Circle is not in Open status.',
    remediation: 'Cannot join circles that are expired, locked, or closed.',
  },
  6: {
    name: 'CircleExpired',
    description: 'Circle deadline timestamp has passed.',
    remediation: 'Claim refund or create a new buying circle.',
  },
  7: {
    name: 'QuorumNotMet',
    description: 'Buyer commitments have not reached the required minimum volume quorum.',
    remediation: 'Wait for additional buyers to join before settlement.',
  },
  8: {
    name: 'CeilingPriceExceeded',
    description: 'Seller offer unit price exceeds the maximum ceiling price specified by the circle.',
    remediation: 'Submit an offer at or below the maximum price.',
  },
  9: {
    name: 'InsufficientVolume',
    description: 'Seller offer volume does not cover minimum circle demand.',
    remediation: 'Increase batch capacity to meet minimum volume.',
  },
  10: {
    name: 'LeadTimeTooLong',
    description: 'Offered delivery lead time exceeds the maximum lead time allowed.',
    remediation: 'Reduce estimated delivery lead time.',
  },
  11: {
    name: 'OfferNotFound',
    description: 'The requested seller offer does not exist.',
    remediation: 'Check offer ID or submit a new seller offer.',
  },
  12: {
    name: 'OfferAlreadyAccepted',
    description: 'An offer has already been accepted for this circle.',
    remediation: 'Circles can only accept one winning wholesale deal.',
  },
  13: {
    name: 'InspectionPeriodActive',
    description: 'The deal is still within the buyer inspection window.',
    remediation: 'Wait for inspection period to elapse or resolve disputes.',
  },
  14: {
    name: 'InspectionPeriodExpired',
    description: 'The inspection window has passed; deal is already finalised.',
    remediation: 'Disputes must be raised during the active inspection window.',
  },
  15: {
    name: 'DisputeActive',
    description: 'Deal cannot settle because an unresolved dispute exists.',
    remediation: 'Arbitrator must resolve dispute before funds release.',
  },
  16: {
    name: 'DisputeNotActive',
    description: 'No dispute is currently active for this circle.',
    remediation: 'Cannot resolve or act on non-existent disputes.',
  },
  17: {
    name: 'UnauthorizedArbitrator',
    description: 'Only the designated arbitrator or admin can resolve disputes.',
    remediation: 'Execute resolution using arbitrator key.',
  },
  18: {
    name: 'RebateAlreadyClaimed',
    description: 'Buyer has already claimed their surplus discount rebate.',
    remediation: 'Rebates can only be withdrawn once per participant.',
  },
  19: {
    name: 'NoRebateAvailable',
    description: 'No surplus rebate is available for this account in this circle.',
    remediation: 'Check if winning offer price was lower than ceiling price.',
  },
  20: {
    name: 'RefundAlreadyClaimed',
    description: 'Buyer refund has already been processed for this circle.',
    remediation: 'Refunds cannot be claimed multiple times.',
  },
  21: {
    name: 'DuplicateBuyerCommitment',
    description: 'Buyer is already registered in this buying circle.',
    remediation: 'Increase existing commitment rather than creating duplicate.',
  },
  22: {
    name: 'ZeroAmount',
    description: 'Operation amount must be greater than zero.',
    remediation: 'Specify a non-zero token amount.',
  },
  23: {
    name: 'InvalidFeeBps',
    description: 'Protocol fee bps exceeds the maximum allowable threshold (1,000 bps / 10%).',
    remediation: 'Specify fee basis points between 0 and 1000.',
  },
  24: {
    name: 'StorageLimitExceeded',
    description: 'Maximum circle or offer storage limit reached for this ledger.',
    remediation: 'Archive or settle prior circles before creating new ones.',
  },
  25: {
    name: 'ArithmeticOverflow',
    description: 'Math calculation resulted in an overflow or divide-by-zero.',
    remediation: 'Verify unit and volume values do not overflow i128.',
  },
  26: {
    name: 'DealNotAccepted',
    description: 'Cannot initiate dispute or settlement before a deal offer is accepted.',
    remediation: 'Accept a seller offer before lifecycle settlement.',
  },
  27: {
    name: 'PrematureSettlement',
    description: 'Attempted to finalize deal before delivery deadline or inspection completion.',
    remediation: 'Allow inspection window to close before trigger.',
  },
};

export class SorobanContractError extends StellarBazaarError {
  public code: number;
  public errorName: string;
  public remediation: string;

  constructor(code: number) {
    const info = CONTRACT_ERROR_MAP[code];
    const name = info ? info.name : `UnknownError${code}`;
    const desc = info ? info.description : `Unrecognized contract error code ${code}`;
    const remediation = info ? info.remediation : 'Check contract ledger logs and transaction simulation.';
    
    super(`Soroban Contract Error #${code} [${name}]: ${desc}`);
    this.name = 'SorobanContractError';
    this.code = code;
    this.errorName = name;
    this.remediation = remediation;
  }
}

export function resolveContractError(code: number): SorobanContractError {
  return new SorobanContractError(code);
}

