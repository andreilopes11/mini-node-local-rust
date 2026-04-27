# Mini-Node Local Rust

A minimal local blockchain node written in Rust for applied computer science practice.

The project implements the core mechanics behind a blockchain node in a small, reproducible codebase:

- Text transaction protocol
- In-memory mempool
- Block assembly
- Chained SHA-256 block hashes
- Append-only persistence in `blocks.log`
- `CHECK` validation for chain integrity
- Simple TCP server on `127.0.0.1:8765`

This is an educational system, not a production blockchain.

## Why This Project Exists

The goal is to connect undergraduate computer science foundations to the kinds of engineering tradeoffs that blockchains and smart contracts expose:

- Cost: hashing, validation, and algorithmic complexity matter.
- Data structures: hash tables and queues shape node behavior.
- Networking: nodes receive external messages through protocols.
- Persistence: durable systems need replayable storage.
- Consensus thinking: a single node can validate itself, but distributed systems must agree under latency and failure.

## Current Implementation

This repository now contains a working Rust implementation with no external crate dependencies.

```text
mini-node-local-rust/
|-- Cargo.toml
|-- README.md
|-- blocks.log              # generated at runtime
|-- docs/
|   |-- DESIGN.md
|   |-- EXERCISES.md
|   `-- ROLES.md
`-- src/
    |-- main.rs             # CLI and server entry point
    |-- lib.rs              # module exports
    |-- command.rs          # command parser and TCP server
    |-- chain.rs            # blockchain state and validation
    |-- block.rs            # block structure and hashing
    |-- transaction.rs      # transaction parsing and IDs
    |-- mempool.rs          # pending transaction queue
    |-- storage.rs          # append-only log wrapper
    |-- hash.rs             # dependency-free SHA-256
    `-- hash_table.rs       # exercise implementation
```

## Quick Start

Build and test:

```bash
cargo test
cargo build --release
```

Run one command:

```bash
cargo run -- --once HELP
cargo run -- --once CHECK
```

Run a full scripted flow in one process:

```bash
printf "TX alice bob 10\nTX bob carol 5\nMINE\nCHECK\nLIST\n" | cargo run -- --once
```

Start the TCP server:

```bash
cargo run --
```

Send commands from another terminal:

```bash
echo "TX alice bob 10" | nc 127.0.0.1 8765
echo "MINE" | nc 127.0.0.1 8765
echo "CHECK" | nc 127.0.0.1 8765
echo "LIST" | nc 127.0.0.1 8765
```

Use a custom log file:

```bash
cargo run -- --log /tmp/mini-node-demo.log --once CHECK
printf "TX alice bob 10\nMINE\nLIST\n" | cargo run -- --log /tmp/mini-node-demo.log --once
```

## Command Protocol

```text
TX <from> <to> <amount>  Add a transaction to the mempool
MINE                     Build and persist one block
CHECK                    Validate the hash chain
LIST                     Show blocks and mempool size
HELP                     Show command help
QUIT                     Close a TCP client connection
```

Example responses:

```text
OK: TX added to mempool (2d94e5d9a0c1)
OK: Block #1 created with hash 3b47f0c7ab12 (2 transactions)
OK: Chain is valid (2 blocks)
```

## Persistence Format

Blocks are written to `blocks.log` as an append-only text log.

```text
BLOCK|0|prev=0000000000000000000000000000000000000000000000000000000000000000|hash=<hash>|ts=0|txs=0
BLOCK|1|prev=<genesis_hash>|hash=<block_hash>|ts=<unix_seconds>|txs=2|TX alice bob 10|TX bob carol 5
```

The node rebuilds the chain by replaying this file on startup. The mempool is intentionally in-memory and is not persisted.

## Hashing

Transaction ID:

```text
SHA256("tx|<sender>|<receiver>|<amount>")
```

Block hash:

```text
SHA256("block|<previous_hash>|<index>|<timestamp>|<concatenated_tx_ids>")
```

The genesis block has index `0`, timestamp `0`, no transactions, and a previous hash of 64 zeroes.

## CHECK Validation

`CHECK` verifies:

- The genesis block has index `0` and the expected previous hash.
- Every transaction ID matches its recalculated SHA-256 hash.
- Every block hash matches its recalculated SHA-256 hash.
- Every block points to the previous block hash.
- Block indexes are sequential.

If any value is changed manually in `blocks.log`, `CHECK` should report where the chain breaks.

## Learning Resources

- MIT 6.006 Introduction to Algorithms: https://ocw.mit.edu/courses/6-006-introduction-to-algorithms-spring-2020/
- MIT 6.1810 Operating System Engineering: https://pdos.csail.mit.edu/6.1810/
- MIT Missing Semester: https://missing.csail.mit.edu/
- Beej's Guide to Network Programming: https://beej.us/guide/bgnet/

Recommended focus:

- MIT 6.006 lectures 1, 2, 3, hashing, graphs, and complexity
- MIT 6.1810 lectures and xv6 labs as reference, without trying to complete the whole course

## What This Teaches

After finishing and explaining this project, you should be able to describe:

- Why a blockchain is a distributed system, not just a database
- How consensus, finality, latency, and failures relate
- Why computational cost matters in smart contracts
- How append-only storage supports replay and auditability
- How chained hashes make tampering detectable

## Limitations

This project intentionally does not implement:

- Real peer-to-peer networking
- Proof of work or proof of stake
- Cryptographic signatures
- Account balances or smart contract execution
- Fees, gas, or transaction prioritization
- Distributed consensus

Those features can be added later as extensions once the local node is clear.

## More Documentation

- [Design Notes](docs/DESIGN.md)
- [Exercises](docs/EXERCISES.md)
- [Project Roles](docs/ROLES.md)
