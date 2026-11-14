import {
  FiatConversionQuote,
  ReflectorPriceQuote,
  RfqPriceOracleConfig,
  Sep0038FirmQuote,
  Sep0038FirmQuoteRequest,
  Sep0038InfoResponse,
  Sep0038PriceQuote,
  Sep0038PricesResponse,
  Sep0038QuoteContext,
} from './types';
import {
  applySlippageGuard,
  fiatToTokenStroops,
  fromStroops,
  parseScaledDecimal,
  toStroops,
} from './converters';
import { NetworkError, ValidationError } from './errors';
import { STROOPS_PER_XLM } from './constants';

const DEFAULT_MAX_SLIPPAGE_BPS = 100; // 1.00%
const DEFAULT_MAX_QUOTE_AGE_SECONDS = 300; // 5 minutes
const DEFAULT_MIN_REMAINING_TTL_SECONDS = 15; // 15 seconds safety buffer before quote expiry

/**
 * SEP-0038 Anchor RFQ & Reflector Oracle price integration for converting fiat-pegged
 * target prices (e.g., USD, EUR, NGN) into 7-decimal Soroban token stroops (USDC/XLM)
 * with slippage protection and quote expiration guards.
 */
export class RfqPriceOracle {
  private readonly anchorBaseUrl: string;
  private readonly jwtToken?: string;
  private readonly maxSlippageBps: number;
  private readonly maxQuoteAgeSeconds: number;
  private readonly minRemainingTtlSeconds: number;
  private readonly fetchFn: typeof fetch;

  constructor(config: RfqPriceOracleConfig) {
    if (!config.anchorBaseUrl || !config.anchorBaseUrl.trim()) {
      throw new ValidationError('anchorBaseUrl is required for RfqPriceOracle', 'anchorBaseUrl');
    }

    this.anchorBaseUrl = config.anchorBaseUrl.trim().replace(/\/+$/, '');
    this.jwtToken = config.jwtToken;
    this.maxSlippageBps = config.maxSlippageBps ?? DEFAULT_MAX_SLIPPAGE_BPS;
    this.maxQuoteAgeSeconds = config.maxQuoteAgeSeconds ?? DEFAULT_MAX_QUOTE_AGE_SECONDS;
    this.minRemainingTtlSeconds =
      config.minRemainingTtlSeconds ?? DEFAULT_MIN_REMAINING_TTL_SECONDS;
    this.fetchFn = config.fetchFn ?? fetch;

    if (this.maxSlippageBps < 0 || this.maxSlippageBps > 10_000) {
      throw new ValidationError('maxSlippageBps must be between 0 and 10000', 'maxSlippageBps');
    }
  }

  /**
   * Queries `GET /info` on the SEP-0038 anchor to discover supported off-chain fiat
   * and on-chain Stellar assets along with delivery methods and country codes.
   */
  public async getInfo(): Promise<Sep0038InfoResponse> {
    return this.requestJson<Sep0038InfoResponse>('/info', { method: 'GET' });
  }

  /**
   * Queries `GET /prices` for indicative prices across available assets for a given
   * sell or buy asset.
   */
  public async getPrices(params: {
    sellAsset?: string;
    buyAsset?: string;
    sellAmount?: string;
    buyAmount?: string;
    sellDeliveryMethod?: string;
    buyDeliveryMethod?: string;
    countryCode?: string;
  }): Promise<Sep0038PricesResponse> {
    if (!params.sellAsset && !params.buyAsset) {
      throw new ValidationError('Either sellAsset or buyAsset must be provided', 'sellAsset');
    }
    if (!params.sellAmount && !params.buyAmount) {
      throw new ValidationError('Either sellAmount or buyAmount must be provided', 'sellAmount');
    }
    if (params.sellAmount && params.buyAmount) {
      throw new ValidationError(
        'sellAmount and buyAmount are mutually exclusive in SEP-0038 /prices',
        'sellAmount'
      );
    }

    const query = new URLSearchParams();
    if (params.sellAsset) query.set('sell_asset', params.sellAsset);
    if (params.buyAsset) query.set('buy_asset', params.buyAsset);
    if (params.sellAmount) query.set('sell_amount', params.sellAmount);
    if (params.buyAmount) query.set('buy_amount', params.buyAmount);
    if (params.sellDeliveryMethod) query.set('sell_delivery_method', params.sellDeliveryMethod);
    if (params.buyDeliveryMethod) query.set('buy_delivery_method', params.buyDeliveryMethod);
    if (params.countryCode) query.set('country_code', params.countryCode);

    return this.requestJson<Sep0038PricesResponse>(`/prices?${query.toString()}`, {
      method: 'GET',
    });
  }

  /**
   * Queries `GET /price` for an indicative SEP-0038 exchange quote between `sellAsset`
   * and `buyAsset`.
   */
  public async getIndicativePrice(params: {
    context: Sep0038QuoteContext;
    sellAsset: string;
    buyAsset: string;
    sellAmount?: string;
    buyAmount?: string;
    sellDeliveryMethod?: string;
    buyDeliveryMethod?: string;
    countryCode?: string;
  }): Promise<Sep0038PriceQuote> {
    if (!params.sellAsset || !params.buyAsset) {
      throw new ValidationError('Both sellAsset and buyAsset are required', 'sellAsset');
    }
    if ((!params.sellAmount && !params.buyAmount) || (params.sellAmount && params.buyAmount)) {
      throw new ValidationError(
        'Exactly one of sellAmount or buyAmount must be provided',
        'sellAmount'
      );
    }

    const query = new URLSearchParams({
      context: params.context,
      sell_asset: params.sellAsset,
      buy_asset: params.buyAsset,
    });
    if (params.sellAmount) query.set('sell_amount', params.sellAmount);
    if (params.buyAmount) query.set('buy_amount', params.buyAmount);
    if (params.sellDeliveryMethod) query.set('sell_delivery_method', params.sellDeliveryMethod);
    if (params.buyDeliveryMethod) query.set('buy_delivery_method', params.buyDeliveryMethod);
    if (params.countryCode) query.set('country_code', params.countryCode);

    return this.requestJson<Sep0038PriceQuote>(`/price?${query.toString()}`, {
      method: 'GET',
    });
  }

  /**
   * Requests a firm, time-bound exchange quote via `POST /quote` on a SEP-0038 anchor.
   * Validates that the returned quote has not already expired or fallen within the
   * minimum remaining TTL safety buffer.
   */
  public async requestFirmQuote(
    request: Sep0038FirmQuoteRequest,
    nowMs: number = Date.now()
  ): Promise<Sep0038FirmQuote> {
    if (!request.sell_asset || !request.buy_asset) {
      throw new ValidationError('sell_asset and buy_asset are required', 'sell_asset');
    }
    if ((!request.sell_amount && !request.buy_amount) || (request.sell_amount && request.buy_amount)) {
      throw new ValidationError(
        'Exactly one of sell_amount or buy_amount must be specified',
        'sell_amount'
      );
    }

    const quote = await this.requestJson<Sep0038FirmQuote>('/quote', {
      method: 'POST',
      body: JSON.stringify(request),
    });

    this.validateQuoteFreshness({ expiresAt: quote.expires_at }, nowMs);
    return quote;
  }

  /**
   * Fetches a previously created SEP-0038 firm quote by ID via `GET /quote/:id`
   * and verifies that it has not expired.
   */
  public async getFirmQuote(quoteId: string, nowMs: number = Date.now()): Promise<Sep0038FirmQuote> {
    if (!quoteId || !quoteId.trim()) {
      throw new ValidationError('quoteId is required', 'quoteId');
    }

    const quote = await this.requestJson<Sep0038FirmQuote>(
      `/quote/${encodeURIComponent(quoteId.trim())}`,
      { method: 'GET' }
    );
    this.validateQuoteFreshness({ expiresAt: quote.expires_at }, nowMs);
    return quote;
  }

  /**
   * Validates that an RFQ firm quote or Reflector oracle quote is neither expired
   * nor older than `maxQuoteAgeSeconds`.
   */
  public validateQuoteFreshness(
    quote: { expiresAt?: string; timestamp?: number },
    nowMs: number = Date.now()
  ): void {
    if (quote.expiresAt !== undefined) {
      const expiresAtMs = Date.parse(quote.expiresAt);
      if (Number.isNaN(expiresAtMs)) {
        throw new ValidationError(`Invalid quote expires_at timestamp: "${quote.expiresAt}"`, 'expires_at');
      }

      const remainingSeconds = (expiresAtMs - nowMs) / 1000;
      if (remainingSeconds <= 0) {
        throw new ValidationError(
          `SEP-0038 quote has expired at ${quote.expiresAt}`,
          'expires_at'
        );
      }

      if (remainingSeconds < this.minRemainingTtlSeconds) {
        throw new ValidationError(
          `SEP-0038 quote remaining TTL (${remainingSeconds.toFixed(1)}s) is below minimum safety buffer (${this.minRemainingTtlSeconds}s)`,
          'expires_at'
        );
      }
    }

    if (quote.timestamp !== undefined) {
      const quoteTimestampMs = quote.timestamp < 1e12 ? quote.timestamp * 1000 : quote.timestamp;
      const ageSeconds = (nowMs - quoteTimestampMs) / 1000;
      if (ageSeconds > this.maxQuoteAgeSeconds) {
        throw new ValidationError(
          `Oracle price quote is stale (age ${ageSeconds.toFixed(1)}s exceeds max ${this.maxQuoteAgeSeconds}s)`,
          'timestamp'
        );
      }
    }
  }

  /**
   * Verifies that a quoted price or token amount stays within `maxSlippageBps` of the
   * caller's expected baseline. Throws `ValidationError` if slippage exceeds tolerance.
   */
  public validateQuoteSlippage(
    expectedPriceOrAmount: string | number | bigint,
    actualPriceOrAmount: string | number | bigint,
    maxSlippageBps: number = this.maxSlippageBps,
    direction: 'max_spend' | 'min_receive' = 'max_spend'
  ): number {
    const expectedScaled =
      typeof expectedPriceOrAmount === 'bigint'
        ? expectedPriceOrAmount
        : parseScaledDecimal(expectedPriceOrAmount, 18);
    const actualScaled =
      typeof actualPriceOrAmount === 'bigint'
        ? actualPriceOrAmount
        : parseScaledDecimal(actualPriceOrAmount, 18);

    const guard = applySlippageGuard(expectedScaled, actualScaled, maxSlippageBps, direction);
    if (!guard.withinSlippage) {
      throw new ValidationError(
        `Quote slippage of ${guard.slippageBps} bps exceeds maximum allowed ${maxSlippageBps} bps`,
        'slippageBps'
      );
    }

    return guard.slippageBps;
  }

  /**
   * Converts a fiat target amount (e.g., `"250.00"` in `"iso4217:USD"`) into Soroban
   * 7-decimal token stroops (e.g., USDC or XLM) using either an indicative or firm
   * SEP-0038 quote, applying staleness and slippage guards before returning.
   */
  public async convertFiatTargetToTokenStroops(
    fiatAmount: string | number,
    fiatAsset: string,
    tokenAsset: string,
    options: {
      context?: Sep0038QuoteContext;
      expectedTokenAmount?: string | number;
      maxSlippageBps?: number;
      firm?: boolean;
      expireAfter?: string;
      nowMs?: number;
    } = {}
  ): Promise<FiatConversionQuote> {
    const formattedFiatAmount = String(fiatAmount).trim();
    parseScaledDecimal(formattedFiatAmount, 18);

    const context: Sep0038QuoteContext = options.context ?? 'sep6';
    const maxSlippageBps = options.maxSlippageBps ?? this.maxSlippageBps;
    const nowMs = options.nowMs ?? Date.now();

    let quote: Sep0038PriceQuote | Sep0038FirmQuote;
    let quoteId: string | undefined;
    let expiresAt: string | undefined;

    if (options.firm) {
      const firmQuote = await this.requestFirmQuote(
        {
          context,
          sell_asset: fiatAsset,
          buy_asset: tokenAsset,
          sell_amount: formattedFiatAmount,
          expire_after: options.expireAfter,
        },
        nowMs
      );
      quote = firmQuote;
      quoteId = firmQuote.id;
      expiresAt = firmQuote.expires_at;
    } else {
      quote = await this.getIndicativePrice({
        context,
        sellAsset: fiatAsset,
        buyAsset: tokenAsset,
        sellAmount: formattedFiatAmount,
      });
    }

    const tokenStroops =
      quote.buy_amount && Number(quote.buy_amount) > 0
        ? toStroops(quote.buy_amount)
        : fiatToTokenStroops(formattedFiatAmount, quote.total_price || quote.price);

    if (tokenStroops <= 0n) {
      throw new ValidationError('Converted token stroops must be greater than zero', 'buy_amount');
    }

    let slippageBps = 0;
    if (options.expectedTokenAmount !== undefined) {
      const expectedStroops = toStroops(options.expectedTokenAmount);
      slippageBps = this.validateQuoteSlippage(
        expectedStroops,
        tokenStroops,
        maxSlippageBps,
        'max_spend'
      );
    }

    return {
      fiatAmount: formattedFiatAmount,
      fiatAsset,
      tokenAsset,
      unitPrice: quote.price,
      totalPrice: quote.total_price,
      tokenStroops,
      tokenAmountDecimal: fromStroops(tokenStroops),
      slippageBps,
      quoteId,
      expiresAt,
    };
  }

  /**
   * Normalizes an on-chain Reflector Oracle price feed entry into a 7-decimal stroop rate
   * and decimal string after verifying quote freshness.
   */
  public normalizeReflectorQuote(
    reflectorQuote: ReflectorPriceQuote,
    nowMs: number = Date.now()
  ): { priceDecimal: string; stroopsPerUnit: bigint } {
    this.validateQuoteFreshness({ timestamp: reflectorQuote.timestamp }, nowMs);

    const rawPrice =
      typeof reflectorQuote.price === 'bigint'
        ? reflectorQuote.price
        : BigInt(reflectorQuote.price);
    if (rawPrice <= 0n) {
      throw new ValidationError('Reflector oracle price must be positive', 'price');
    }
    if (reflectorQuote.decimals < 0 || reflectorQuote.decimals > 36) {
      throw new ValidationError('Reflector oracle decimals must be between 0 and 36', 'decimals');
    }

    const sourceScale = 10n ** BigInt(reflectorQuote.decimals);
    const stroopsPerUnit = (rawPrice * STROOPS_PER_XLM + sourceScale / 2n) / sourceScale;

    return {
      priceDecimal: fromStroops(stroopsPerUnit),
      stroopsPerUnit,
    };
  }

  private async requestJson<T>(path: string, init: RequestInit): Promise<T> {
    const headers: Record<string, string> = {
      Accept: 'application/json',
      ...(init.body ? { 'Content-Type': 'application/json' } : {}),
      ...(this.jwtToken ? { Authorization: `Bearer ${this.jwtToken}` } : {}),
    };

    let response: Response;
    try {
      response = await this.fetchFn(`${this.anchorBaseUrl}${path}`, {
        ...init,
        headers,
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new NetworkError(`SEP-0038 RFQ network request failed for ${path}: ${message}`);
    }

    if (!response.ok) {
      let errorDetail = response.statusText;
      try {
        const body = (await response.json()) as { error?: string };
        if (body && typeof body.error === 'string') {
          errorDetail = body.error;
        }
      } catch {
        // Fallback to HTTP status text if error body is not JSON
      }
      throw new NetworkError(
        `SEP-0038 endpoint ${path} returned HTTP ${response.status}: ${errorDetail}`,
        response.status
      );
    }

    return (await response.json()) as T;
  }
}
