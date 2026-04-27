use crate::hash::sha256_hex;
use crate::transaction::Transaction;
use std::time::{SystemTime, UNIX_EPOCH};

pub const GENESIS_PREVIOUS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn genesis() -> Self {
        Self::new(0, GENESIS_PREVIOUS_HASH.to_string(), Vec::new(), 0)
    }

    pub fn new(
        index: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        timestamp: u64,
    ) -> Self {
        let hash = Self::calculate_hash_fields(&previous_hash, index, timestamp, &transactions);

        Self {
            index,
            previous_hash,
            hash,
            timestamp,
            transactions,
        }
    }

    pub fn new_now(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        Self::new(index, previous_hash, transactions, now_seconds())
    }

    pub fn calculate_hash(&self) -> String {
        Self::calculate_hash_fields(
            &self.previous_hash,
            self.index,
            self.timestamp,
            &self.transactions,
        )
    }

    pub fn calculate_hash_fields(
        previous_hash: &str,
        index: u64,
        timestamp: u64,
        transactions: &[Transaction],
    ) -> String {
        let tx_hashes = transactions
            .iter()
            .map(|tx| tx.id.as_str())
            .collect::<Vec<_>>()
            .join("");
        let content = format!("block|{previous_hash}|{index}|{timestamp}|{tx_hashes}");

        sha256_hex(content)
    }

    pub fn to_log_line(&self) -> String {
        let mut parts = vec![
            "BLOCK".to_string(),
            self.index.to_string(),
            format!("prev={}", self.previous_hash),
            format!("hash={}", self.hash),
            format!("ts={}", self.timestamp),
            format!("txs={}", self.transactions.len()),
        ];

        parts.extend(self.transactions.iter().map(Transaction::to_log_fragment));
        parts.join("|")
    }

    pub fn from_log_line(line: &str) -> Result<Self, String> {
        let parts: Vec<&str> = line.trim().split('|').collect();

        if parts.len() < 6 || parts[0] != "BLOCK" {
            return Err("invalid block log line".to_string());
        }

        let index = parts[1]
            .parse::<u64>()
            .map_err(|_| "invalid block index".to_string())?;
        let previous_hash = parts[2]
            .strip_prefix("prev=")
            .ok_or_else(|| "missing prev= field".to_string())?
            .to_string();
        let hash = parts[3]
            .strip_prefix("hash=")
            .ok_or_else(|| "missing hash= field".to_string())?
            .to_string();
        let timestamp = parts[4]
            .strip_prefix("ts=")
            .ok_or_else(|| "missing ts= field".to_string())?
            .parse::<u64>()
            .map_err(|_| "invalid block timestamp".to_string())?;
        let tx_count = parts[5]
            .strip_prefix("txs=")
            .ok_or_else(|| "missing txs= field".to_string())?
            .parse::<usize>()
            .map_err(|_| "invalid tx count".to_string())?;

        let transactions = parts
            .iter()
            .skip(6)
            .map(|fragment| Transaction::from_log_fragment(fragment))
            .collect::<Result<Vec<_>, _>>()?;

        if transactions.len() != tx_count {
            return Err(format!(
                "tx count mismatch: header says {tx_count}, found {}",
                transactions.len()
            ));
        }

        Ok(Self {
            index,
            previous_hash,
            hash,
            timestamp,
            transactions,
        })
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before Unix epoch")
        .as_secs()
}

pub fn short_hash(hash: &str) -> &str {
    hash.get(..12).unwrap_or(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(command: &str) -> Transaction {
        Transaction::from_command(command).unwrap()
    }

    #[test]
    fn genesis_is_deterministic() {
        assert_eq!(Block::genesis(), Block::genesis());
    }

    #[test]
    fn block_round_trips_through_log_line() {
        let block = Block::new(
            1,
            Block::genesis().hash,
            vec![tx("TX alice bob 10"), tx("TX bob carol 5")],
            123,
        );

        let parsed = Block::from_log_line(&block.to_log_line()).unwrap();

        assert_eq!(parsed, block);
        assert_eq!(parsed.hash, parsed.calculate_hash());
    }

    #[test]
    fn changing_transaction_changes_hash() {
        let previous_hash = Block::genesis().hash;
        let block_a = Block::new(1, previous_hash.clone(), vec![tx("TX alice bob 10")], 123);
        let block_b = Block::new(1, previous_hash, vec![tx("TX alice bob 11")], 123);

        assert_ne!(block_a.hash, block_b.hash);
    }
}
