import { describe, it, expect } from 'vitest';
import { StellarBazaarClient } from '../src/client';

describe('StellarBazaarClient', () => {
  it('initializes with default Testnet configuration', () => {
    const client = new StellarBazaarClient();
    const net = client.getNetwork();

    expect(net.id).toBe('testnet');
    expect(net.rpcUrl).toContain('soroban-testnet.stellar.org');
    expect(client.getDealEngineAddress().length).toBeGreaterThan(0);
  });

  it('supports custom network overrides', () => {
    const client = new StellarBazaarClient({
      network: 'local',
      customRpcUrl: 'http://my-node:8000/soroban/rpc',
      dealEngineAddress: 'C_CUSTOM_DEAL_ENGINE',
    });

    const net = client.getNetwork();
    expect(net.id).toBe('local');
    expect(net.rpcUrl).toBe('http://my-node:8000/soroban/rpc');
    expect(client.getDealEngineAddress()).toBe('C_CUSTOM_DEAL_ENGINE');
  });

  it('builds valid invocation payloads for contract methods', () => {
    const client = new StellarBazaarClient();
    const creator = 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN';
    const token = 'CCFPZOCU33AWX2NKX47XD6W5JNYFP7MU57DTQFB5XOOQSJLSSC4PMX25';

    const createTx = client.buildCreateCircleInvocation(
      creator,
      token,
      10_000_000n,
      10,
      50,
      86400
    );

    expect(createTx.method).toBe('create_circle');
    expect(createTx.args[0]).toBe(creator);
    expect(createTx.args[2]).toBe(10_000_000n);

    const commitTx = client.buildCommitDemandInvocation(creator, 42, 5);
    expect(commitTx.method).toBe('commit_demand');
    expect(commitTx.args[1]).toBe(42n);
    expect(commitTx.args[2]).toBe(5);

    const disputeTx = client.buildDisputeDealInvocation(creator, 42, 'Damaged');
    expect(disputeTx.method).toBe('dispute_deal');
    expect(disputeTx.args[2]).toBe('Damaged');
  });
});
