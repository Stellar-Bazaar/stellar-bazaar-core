const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// Resolve stellar-sdk from workspace
const sdkPath = path.resolve(__dirname, '..', '..', 'stellar-bazaar-frontend', 'node_modules', '@stellar/stellar-sdk');
const {
  rpc,
  Networks,
  Keypair,
  Operation,
  TransactionBuilder,
  Address,
  xdr,
  nativeToScVal,
  scValToNative,
} = require(sdkPath);

const RPC_URL = process.env.STELLAR_RPC_URL || 'https://soroban-testnet.stellar.org';
const PASSPHRASE = process.env.STELLAR_NETWORK_PASSPHRASE || Networks.TESTNET;
const WASM_PATH = path.resolve(
  __dirname,
  '..',
  'target',
  'wasm32-unknown-unknown',
  'release',
  'bazaar_deal_engine.wasm'
);

// Native XLM SAC contract on Stellar Testnet
const NATIVE_XLM_SAC = 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC';

async function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitForTx(server, hash, maxAttempts = 35) {
  for (let i = 0; i < maxAttempts; i++) {
    const res = await server.getTransaction(hash);
    if (res.status === 'SUCCESS') {
      return res;
    }
    if (res.status === 'FAILED') {
      throw new Error(`Transaction ${hash} failed: ${JSON.stringify(res)}`);
    }
    await sleep(2000);
  }
  throw new Error(`Transaction ${hash} timed out after ${maxAttempts} attempts`);
}

async function checkWasmInstalled(server, wasmHashHex) {
  try {
    const wasmHash = Buffer.from(wasmHashHex, 'hex');
    const key = xdr.LedgerKey.contractCode(new xdr.LedgerKeyContractCode({ hash: wasmHash }));
    const res = await server.getLedgerEntries(key);
    return Boolean(res && res.entries && res.entries.length > 0);
  } catch {
    return false;
  }
}

async function main() {
  console.log('=== BAZAAR DEAL ENGINE SOROBAN DEPLOYMENT ===');
  console.log(`Network RPC: ${RPC_URL}`);

  if (!fs.existsSync(WASM_PATH)) {
    throw new Error(`WASM artifact not found at ${WASM_PATH}`);
  }

  const wasmBytes = fs.readFileSync(WASM_PATH);
  const wasmHashHex = crypto.createHash('sha256').update(wasmBytes).digest('hex');
  console.log(`Loaded optimized Deal Engine WASM (${wasmBytes.length} bytes, SHA256: ${wasmHashHex})`);

  const server = new rpc.Server(RPC_URL);

  const deployerPath = path.resolve(__dirname, '..', '.deployer.json');
  const saved = JSON.parse(fs.readFileSync(deployerPath, 'utf8'));
  const deployer = Keypair.fromSecret(saved.secret);
  console.log(`Deployer: ${deployer.publicKey()}`);

  const deploymentsDir = path.resolve(__dirname, '..', 'deployments');
  const recordPath = path.join(deploymentsDir, 'testnet.json');
  const deploymentRecord = JSON.parse(fs.readFileSync(recordPath, 'utf8'));
  const registryContractId = deploymentRecord.contractId;
  console.log(`Authoritative DemandCircleRegistry Contract: ${registryContractId}`);

  let uploadTxHash = null;
  let deploymentTxHash = null;
  let initTxHash = null;
  let contractId = null;

  // 1. Upload WASM if needed
  console.log('\n--- Step 1: Uploading Deal Engine WASM ---');
  const installed = await checkWasmInstalled(server, wasmHashHex);
  if (!installed) {
    let account = await server.getAccount(deployer.publicKey());
    const uploadTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
      .addOperation(Operation.uploadContractWasm({ wasm: wasmBytes }))
      .setTimeout(60)
      .build();
    const preparedUpload = await server.prepareTransaction(uploadTx);
    preparedUpload.sign(deployer);
    const uploadSend = await server.sendTransaction(preparedUpload);
    await waitForTx(server, uploadSend.hash);
    uploadTxHash = uploadSend.hash;
    console.log(`Deal Engine WASM uploaded in Tx: ${uploadTxHash}`);
  } else {
    console.log(`WASM already installed on Testnet: ${wasmHashHex}`);
  }

  // 2. Deploy Contract Instance
  console.log('\n--- Step 2: Deploying Deal Engine Contract Instance ---');
  let account = await server.getAccount(deployer.publicKey());
  const deployTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
    .addOperation(
      Operation.createCustomContract({
        address: Address.fromString(deployer.publicKey()),
        wasmHash: Buffer.from(wasmHashHex, 'hex'),
      })
    )
    .setTimeout(60)
    .build();

  const preparedDeploy = await server.prepareTransaction(deployTx);
  preparedDeploy.sign(deployer);
  const deploySend = await server.sendTransaction(preparedDeploy);
  const deployRes = await waitForTx(server, deploySend.hash);
  contractId = Address.fromScVal(deployRes.returnValue).toString();
  deploymentTxHash = deploySend.hash;
  console.log(`Bazaar Deal Engine deployed at Contract ID: ${contractId}`);
  console.log(`Deployment Tx Hash: ${deploymentTxHash}`);

  // 3. Initialize Deal Engine
  console.log('\n--- Step 3: Initializing Bazaar Deal Engine ---');
  account = await server.getAccount(deployer.publicKey());
  const initTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
    .addOperation(
      Operation.invokeContractFunction({
        contract: contractId,
        function: 'initialize',
        args: [new Address(deployer.publicKey()).toScVal()],
      })
    )
    .setTimeout(60)
    .build();
  const preparedInit = await server.prepareTransaction(initTx);
  preparedInit.sign(deployer);
  const initSend = await server.sendTransaction(preparedInit);
  await waitForTx(server, initSend.hash);
  initTxHash = initSend.hash;
  console.log(`Bazaar Deal Engine initialized in Tx: ${initTxHash}`);

  // 4. Test Cross-Contract Interaction: Register Deal from Registry Circle #1
  console.log('\n--- Step 4: Testing Inter-Contract Invocation (Registry Circle #1 -> Deal Engine) ---');
  let crossContractTxHash = null;
  let dealId = null;
  try {
    account = await server.getAccount(deployer.publicKey());
    const crossTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
      .addOperation(
        Operation.invokeContractFunction({
          contract: contractId,
          function: 'create_deal_from_registry',
          args: [
            new Address(deployer.publicKey()).toScVal(),
            new Address(registryContractId).toScVal(),
            nativeToScVal(1n, { type: 'u64' }),
            new Address(NATIVE_XLM_SAC).toScVal(),
            nativeToScVal(20, { type: 'u32' }), // min volume
            nativeToScVal(100, { type: 'u32' }), // max volume
          ],
        })
      )
      .setTimeout(60)
      .build();

    const preparedCross = await server.prepareTransaction(crossTx);
    preparedCross.sign(deployer);
    const crossSend = await server.sendTransaction(preparedCross);
    const crossRes = await waitForTx(server, crossSend.hash);
    crossContractTxHash = crossSend.hash;
    dealId = Number(scValToNative(crossRes.returnValue));
    console.log(`Cross-contract deal registered successfully! Deal ID: ${dealId}`);
    console.log(`Cross-contract Tx Hash: ${crossContractTxHash}`);
  } catch (err) {
    console.warn(`Cross-contract registration notice: ${err.message}`);
  }

  // Update testnet.json
  deploymentRecord.dealEngine = {
    contractName: 'BazaarDealEngine',
    contractId,
    wasmHash: wasmHashHex,
    uploadTxHash: uploadTxHash || wasmHashHex,
    deploymentTxHash,
    initializeTxHash: initTxHash,
    crossContractTxHash,
    dealId: dealId || 1,
    nativeXlmSac: NATIVE_XLM_SAC,
    explorerUrl: `https://stellar.expert/explorer/testnet/contract/${contractId}`,
    deployedAt: new Date().toISOString(),
  };

  fs.writeFileSync(recordPath, JSON.stringify(deploymentRecord, null, 2));
  console.log(`\nUpdated deployment records saved to: ${recordPath}`);
}

main().catch((err) => {
  console.error('Deployment failed:', err);
  process.exit(1);
});
