# Mini-Node Exercises

These exercises are now implemented in the Rust codebase. Use this file as a study guide: read the source file, run its tests, then change or extend it.

Run all tests:

```bash
cargo test
```

Run a demo:

```bash
printf "TX alice bob 10\nTX bob carol 5\nMINE\nCHECK\nLIST\n" | cargo run -- --once
```

## Exercise 1: Simple Hash Table

Source: `src/hash_table.rs`

Goal:

- Understand average-case key-value lookup.
- Practice collision handling.

Implemented behavior:

- `HashTable::new(capacity)`
- `insert(key, value)`
- `get(&key)`
- `remove(&key)`
- Collision handling through chaining.

Why it matters:

- Nodes often need fast lookup by transaction ID or block hash.
- Smart contract systems care about access cost because repeated lookup can become expensive.

Try next:

- Add resizing when the load factor gets too high.
- Track collision counts.
- Compare with `std::collections::HashMap`.

## Exercise 2: Append-Only Log

Source: `src/storage.rs`

Goal:

- Persist history without rewriting past entries.

Implemented behavior:

- `AppendOnlyLog::append_line`
- `AppendOnlyLog::read_lines`
- Missing log files read as empty.

Log example:

```text
BLOCK|0|prev=<zero_hash>|hash=<genesis_hash>|ts=0|txs=0
BLOCK|1|prev=<genesis_hash>|hash=<block_hash>|ts=<unix_seconds>|txs=1|TX alice bob 10
```

Why it matters:

- Append-only storage is simple to audit.
- Recovery can happen by replaying the log.
- Old history is not rewritten during normal operation.

Try next:

- Add checksums per line.
- Add log compaction into snapshots.
- Add file locking for multiple processes.

## Exercise 3: Simple TCP Server

Source: `src/command.rs`

Goal:

- Receive external commands over a network boundary.

Run:

```bash
cargo run --
```

Send commands:

```bash
echo "TX alice bob 10" | nc 127.0.0.1 8765
echo "MINE" | nc 127.0.0.1 8765
echo "CHECK" | nc 127.0.0.1 8765
```

Implemented commands:

```text
TX <from> <to> <amount>
MINE
CHECK
LIST
HELP
QUIT
```

Why it matters:

- A node is a service, not just a data structure.
- Protocol design determines what outside clients can ask the node to do.

Try next:

- Add a `MEMPOOL` command.
- Add request IDs.
- Add JSON output as an optional mode.

## Exercise 4: Text Transaction Parsing

Source: `src/transaction.rs`

Goal:

- Convert raw text into trusted internal data.

Input format:

```text
TX alice bob 10
```

Implemented validation:

- Command must be `TX`.
- There must be exactly three arguments after `TX`.
- Amount must be an unsigned integer greater than zero.
- Sender and receiver cannot contain whitespace or `|`.

Transaction ID:

```text
SHA256("tx|<sender>|<receiver>|<amount>")
```

Try next:

- Add transaction nonces.
- Add signatures.
- Add account balance checks.

## Exercise 5: In-Memory Mempool

Source: `src/mempool.rs`

Goal:

- Keep pending transactions fast and temporary.

Implemented behavior:

- FIFO transaction storage.
- Duplicate transaction rejection by ID.
- `drain_batch(size)` for block creation.
- Default maximum size of 1024 transactions.

Why it matters:

- Real mempools are policy-heavy and affect fee markets.
- This project keeps only the core idea: pending transactions wait before final persistence.

Try next:

- Add fee-based ordering.
- Add expiration for old transactions.
- Add a command that lists pending transactions.

## Exercise 6: Block Assembly

Source: `src/block.rs` and `src/chain.rs`

Goal:

- Group transactions into blocks and link them to history.

Implemented behavior:

- `MINE` drains up to 5 transactions from the mempool.
- The new block points to the previous block hash.
- The block hash includes previous hash, index, timestamp, and transaction IDs.

Block hash:

```text
SHA256("block|<previous_hash>|<index>|<timestamp>|<concatenated_tx_ids>")
```

Try next:

- Make block size configurable from CLI.
- Add a Merkle root instead of concatenated transaction IDs.
- Add proof-of-work leading-zero difficulty.

## Exercise 7: Block Persistence

Source: `src/block.rs`, `src/storage.rs`, and `src/chain.rs`

Goal:

- Save blocks to disk and reload them.

Implemented behavior:

- Genesis block is created if the log is missing or empty.
- Every mined block is appended to `blocks.log`.
- Startup replays the log and validates it before accepting commands.

Why it matters:

- Durability separates a toy in-memory chain from a reproducible node.
- Replay is the simplest recovery model.

Try next:

- Add snapshots for faster startup.
- Add a block index file.
- Add corruption tests by editing `blocks.log` manually.

## Exercise 8: CHECK Command

Source: `src/chain.rs`

Goal:

- Validate chain integrity.

Implemented checks:

- Genesis block index and previous hash.
- Transaction IDs.
- Block hashes.
- Sequential indexes.
- Previous-hash links.

Run:

```bash
cargo run -- --once CHECK
```

Expected success:

```text
OK: Chain is valid (N blocks)
```

Expected failure after manual tampering:

```text
ERROR: chain broken at block 1: hash mismatch
```

Try next:

- Return a multi-line validation report.
- Validate timestamps are monotonic.
- Validate no duplicate transactions across the whole chain.

## Portfolio README Checklist

A strong README for this project should explain:

- Project objective
- How to build and run
- Command protocol
- Block structure
- How transaction IDs are calculated
- How block hashes are calculated
- How `CHECK` works
- Limitations of the project
- Relationship to a real blockchain node

The root `README.md` now covers these points.
