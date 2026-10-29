export interface SorobanContractEvent {
  id: string;
  type: string;
  ledger: number;
  ledgerClosedAt: string;
  contractId: string;
  topics: string[];
  value: unknown;
  pagingToken: string;
}

export interface EventWatcherOptions {
  rpcUrl: string;
  contractId: string;
  startLedger?: number;
  pollIntervalMs?: number;
  onEvent: (event: SorobanContractEvent) => void;
  onError?: (err: Error) => void;
  fetchFn?: typeof fetch;
}

export class EventWatcher {
  private rpcUrl: string;
  private contractId: string;
  private currentLedger: number;
  private pollIntervalMs: number;
  private onEvent: (event: SorobanContractEvent) => void;
  private onError?: (err: Error) => void;
  private fetchFn: typeof fetch;

  private isRunning: boolean = false;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private lastPagingToken?: string;

  constructor(options: EventWatcherOptions) {
    this.rpcUrl = options.rpcUrl;
    this.contractId = options.contractId;
    this.currentLedger = options.startLedger ?? 0;
    this.pollIntervalMs = options.pollIntervalMs ?? 3000;
    this.onEvent = options.onEvent;
    this.onError = options.onError;
    this.fetchFn = options.fetchFn ?? globalThis.fetch;
  }

  public get active(): boolean {
    return this.isRunning;
  }

  public get cursor(): string | undefined {
    return this.lastPagingToken;
  }

  public start(): void {
    if (this.isRunning) return;
    this.isRunning = true;
    this.scheduleNext(0);
  }

  public stop(): void {
    this.isRunning = false;
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }

  public async pollOnce(): Promise<SorobanContractEvent[]> {
    try {
      const filters = [
        {
          type: 'contract',
          contractIds: [this.contractId],
        },
      ];

      const body: Record<string, unknown> = {
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'getEvents',
        params: {
          startLedger: this.currentLedger,
          filters,
          pagination: this.lastPagingToken ? { cursor: this.lastPagingToken } : undefined,
        },
      };

      const res = await this.fetchFn(this.rpcUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      if (!res.ok) {
        throw new Error(`Soroban RPC getEvents error: HTTP ${res.status}`);
      }

      const json = await res.json();
      if (json.error) {
        throw new Error(`Soroban RPC error: ${json.error.message || JSON.stringify(json.error)}`);
      }

      const events: SorobanContractEvent[] = json.result?.events || [];
      for (const ev of events) {
        if (ev.pagingToken) {
          this.lastPagingToken = ev.pagingToken;
        }
        if (ev.ledger && ev.ledger >= this.currentLedger) {
          this.currentLedger = ev.ledger + 1;
        }
        this.onEvent(ev);
      }

      return events;
    } catch (err) {
      if (this.onError) {
        this.onError(err instanceof Error ? err : new Error(String(err)));
      }
      return [];
    }
  }

  private scheduleNext(delayMs: number): void {
    if (!this.isRunning) return;
    this.timer = setTimeout(async () => {
      await this.pollOnce();
      if (this.isRunning) {
        this.scheduleNext(this.pollIntervalMs);
      }
    }, delayMs);
  }
}
