# ShardX Python SDK

Python SDK for interacting with the ShardX blockchain platform.

## Installation

```bash
pip install shardx-sdk
```

## Usage

### Basic Usage

```python
from shardx_sdk import ShardXClient

# Create a client
client = ShardXClient(
    base_url="https://api.shardx.io/v1",
    api_key="YOUR_API_KEY"  # Optional
)

# Get node information
node_info = client.get_node_info()
print(node_info)
```

### Working with Wallets

```python
from shardx_sdk import ShardXClient, Wallet

# Create a client
client = ShardXClient()

# Create a random wallet
wallet = Wallet.create_random(client)
print(f"Address: {wallet.get_address()}")

# Create a wallet from private key
private_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
imported_wallet = Wallet(client, private_key=private_key)

# Create a wallet from mnemonic
mnemonic_wallet = Wallet.from_mnemonic(
    "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12",
    client
)

# Get wallet balance
balance = wallet.get_balance()
print(f"Balance: {balance}")

# Send a transaction
tx = wallet.create_transaction(
    to="recipient_address",
    amount="10.5",  # amount
    data="Optional data"
)
print(f"Transaction ID: {tx.id}")
```

### Transaction Management

```python
from shardx_sdk import ShardXClient, TransactionManager

# Create a client
client = ShardXClient()

# Create a transaction manager
tx_manager = TransactionManager(client)

# Wait for transaction confirmation
confirmed_tx = tx_manager.wait_for_confirmation("tx_id")
print(f"Confirmed transaction: {confirmed_tx}")

# Get transaction with analysis
tx_with_analysis = tx_manager.get_transaction_with_analysis("tx_id")
print(f"Transaction analysis: {tx_with_analysis.analysis}")

# Estimate transaction fee
fee = tx_manager.estimate_fee(
    "sender_address",
    "recipient_address",
    "10.5"
)
print(f"Estimated fee: {fee}")
```

### Multisig Wallets

```python
from shardx_sdk import ShardXClient, Wallet, MultisigManager

# Create a client
client = ShardXClient()

# Create a wallet
wallet = Wallet.create_random(client)

# Create a multisig manager
multisig_manager = MultisigManager(client)

# Create a multisig wallet
multisig_wallet = multisig_manager.create_wallet(
    wallet,
    "My Multisig Wallet",
    ["address1", "address2", "address3"],
    2  # required signatures
)
print(f"Multisig wallet ID: {multisig_wallet.id}")

# Create a multisig transaction
multisig_tx = multisig_manager.create_transaction(
    wallet,
    multisig_wallet.id,
    "recipient_address",
    "10.5"
)
print(f"Multisig transaction ID: {multisig_tx.id}")

# Sign a multisig transaction
signed_tx = multisig_manager.sign_transaction(wallet, multisig_tx.id)
print(f"Signatures collected: {len(signed_tx.signatures)}")

# Execute a multisig transaction
executed_tx = multisig_manager.execute_transaction(wallet, multisig_tx.id)
print(f"Executed transaction ID: {executed_tx.id}")
```

### AI Predictions

```python
from shardx_sdk import ShardXClient, AIPredictionManager

# Create a client
client = ShardXClient()

# Create an AI prediction manager
ai_manager = AIPredictionManager(client)

# Get available trading pairs
pairs = ai_manager.get_trading_pairs()
print(f"Available pairs: {pairs}")

# Get prediction for a trading pair
prediction = ai_manager.get_prediction("BTC/USD", period="day")
print(f"Current price: {prediction.current_price}")
print(f"Predicted price: {prediction.predicted_price}")
print(f"Confidence: {prediction.confidence}")

# Get trading recommendation
recommendation = ai_manager.get_trading_recommendation(prediction)
print(f"Recommendation: {recommendation.action}")
print(f"Reasoning: {recommendation.reasoning}")

# Calculate potential profit/loss
profit_loss = ai_manager.calculate_potential_profit_loss(
    prediction.current_price,
    prediction.predicted_price,
    "1000"  # investment amount
)
print(f"Potential profit/loss: {profit_loss.profit_loss}")
print(f"Percent change: {profit_loss.percent_change}%")
```

## API Reference

### ShardXClient

The main client for interacting with the ShardX API.

```python
client = ShardXClient(
    base_url="https://api.shardx.io/v1",
    api_key="YOUR_API_KEY",
    timeout=30
)
```

#### Methods

- `get_node_info()`: Get information about the node
- `get_network_stats()`: Get network statistics
- `get_shards()`: Get information about all shards
- `get_shard(shard_id)`: Get information about a specific shard
- `create_transaction(tx_data)`: Create a new transaction
- `get_transaction(tx_id)`: Get transaction by ID
- `get_transaction_status(tx_id)`: Get transaction status
- `get_transactions_by_address(address, limit, offset)`: Get transactions by address
- `get_block(hash_or_height)`: Get block by hash or height
- `get_latest_blocks(limit)`: Get latest blocks
- `get_account(address)`: Get account information
- `create_multisig_wallet(wallet_data)`: Create a new multisig wallet
- `get_multisig_wallet(wallet_id)`: Get multisig wallet by ID
- `get_multisig_wallets_by_owner(owner_address)`: Get multisig wallets by owner
- `get_prediction(pair, period)`: Get AI prediction for a trading pair
- `get_trading_pairs()`: Get available trading pairs
- `get_transaction_analysis(tx_id)`: Get detailed transaction analysis
- `get_chart_data(metric, period, from_time, to_time)`: Get advanced charts data

### Wallet

Manages keys and signing transactions.

```python
# Create a random wallet
wallet = Wallet.create_random(client)

# Create a wallet from private key
wallet = Wallet(client, private_key="your_private_key")

# Create a wallet from mnemonic
wallet = Wallet.from_mnemonic("your mnemonic phrase", client)
```

#### Methods

- `get_address()`: Get wallet address
- `get_public_key()`: Get wallet public key
- `sign(message)`: Sign a message
- `create_transaction(to, amount, data)`: Create and sign a transaction
- `get_balance()`: Get account balance
- `get_transactions(limit, offset)`: Get transaction history

### TransactionManager

Utility class for working with transactions.

```python
tx_manager = TransactionManager(client)
```

#### Methods

- `wait_for_confirmation(tx_id, timeout, interval)`: Wait for transaction confirmation
- `get_transaction_with_analysis(tx_id)`: Get transaction details with analysis
- `estimate_fee(from_addr, to_addr, amount, data)`: Estimate transaction fee
- `decode_transaction_data(data)`: Decode transaction data
- `encode_transaction_data(data)`: Encode transaction data

### MultisigManager

Utility class for working with multisig wallets.

```python
multisig_manager = MultisigManager(client)
```

#### Methods

- `create_wallet(wallet, name, signers, required_signatures)`: Create a new multisig wallet
- `get_wallet(wallet_id)`: Get multisig wallet by ID
- `get_wallets_by_owner(owner_address)`: Get multisig wallets by owner
- `create_transaction(wallet, multisig_id, to, amount, data)`: Create a multisig transaction
- `sign_transaction(wallet, tx_id)`: Sign a multisig transaction
- `execute_transaction(wallet, tx_id)`: Execute a multisig transaction

### AIPredictionManager

Utility class for working with AI predictions.

```python
ai_manager = AIPredictionManager(client)
```

#### Methods

- `get_trading_pairs()`: Get available trading pairs
- `get_prediction(pair, period)`: Get prediction for a trading pair
- `get_predictions(pairs, period)`: Get predictions for multiple trading pairs
- `get_price_history(pair, period, limit)`: Get price history for a trading pair
- `calculate_potential_profit_loss(current_price, predicted_price, investment)`: Calculate potential profit/loss
- `get_trading_recommendation(prediction)`: Get trading recommendation

## Error Handling

The SDK throws specific error types for different kinds of errors:

```python
from shardx_sdk import ShardXClient, ShardXError, TransactionError, WalletError

try:
    client = ShardXClient()
    node_info = client.get_node_info()
except TransactionError as e:
    print(f"Transaction error: {e}")
except WalletError as e:
    print(f"Wallet error: {e}")
except ShardXError as e:
    print(f"ShardX error: {e}, Code: {e.code}")
except Exception as e:
    print(f"Unknown error: {e}")
```

## License

MIT