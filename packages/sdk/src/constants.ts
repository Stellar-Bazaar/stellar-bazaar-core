import { NetworkId, NetworkProfile } from './types';

export const STELLAR_DECIMALS = 7;
export const STROOPS_PER_XLM = 10_000_000n;
export const STELLAR_LEDGER_SECONDS = 5;

export const NETWORKS: Record<NetworkId, NetworkProfile> = {
  testnet: {
    id: 'testnet',
    name: 'Stellar Testnet',
    rpcUrl: 'https://soroban-testnet.stellar.org',
    horizonUrl: 'https://horizon-testnet.stellar.org',
    networkPassphrase: 'Test SDF Network ; September 2015',
  },
  futurenet: {
    id: 'futurenet',
    name: 'Stellar Futurenet',
    rpcUrl: 'https://rpc-futurenet.stellar.org',
    horizonUrl: 'https://horizon-futurenet.stellar.org',
    networkPassphrase: 'Test SDF Future Network ; October 2022',
  },
  local: {
    id: 'local',
    name: 'Local Standalone Node',
    rpcUrl: 'http://localhost:8000/soroban/rpc',
    horizonUrl: 'http://localhost:8000',
    networkPassphrase: 'Standalone Network ; February 2024',
  },
};

export const CONTRACT_ADDRESSES: Record<NetworkId, { dealEngine: string; circleRegistry: string }> = {
  testnet: {
    dealEngine: 'CBLJ2Y7PVR64F7V4GZJ5332YVDFO44V4V5P2O7C6X7NXXS4E77LNZD2E',
    circleRegistry: 'CCK63X7JRP6V4O3BJVCFM76M5B4H7L6D2G5B4H7L6D2G5B4H7L6D2G5B',
  },
  futurenet: {
    dealEngine: 'CBLJ2Y7PVR64F7V4GZJ5332YVDFO44V4V5P2O7C6X7NXXS4E77LNZD2E',
    circleRegistry: 'CCK63X7JRP6V4O3BJVCFM76M5B4H7L6D2G5B4H7L6D2G5B4H7L6D2G5B',
  },
  local: {
    dealEngine: 'CBLJ2Y7PVR64F7V4GZJ5332YVDFO44V4V5P2O7C6X7NXXS4E77LNZD2E',
    circleRegistry: 'CCK63X7JRP6V4O3BJVCFM76M5B4H7L6D2G5B4H7L6D2G5B4H7L6D2G5B',
  },
};
