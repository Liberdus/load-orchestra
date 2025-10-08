# load-orchestra

A load testing tool for Liberdus blockchain transactions. Supports automated account registration, transaction injection, and account reuse for efficient testing.

## Quick Start

1. **Set up environment** (Required):
   ```bash
   cp .env.example .env
   # Edit .env to set your NETWORK_ID
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run a basic message test**:
   ```bash
   ./target/release/load-orchestra sustain_load \
     --tx_type message \
     --tps 1 \
     --eoa 3 \
     --duration 30 \
     --gateway_url https://dev.liberdus.com:3030
   ```

```
Usage: load-orchestra [COMMAND]

Commands:
  sustain_load   Inject Transactions for a duration
  stake          Staking nodes
  change_config  Change the configuration of the network
  tui            Starts the TUI, (still in development)
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Message Transaction Examples

### Basic Message Transaction Test

Send message transactions for 60 seconds at 2 TPS using 5 accounts:

```bash
./target/debug/load-orchestra sustain_load \
  --tx_type message \
  --tps 2 \
  --eoa 5 \
  --duration 60 \
  --gateway_url https://dev.liberdus.com:3030
```

### Verbose Output with Account Registration Details

Run with detailed logging to see transaction payloads and responses:

```bash
./target/debug/load-orchestra sustain_load \
  --tx_type message \
  --tps 1 \
  --eoa 3 \
  --eoa_tps 2 \
  --duration 30 \
  --verbose \
  --gateway_url https://dev.liberdus.com:3030
```

Expected output includes:
- Account registration progress
- Transaction payloads in JSON format
- Raw server responses
- Success/failure statistics

### Account Reuse for Faster Testing

After running tests once, reuse previously registered accounts to skip registration:

```bash
./target/debug/load-orchestra sustain_load \
  --tx_type message \
  --tps 5 \
  --eoa 10 \
  --duration 120 \
  --reuse_accounts \
  --gateway_url https://dev.liberdus.com:3030
```

**Benefits of `--reuse_accounts`:**
- Skips 30-second account registration wait time
- Reuses accounts from `./artifacts/registered_accounts.json`
- Automatically registers additional accounts if needed
- Significantly faster test startup for repeated runs

### High-Frequency Load Testing

Stress test with high transaction volume:

```bash
./target/debug/load-orchestra sustain_load \
  --tx_type message \
  --tps 20 \
  --eoa 50 \
  --eoa_tps 10 \
  --duration 300 \
  --reuse_accounts \
  --verbose \
  --gateway_url https://dev.liberdus.com:3030
```

### Quick Development Test

Fast test for development with minimal setup:

```bash
./target/debug/load-orchestra sustain_load \
  --tx_type message \
  --tps 1 \
  --eoa 2 \
  --duration 5 \
  --reuse_accounts \
  --gateway_url https://dev.liberdus.com:3030
```

## Configuration

### Environment Variables

**Required**: Create a `.env` file in the project root with the network ID:

```env
# Network ID for transactions (automatically included in all transactions)
NETWORK_ID=liberdus-test
```

> ⚠️ **Important**: The `NETWORK_ID` in the `.env` file is **required** for all transactions. Without it, transactions will use the default fallback value `liberdus-default`.

You can copy the example file:
```bash
cp .env.example .env
```

### Account Storage

Registered accounts are automatically saved to `./artifacts/registered_accounts.json` and include:
- Private keys (hex encoded)
- Public addresses
- Registration aliases
- Registration timestamps

## Parameters

| Parameter | Description | Default | Example |
|-----------|-------------|---------|---------|
| `--tx_type` | Transaction type | Required | `message` |
| `--tps` | Transactions per second | 1 | `5` |
| `--eoa` | Number of accounts to use | Auto-calculated | `10` |
| `--eoa_tps` | Account registration TPS | 4 | `8` |
| `--duration` | Test duration in seconds | 60 | `300` |
| `--gateway_url` | Liberdus gateway URL | Required | `https://dev.liberdus.com:3030` |
| `--verbose` | Enable detailed logging | false | - |
| `--reuse_accounts` | Reuse existing accounts | false | - |

## Transaction Requirements

Message transactions require:
- **Network ID**: Must be set in `.env` file (see Configuration section above)
- **Minimum amount**: 25,000,000,000,000,000,000 wei (25 ETH equivalent)
- **Account balance**: Sufficient funds for transaction fees
- **Gateway connection**: Valid Liberdus gateway URL

> 🔥 **Critical**: All transactions will fail without a proper `NETWORK_ID` in your `.env` file!
