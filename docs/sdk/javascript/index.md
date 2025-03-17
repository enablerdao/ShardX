# ShardX JavaScript SDK

JavaScript/TypeScript SDK for interacting with the ShardX blockchain platform.

## Installation

```bash
npm install shardx-sdk
```

## Usage

### Basic Usage

```typescript
import { ShardXClient } from 'shardx-sdk';

// Create a client
const client = new ShardXClient({
  baseUrl: 'https://api.shardx.io/v1',
  apiKey: 'YOUR_API_KEY' // Optional
});

// Get node information
const nodeInfo = await client.getNodeInfo();
console.log(nodeInfo);
```

### Working with Wallets

```typescript
import { ShardXClient, Wallet } from 'shardx-sdk';

// Create a client
const client = new ShardXClient();

// Create a random wallet
const wallet = Wallet.createRandom(client);
console.log('Address:', wallet.getAddress());

// Create a wallet from private key
const privateKey = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const importedWallet = new Wallet({ client, privateKey });

// Create a wallet from mnemonic
const mnemonicWallet = Wallet.fromMnemonic('word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12', client);

// Get wallet balance
const balance = await wallet.getBalance();
console.log('Balance:', balance);

// Send a transaction
const tx = await wallet.createTransaction(
  'recipient_address',
  '10.5', // amount
  'Optional data'
);
console.log('Transaction ID:', tx.id);
```

### Transaction Management

```typescript
import { ShardXClient, TransactionManager } from 'shardx-sdk';

// Create a client
const client = new ShardXClient();

// Create a transaction manager
const txManager = new TransactionManager(client);

// Wait for transaction confirmation
const confirmedTx = await txManager.waitForConfirmation('tx_id');
console.log('Confirmed transaction:', confirmedTx);

// Get transaction with analysis
const txWithAnalysis = await txManager.getTransactionWithAnalysis('tx_id');
console.log('Transaction analysis:', txWithAnalysis.analysis);

// Estimate transaction fee
const fee = await txManager.estimateFee(
  'sender_address',
  'recipient_address',
  '10.5'
);
console.log('Estimated fee:', fee);
```

### Multisig Wallets

```typescript
import { ShardXClient, Wallet, MultisigManager } from 'shardx-sdk';

// Create a client
const client = new ShardXClient();

// Create a wallet
const wallet = Wallet.createRandom(client);

// Create a multisig manager
const multisigManager = new MultisigManager(client);

// Create a multisig wallet
const multisigWallet = await multisigManager.createWallet(
  wallet,
  'My Multisig Wallet',
  ['address1', 'address2', 'address3'],
  2 // required signatures
);
console.log('Multisig wallet ID:', multisigWallet.id);

// Create a multisig transaction
const multisigTx = await multisigManager.createTransaction(
  wallet,
  multisigWallet.id,
  'recipient_address',
  '10.5'
);
console.log('Multisig transaction ID:', multisigTx.id);

// Sign a multisig transaction
const signedTx = await multisigManager.signTransaction(wallet, multisigTx.id);
console.log('Signatures collected:', signedTx.signatures.length);

// Execute a multisig transaction
const executedTx = await multisigManager.executeTransaction(wallet, multisigTx.id);
console.log('Executed transaction ID:', executedTx.id);
```

### AI Predictions

```typescript
import { ShardXClient, AIPredictionManager } from 'shardx-sdk';

// Create a client
const client = new ShardXClient();

// Create an AI prediction manager
const aiManager = new AIPredictionManager(client);

// Get available trading pairs
const pairs = await aiManager.getTradingPairs();
console.log('Available pairs:', pairs);

// Get prediction for a trading pair
const prediction = await aiManager.getPrediction('BTC/USD', 'day');
console.log('Current price:', prediction.currentPrice);
console.log('Predicted price:', prediction.predictedPrice);
console.log('Confidence:', prediction.confidence);

// Get trading recommendation
const recommendation = aiManager.getTradingRecommendation(prediction);
console.log('Recommendation:', recommendation.action);
console.log('Reasoning:', recommendation.reasoning);

// Calculate potential profit/loss
const profitLoss = aiManager.calculatePotentialProfitLoss(
  prediction.currentPrice,
  prediction.predictedPrice,
  '1000' // investment amount
);
console.log('Potential profit/loss:', profitLoss.profitLoss);
console.log('Percent change:', profitLoss.percentChange + '%');
```

## API Reference

### ShardXClient

The main client for interacting with the ShardX API.

```typescript
const client = new ShardXClient({
  baseUrl: 'https://api.shardx.io/v1',
  apiKey: 'YOUR_API_KEY',
  timeout: 30000
});
```

#### Methods

- `getNodeInfo()`: Get information about the node
- `getNetworkStats()`: Get network statistics
- `getShards()`: Get information about all shards
- `getShard(shardId)`: Get information about a specific shard
- `createTransaction(txData)`: Create a new transaction
- `getTransaction(txId)`: Get transaction by ID
- `getTransactionStatus(txId)`: Get transaction status
- `getTransactionsByAddress(address, limit, offset)`: Get transactions by address
- `getBlock(hashOrHeight)`: Get block by hash or height
- `getLatestBlocks(limit)`: Get latest blocks
- `getAccount(address)`: Get account information
- `createMultisigWallet(walletData)`: Create a new multisig wallet
- `getMultisigWallet(walletId)`: Get multisig wallet by ID
- `getMultisigWalletsByOwner(ownerAddress)`: Get multisig wallets by owner
- `getPrediction(pair, period)`: Get AI prediction for a trading pair
- `getTradingPairs()`: Get available trading pairs
- `getTransactionAnalysis(txId)`: Get detailed transaction analysis
- `getChartData(metric, period, from, to)`: Get advanced charts data

### Wallet

Manages keys and signing transactions.

```typescript
// Create a random wallet
const wallet = Wallet.createRandom(client);

// Create a wallet from private key
const wallet = new Wallet({
  client,
  privateKey: 'your_private_key'
});

// Create a wallet from mnemonic
const wallet = Wallet.fromMnemonic('your mnemonic phrase', client);
```

#### Methods

- `getAddress()`: Get wallet address
- `getPublicKey()`: Get wallet public key
- `sign(message)`: Sign a message
- `createTransaction(to, amount, data)`: Create and sign a transaction
- `getBalance()`: Get account balance
- `getTransactions(limit, offset)`: Get transaction history

### TransactionManager

Utility class for working with transactions.

```typescript
const txManager = new TransactionManager(client);
```

#### Methods

- `waitForConfirmation(txId, timeout, interval)`: Wait for transaction confirmation
- `getTransactionWithAnalysis(txId)`: Get transaction details with analysis
- `estimateFee(from, to, amount, data)`: Estimate transaction fee
- `decodeTransactionData(data)`: Decode transaction data
- `encodeTransactionData(data)`: Encode transaction data

### MultisigManager

Utility class for working with multisig wallets.

```typescript
const multisigManager = new MultisigManager(client);
```

#### Methods

- `createWallet(wallet, name, signers, requiredSignatures)`: Create a new multisig wallet
- `getWallet(walletId)`: Get multisig wallet by ID
- `getWalletsByOwner(ownerAddress)`: Get multisig wallets by owner
- `createTransaction(wallet, multisigId, to, amount, data)`: Create a multisig transaction
- `signTransaction(wallet, txId)`: Sign a multisig transaction
- `executeTransaction(wallet, txId)`: Execute a multisig transaction

### AIPredictionManager

Utility class for working with AI predictions.

```typescript
const aiManager = new AIPredictionManager(client);
```

#### Methods

- `getTradingPairs()`: Get available trading pairs
- `getPrediction(pair, period)`: Get prediction for a trading pair
- `getPredictions(pairs, period)`: Get predictions for multiple trading pairs
- `getPriceHistory(pair, period, limit)`: Get price history for a trading pair
- `calculatePotentialProfitLoss(currentPrice, predictedPrice, investment)`: Calculate potential profit/loss
- `getTradingRecommendation(prediction)`: Get trading recommendation

## Error Handling

The SDK throws specific error types for different kinds of errors:

```typescript
import { ShardXClient, ShardXError, TransactionError, WalletError } from 'shardx-sdk';

try {
  const client = new ShardXClient();
  const nodeInfo = await client.getNodeInfo();
} catch (error) {
  if (error instanceof TransactionError) {
    console.error('Transaction error:', error.message);
  } else if (error instanceof WalletError) {
    console.error('Wallet error:', error.message);
  } else if (error instanceof ShardXError) {
    console.error('ShardX error:', error.message, 'Code:', error.code);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## License

MIT