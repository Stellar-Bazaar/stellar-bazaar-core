import { describe, it, expect, vi } from 'vitest';
import {
  RfqPriceOracle,
  fiatToTokenStroops,
  tokenStroopsToFiat,
  applySlippageGuard,
  ValidationError,
  NetworkError,
} from '../src';

describe('Lossless Fiat/Stroop Converters & Slippage Guards', () => {
  it('converts fiat amounts to 7-decimal token stroops losslessly', () => {
    // 100 USD at $1.00/USDC = 100 USDC = 1,000,000,000 stroops
    expect(fiatToTokenStroops('100.00', '1.00')).toBe(1_000_000_000n);

    // 25 USD at $0.125/XLM = 200 XLM = 2,000,000,000 stroops
    expect(fiatToTokenStroops('25.00', '0.125')).toBe(2_000_000_000n);

    // 10 USD at $3.00/token = 3.3333333 tokens = 33,333,333 stroops (floor/half_up) or 33,333,334 (ceil)
    expect(fiatToTokenStroops('10', '3', 'floor')).toBe(33_333_333n);
    expect(fiatToTokenStroops('10', '3', 'ceil')).toBe(33_333_334n);
  });

  it('converts token stroops back to fiat decimal strings without floating-point drift', () => {
    expect(tokenStroopsToFiat(1_000_000_000n, '1.00')).toBe('100');
    expect(tokenStroopsToFiat(2_000_000_000n, '0.125')).toBe('25');
    expect(tokenStroopsToFiat(123_456_789n, '1.00', 2)).toBe('12.34');
  });

  it('enforces basis-point slippage guards accurately', () => {
    // Expected 100 XLM (1_000_000_000n), max 100 bps (1%) -> limit 101 XLM (1_010_000_000n)
    const ok = applySlippageGuard(1_000_000_000n, 1_008_000_000n, 100, 'max_spend');
    expect(ok.withinSlippage).toBe(true);
    expect(ok.slippageBps).toBe(80);
    expect(ok.limitStroops).toBe(1_010_000_000n);

    const exceeded = applySlippageGuard(1_000_000_000n, 1_025_000_000n, 100, 'max_spend');
    expect(exceeded.withinSlippage).toBe(false);
    expect(exceeded.slippageBps).toBe(250);
  });
});

describe('RfqPriceOracle (SEP-0038 & Reflector Integration)', () => {
  const ANCHOR_URL = 'https://anchor.stellar-bazaar.example/sep38';
  const USDC_ASSET =
    'stellar:USDC:GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN';
  const USD_ASSET = 'iso4217:USD';

  it('discovers anchor assets via GET /info', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        assets: [
          {
            asset: USDC_ASSET,
          },
          {
            asset: USD_ASSET,
            sell_delivery_methods: [{ name: 'ACH', description: 'US ACH Bank Transfer' }],
            country_codes: ['US'],
          },
        ],
      }),
    });

    const oracle = new RfqPriceOracle({
      anchorBaseUrl: ANCHOR_URL,
      fetchFn: mockFetch as unknown as typeof fetch,
    });

    const info = await oracle.getInfo();
    expect(info.assets).toHaveLength(2);
    expect(info.assets[0]?.asset).toBe(USDC_ASSET);
    expect(mockFetch).toHaveBeenCalledWith(
      `${ANCHOR_URL}/info`,
      expect.objectContaining({ method: 'GET' })
    );
  });

  it('converts a fiat target into 7-decimal token stroops using an indicative SEP-0038 quote', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        total_price: '1.0050000',
        price: '1.0000000',
        sell_amount: '502.50',
        buy_amount: '500.0000000',
        fee: {
          total: '2.50',
          asset: USD_ASSET,
        },
      }),
    });

    const oracle = new RfqPriceOracle({
      anchorBaseUrl: ANCHOR_URL,
      maxSlippageBps: 100,
      fetchFn: mockFetch as unknown as typeof fetch,
    });

    const result = await oracle.convertFiatTargetToTokenStroops(
      '502.50',
      USD_ASSET,
      USDC_ASSET,
      {
        context: 'sep6',
        expectedTokenAmount: '500',
      }
    );

    expect(result.tokenStroops).toBe(5_000_000_000n);
    expect(result.tokenAmountDecimal).toBe('500');
    expect(result.slippageBps).toBe(0);
  });

  it('requests a firm quote via POST /quote and rejects expired or near-expiry quotes', async () => {
    const nowMs = Date.parse('2026-03-29T12:00:00Z');
    const validExpiry = '2026-03-29T12:05:00Z';
    const expiredExpiry = '2026-03-29T11:59:00Z';

    const mockFetch = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: async () => ({
          id: 'quote-firm-001',
          expires_at: validExpiry,
          total_price: '0.1250000',
          price: '0.1240000',
          sell_asset: USD_ASSET,
          sell_amount: '125.00',
          buy_asset: 'stellar:native',
          buy_amount: '1000.0000000',
          fee: { total: '1.00', asset: USD_ASSET },
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: async () => ({
          id: 'quote-expired-002',
          expires_at: expiredExpiry,
          total_price: '0.1250000',
          price: '0.1240000',
          sell_asset: USD_ASSET,
          sell_amount: '125.00',
          buy_asset: 'stellar:native',
          buy_amount: '1000.0000000',
          fee: { total: '1.00', asset: USD_ASSET },
        }),
      });

    const oracle = new RfqPriceOracle({
      anchorBaseUrl: ANCHOR_URL,
      jwtToken: 'sep10-jwt-token',
      fetchFn: mockFetch as unknown as typeof fetch,
    });

    const converted = await oracle.convertFiatTargetToTokenStroops(
      '125.00',
      USD_ASSET,
      'stellar:native',
      {
        firm: true,
        nowMs,
      }
    );

    expect(converted.quoteId).toBe('quote-firm-001');
    expect(converted.tokenStroops).toBe(10_000_000_000n);

    await expect(
      oracle.convertFiatTargetToTokenStroops('125.00', USD_ASSET, 'stellar:native', {
        firm: true,
        nowMs,
      })
    ).rejects.toThrow(ValidationError);
  });

  it('rejects quotes when slippage exceeds configured maxSlippageBps', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        total_price: '1.0500000',
        price: '1.0500000',
        sell_amount: '100.00',
        buy_amount: '105.0000000', // 500 bps above expected 100 USDC ceiling
        fee: { total: '0.00', asset: USD_ASSET },
      }),
    });

    const oracle = new RfqPriceOracle({
      anchorBaseUrl: ANCHOR_URL,
      maxSlippageBps: 100, // 1% max
      fetchFn: mockFetch as unknown as typeof fetch,
    });

    await expect(
      oracle.convertFiatTargetToTokenStroops('100.00', USD_ASSET, USDC_ASSET, {
        expectedTokenAmount: '100',
      })
    ).rejects.toThrow(/slippage/i);
  });

  it('normalizes Reflector Oracle 14-decimal price feeds and rejects stale timestamps', () => {
    const nowMs = 1_750_000_000_000;
    const oracle = new RfqPriceOracle({
      anchorBaseUrl: ANCHOR_URL,
      maxQuoteAgeSeconds: 120,
    });

    // $0.1254321 per XLM with 14 decimals = 12_543_210_000_000n
    const normalized = oracle.normalizeReflectorQuote(
      {
        baseAsset: 'XLM',
        quoteAsset: 'USD',
        price: 12_543_210_000_000n,
        decimals: 14,
        timestamp: 1_749_999_950, // 50s ago
      },
      nowMs
    );

    expect(normalized.stroopsPerUnit).toBe(1_254_321n);
    expect(normalized.priceDecimal).toBe('0.1254321');

    expect(() =>
      oracle.normalizeReflectorQuote(
        {
          baseAsset: 'XLM',
          quoteAsset: 'USD',
          price: 12_543_210_000_000n,
          decimals: 14,
          timestamp: 1_749_999_000, // 1000s ago -> stale
        },
        nowMs
      )
    ).toThrow(ValidationError);
  });

  it('wraps HTTP error responses in NetworkError with status code', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 400,
      statusText: 'Bad Request',
      json: async () => ({ error: 'Unsupported asset pair' }),
    });

    const oracle = new RfqPriceOracle({
      anchorBaseUrl: ANCHOR_URL,
      fetchFn: mockFetch as unknown as typeof fetch,
    });

    await expect(
      oracle.getIndicativePrice({
        context: 'sep6',
        sellAsset: USD_ASSET,
        buyAsset: USDC_ASSET,
        sellAmount: '100',
      })
    ).rejects.toThrow(NetworkError);
  });
});
