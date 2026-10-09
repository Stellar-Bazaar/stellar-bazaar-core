const fs = require('fs');
const path = require('path');

const sdkPath = path.resolve(__dirname, '..', '..', 'stellar-bazaar-frontend', 'node_modules', '@stellar/stellar-sdk');
const {
  rpc,
  Networks,
  Keypair,
  Operation,
  TransactionBuilder,
  Address,
  nativeToScVal,
  scValToNative,
} = require(sdkPath);

const RPC_URL = 'https://soroban-testnet.stellar.org';
const PASSPHRASE = Networks.TESTNET;

async function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitForTx(server, hash, maxAttempts = 35) {
  for (let i = 0; i < maxAttempts; i++) {
    const res = await server.getTransaction(hash);
    if (res.status === 'SUCCESS') return res;
    if (res.status === 'FAILED') throw new Error(`Transaction ${hash} failed: ${JSON.stringify(res)}`);
    await sleep(2000);
  }
  throw new Error(`Transaction ${hash} timed out`);
}

async function main() {
  console.log('=== TESTING LIVE SOROBAN DEAL WORKFLOW ON TESTNET ===');
  const server = new rpc.Server(RPC_URL);

  const deployerPath = path.resolve(__dirname, '..', '.deployer.json');
  const deployer = Keypair.fromSecret(JSON.parse(fs.readFileSync(deployerPath, 'utf8')).secret);
  const deployment = JSON.parse(fs.readFileSync(path.resolve(__dirname, '..', 'deployments', 'testnet.json'), 'utf8'));

  const dealEngineId = deployment.dealEngine.contractId;
  console.log(`Bazaar Deal Engine: ${dealEngineId}`);

  // 1. Submit a Seller Offer
  console.log('\n--- 1. Submitting Seller Offer on-chain ---');
  let account = await server.getAccount(deployer.publicKey());
  const offerTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
    .addOperation(
      Operation.invokeContractFunction({
        contract: dealEngineId,
        function: 'submit_seller_offer',
        args: [
          new Address(deployer.publicKey()).toScVal(),
          nativeToScVal(1n, { type: 'u64' }), // deal_id 1
          nativeToScVal(20_000_000n, { type: 'i128' }), // 2.0 XLM unit price
          nativeToScVal(50, { type: 'u32' }), // volume 50
          nativeToScVal(5, { type: 'u32' }), // lead time days
        ],
      })
    )
    .setTimeout(60)
    .build();

  const preparedOffer = await server.prepareTransaction(offerTx);
  preparedOffer.sign(deployer);
  const offerSend = await server.sendTransaction(preparedOffer);
  const offerRes = await waitForTx(server, offerSend.hash);
  const offerId = Number(scValToNative(offerRes.returnValue));
  console.log(`Seller Offer #${offerId} submitted successfully! Tx: ${offerSend.hash}`);

  // 2. Commit Demand (Buyer Escrow Deposit)
  console.log('\n--- 2. Committing Buyer Demand to Escrow ---');
  account = await server.getAccount(deployer.publicKey());
  const commitTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
    .addOperation(
      Operation.invokeContractFunction({
        contract: dealEngineId,
        function: 'commit_demand',
        args: [
          new Address(deployer.publicKey()).toScVal(),
          nativeToScVal(1n, { type: 'u64' }), // deal_id 1
          nativeToScVal(20, { type: 'u32' }), // 20 units (Cost = 20 * 2 XLM = 40 XLM)
        ],
      })
    )
    .setTimeout(60)
    .build();

  const preparedCommit = await server.prepareTransaction(commitTx);
  preparedCommit.sign(deployer);
  const commitSend = await server.sendTransaction(preparedCommit);
  await waitForTx(server, commitSend.hash);
  console.log(`Buyer commitment of 20 units escrowed successfully! Tx: ${commitSend.hash}`);

  // 3. Query Deal State & Offers
  console.log('\n--- 3. Verifying authoritative Deal & Offer state ---');
  account = await server.getAccount(deployer.publicKey());
  const queryTx = new TransactionBuilder(account, { fee: '100', networkPassphrase: PASSPHRASE })
    .addOperation(
      Operation.invokeContractFunction({
        contract: dealEngineId,
        function: 'get_circle',
        args: [nativeToScVal(1n, { type: 'u64' })],
      })
    )
    .setTimeout(60)
    .build();
  const sim = await server.simulateTransaction(queryTx);
  const circleState = scValToNative(sim.result.retval);
  console.log('On-chain Deal State:', JSON.stringify(circleState, (k, v) => typeof v === 'bigint' ? v.toString() : v, 2));

  // Save hashes to testnet.json
  deployment.dealEngine.liveOfferTxHash = offerSend.hash;
  deployment.dealEngine.liveOfferId = offerId;
  deployment.dealEngine.liveCommitTxHash = commitSend.hash;
  fs.writeFileSync(path.resolve(__dirname, '..', 'deployments', 'testnet.json'), JSON.stringify(deployment, null, 2));
  console.log('Updated deployments/testnet.json with live offer & commit hashes');
}

main().catch((err) => {
  console.error('Execution error:', err);
  process.exit(1);
});
