import { describe, it, expect, vi } from 'vitest';
import { EventWatcher, SorobanContractEvent } from '../src/event-watcher';

describe('EventWatcher', () => {
  const sampleEvent: SorobanContractEvent = {
    id: '0000000001',
    type: 'contract',
    ledger: 1050,
    ledgerClosedAt: '2026-10-29T00:00:00Z',
    contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
    topics: ['circle_created', '1'],
    value: { creator: 'GBUYER123' },
    pagingToken: '1050-00001',
  };

  it('polls events and advances cursor & ledger', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        result: {
          events: [sampleEvent],
        },
      }),
    });

    const receivedEvents: SorobanContractEvent[] = [];
    const watcher = new EventWatcher({
      rpcUrl: 'https://soroban-testnet.stellar.org',
      contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
      startLedger: 1000,
      onEvent: (ev) => receivedEvents.push(ev),
      fetchFn: mockFetch as unknown as typeof fetch,
    });

    expect(watcher.active).toBe(false);
    expect(watcher.cursor).toBeUndefined();

    const polled = await watcher.pollOnce();
    expect(polled.length).toBe(1);
    expect(receivedEvents.length).toBe(1);
    expect(receivedEvents[0].id).toBe('0000000001');
    expect(watcher.cursor).toBe('1050-00001');

    // Verify fetch arguments
    expect(mockFetch).toHaveBeenCalledTimes(1);
    const [, options] = mockFetch.mock.calls[0];
    const parsedBody = JSON.parse(options.body as string);
    expect(parsedBody.method).toBe('getEvents');
    expect(parsedBody.params.startLedger).toBe(1000);
  });

  it('handles fetch error and calls onError handler', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 503,
    });

    const errors: Error[] = [];
    const watcher = new EventWatcher({
      rpcUrl: 'https://soroban-testnet.stellar.org',
      contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
      onEvent: () => {},
      onError: (err) => errors.push(err),
      fetchFn: mockFetch as unknown as typeof fetch,
    });

    const events = await watcher.pollOnce();
    expect(events.length).toBe(0);
    expect(errors.length).toBe(1);
    expect(errors[0].message).toContain('HTTP 503');
  });

  it('starts and stops gracefully', () => {
    const watcher = new EventWatcher({
      rpcUrl: 'https://soroban-testnet.stellar.org',
      contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
      onEvent: () => {},
    });

    watcher.start();
    expect(watcher.active).toBe(true);
    watcher.stop();
    expect(watcher.active).toBe(false);
  });
});

