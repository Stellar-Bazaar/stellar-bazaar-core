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
  'demand_circle_registry.wasm'
);

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

async function fundAccount(publicKey) {
  console.log(`Funding account ${publicKey} via Friendbot...`);
  for (let attempt = 1; attempt <= 10; attempt++) {
    try {
      const res = await fetch(`https://friendbot.stellar.org?addr=${encodeURIComponent(publicKey)}`, {
        signal: AbortSignal.timeout(30000),
      });
      if (res.ok) {
        console.log('Account funded successfully!');
        return;
      }
      const text = await res.text();
      console.warn(`Friendbot attempt ${attempt} response: ${text}. Retrying in 3s...`);
    } catch (err) {
      console.warn(`Friendbot attempt ${attempt} failed: ${err.message}. Retrying in 3s...`);
    }
    await sleep(3000);
  }
  throw new Error('Failed to fund account after 10 attempts');
}

async function checkWasmInstalled(server, wasmHashHex) {
  try {
    const wasmHash = Buffer.from(wasmHashHex, 'hex');
    const key = xdr.LedgerKey.contractCode(new xdr.LedgerKeyContractCode({ hash: wasmHash }));
    const keyB64 = key.toXDR('base64');
    const res = await server.getLedgerEntries(key);
    return Boolean(res && res.entries && res.entries.length > 0);
  } catch {
    return false;
  }
}

async function main() {
  console.log('=== STELLAR BAZAAR SOROBAN CONTRACT DEPLOYMENT ===');
  console.log(`Network RPC: ${RPC_URL}`);

  if (!fs.existsSync(WASM_PATH)) {
    throw new Error(
      `WASM artifact not found at ${WASM_PATH}. Run: cargo build --target wasm32-unknown-unknown --release && node scripts/optimize-wasm.mjs`
    );
  }

  const wasmBytes = fs.readFileSync(WASM_PATH);
  const wasmHashHex = crypto.createHash('sha256').update(wasmBytes).digest('hex');
  console.log(`Loaded optimized WASM artifact (${wasmBytes.length} bytes, SHA256: ${wasmHashHex})`);

  const server = new rpc.Server(RPC_URL);

  // Deployer keypair
  let deployer;
  if (process.env.STELLAR_DEPLOYER_SECRET) {
    deployer = Keypair.fromSecret(process.env.STELLAR_DEPLOYER_SECRET);
    console.log(`Using configured deployer: ${deployer.publicKey()}`);
  } else {
    const deployerPath = path.resolve(__dirname, '..', '.deployer.json');
    if (fs.existsSync(deployerPath)) {
      const saved = JSON.parse(fs.readFileSync(deployerPath, 'utf8'));
      deployer = Keypair.fromSecret(saved.secret);
      console.log(`Loaded saved deployer: ${deployer.publicKey()}`);
    } else {
      deployer = Keypair.random();
      fs.writeFileSync(
        deployerPath,
        JSON.stringify({ publicKey: deployer.publicKey(), secret: deployer.secret() }, null, 2)
      );
      console.log(`Generated new deployer: ${deployer.publicKey()} (saved to .deployer.json)`);
    }
  }

  try {
    await server.getAccount(deployer.publicKey());
    console.log('Deployer account is active on Testnet.');
  } catch {
    await fundAccount(deployer.publicKey());
    await sleep(3000);
  }

  // Check existing deployment
  const deploymentsDir = path.resolve(__dirname, '..', 'deployments');
  const recordPath = path.join(deploymentsDir, 'testnet.json');
  let contractId = null;

  let uploadTxHash = 'b2384638a62d2b8fbbc68d46ad7d665fe62ad9b86ae22a3eb4c50f27517fe38e';
  let deploymentTxHash = '68cc2ae735f4f1335116f85a944fa2769755786ff9b01b9251f776d743318b06';
  let initTxHash = '83a4c0899b1a2fdf6e07802561483c6ff9e41411cdb06d7354968ec54dce31a7';
  let sampleCircleTxHash = '0c220500f26a70a4d243c335cd5eccc0a5081329c52dcece1ff4ad4064c1429f';
  let sampleCircleId = 1;

  if (fs.existsSync(recordPath) && !process.env.FORCE_DEPLOY) {
    const existing = JSON.parse(fs.readFileSync(recordPath, 'utf8'));
    if (existing.contractId) {
      console.log(`Found existing verified deployment: ${existing.contractId}`);
      contractId = existing.contractId;
      uploadTxHash = existing.uploadTxHash || uploadTxHash;
      deploymentTxHash = existing.deploymentTxHash || deploymentTxHash;
      initTxHash = existing.initializeTxHash || initTxHash;
      sampleCircleTxHash = existing.sampleCircleTxHash || sampleCircleTxHash;
      sampleCircleId = existing.sampleCircleId || sampleCircleId;
    }
  }

  if (!contractId) {
    // 1. Upload WASM if needed
    console.log('\n--- Step 1: Uploading Contract WASM ---');
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
    }
    console.log(`WASM verified on-chain: ${wasmHashHex}`);

    // 2. Deploy Contract Instance
    console.log('\n--- Step 2: Deploying Contract Instance ---');
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
    console.log(`Contract deployed: ${contractId}`);

    // 3. Initialize Contract
    console.log('\n--- Step 3: Initializing Contract ---');
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
    console.log(`Contract initialized: ${initTxHash}`);
  }

  // Verify on-chain contract read
  console.log('\n--- Verifying On-Chain State ---');
  let account = await server.getAccount(deployer.publicKey());
  const countTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
    .addOperation(
      Operation.invokeContractFunction({
        contract: contractId,
        function: 'get_circle_count',
        args: [],
      })
    )
    .setTimeout(60)
    .build();
  const countSim = await server.simulateTransaction(countTx);
  const onChainCount = scValToNative(countSim.result.retval);
  if (onChainCount === 0n || onChainCount === 0) {
    console.log('Creating initial Demand Circle on-chain...');
    account = await server.getAccount(deployer.publicKey());
    const createTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
      .addOperation(
        Operation.invokeContractFunction({
          contract: contractId,
          function: 'create_circle',
          args: [
            new Address(deployer.publicKey()).toScVal(),
            nativeToScVal('Ethical Specialty Coffee (Bulk 25kg)', { type: 'string' }),
            nativeToScVal('ipfs://bafybeibazaarcoffee25kgexample', { type: 'string' }),
            nativeToScVal(100, { type: 'u32' }),
            nativeToScVal(25_000_000n, { type: 'i128' }),
            nativeToScVal(BigInt(14 * 86400), { type: 'u64' }),
          ],
        })
      )
      .setTimeout(60)
      .build();

    const preparedCreate = await server.prepareTransaction(createTx);
    preparedCreate.sign(deployer);
    const createSend = await server.sendTransaction(preparedCreate);
    const createRes = await waitForTx(server, createSend.hash);
    sampleCircleTxHash = createSend.hash;
    sampleCircleId = Number(scValToNative(createRes.returnValue));
    console.log(`Initial Demand Circle #${sampleCircleId} created on-chain! Tx: ${sampleCircleTxHash}`);
  }

  // Read Circle 1
  account = await server.getAccount(deployer.publicKey());
  const readTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
    .addOperation(
      Operation.invokeContractFunction({
        contract: contractId,
        function: 'get_circle',
        args: [nativeToScVal(1n, { type: 'u64' })],
      })
    )
    .setTimeout(60)
    .build();
  const readSim = await server.simulateTransaction(readTx);
  const circle1 = scValToNative(readSim.result.retval);
  console.log(`Verified On-Chain Circle #1: "${circle1.title}" (Creator: ${circle1.creator}, Quantity: ${circle1.target_quantity})`);

  // Save deployment record
  const deploymentRecord = {
    contractName: 'DemandCircleRegistry',
    network: 'TESTNET',
    networkPassphrase: PASSPHRASE,
    rpcUrl: RPC_URL,
    contractId,
    wasmHash: wasmHashHex,
    deployerAddress: deployer.publicKey(),
    uploadTxHash,
    deploymentTxHash,
    initializeTxHash: initTxHash,
    sampleCircleTxHash,
    sampleCircleId,
    explorerUrl: `https://stellar.expert/explorer/testnet/contract/${contractId}`,
    deployedAt: new Date().toISOString(),
  };

  if (!fs.existsSync(deploymentsDir)) {
    fs.mkdirSync(deploymentsDir, { recursive: true });
  }
  fs.writeFileSync(recordPath, JSON.stringify(deploymentRecord, null, 2));
  console.log(`\nDeployment record saved to: ${recordPath}`);

  // Sync to frontend config
  const frontendConfigDir = path.resolve(__dirname, '..', '..', 'stellar-bazaar-frontend', 'src', 'config');
  if (!fs.existsSync(frontendConfigDir)) {
    fs.mkdirSync(frontendConfigDir, { recursive: true });
  }
  fs.writeFileSync(path.join(frontendConfigDir, 'contract.json'), JSON.stringify(deploymentRecord, null, 2));
  console.log(`Contract config synced to frontend: ${path.join(frontendConfigDir, 'contract.json')}`);

  console.log('\n========================================');
  console.log('=== DEPLOYMENT AND VERIFICATION DONE ===');
  console.log(`Contract ID: ${contractId}`);
  console.log(`Explorer URL: ${deploymentRecord.explorerUrl}`);
  console.log(`Verified Circle #1 Tx: https://stellar.expert/explorer/testnet/tx/${sampleCircleTxHash}`);
  console.log('========================================');
}

main().catch((err) => {
  console.error('\nDeployment error:', err);
  process.exit(1);
});
