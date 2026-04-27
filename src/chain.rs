use crate::block::{short_hash, Block, GENESIS_PREVIOUS_HASH};
use crate::mempool::{Mempool, DEFAULT_MAX_MEMPOOL_SIZE};
use crate::storage::AppendOnlyLog;
use crate::transaction::Transaction;
use std::io;
use std::path::PathBuf;

pub const DEFAULT_BLOCK_SIZE: usize = 5;

#[derive(Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
    mempool: Mempool,
    log: AppendOnlyLog,
    block_size: usize,
}

impl Blockchain {
    pub fn load_or_create(log_path: impl Into<PathBuf>) -> io::Result<Self> {
        let log = AppendOnlyLog::new(log_path);
        let mut blocks = Vec::new();

        for line in log.read_lines()? {
            if line.trim().is_empty() {
                continue;
            }

            let block = Block::from_log_line(&line)
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;
            blocks.push(block);
        }

        if blocks.is_empty() {
            let genesis = Block::genesis();
            log.append_line(&genesis.to_log_line())?;
            blocks.push(genesis);
        }

        validate_blocks(&blocks)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

        Ok(Self {
            blocks,
            mempool: Mempool::new(DEFAULT_MAX_MEMPOOL_SIZE),
            log,
            block_size: DEFAULT_BLOCK_SIZE,
        })
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        self.mempool.add(tx)
    }

    pub fn mine_block(&mut self) -> Result<Block, String> {
        if self.mempool.is_empty() {
            return Err("mempool is empty".to_string());
        }

        let previous = self
            .blocks
            .last()
            .ok_or_else(|| "blockchain has no genesis block".to_string())?;
        let batch = self.mempool.drain_batch(self.block_size);
        let block = Block::new_now(previous.index + 1, previous.hash.clone(), batch.clone());

        if let Err(error) = self.log.append_line(&block.to_log_line()) {
            self.mempool.prepend_batch(batch);
            return Err(format!("failed to persist block: {error}"));
        }

        self.blocks.push(block.clone());
        Ok(block)
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_blocks(&self.blocks)
    }

    pub fn validation_report(&self) -> String {
        match self.validate() {
            Ok(()) => format!("OK: Chain is valid ({} blocks)\n", self.blocks.len()),
            Err(error) => format!("ERROR: {error}\n"),
        }
    }

    pub fn list_blocks(&self) -> String {
        let mut output = String::new();

        for block in &self.blocks {
            output.push_str(&format!(
                "Block #{} | hash={} | prev={} | txs={}\n",
                block.index,
                short_hash(&block.hash),
                short_hash(&block.previous_hash),
                block.transactions.len()
            ));
        }

        output.push_str("---\n");
        output.push_str(&format!(
            "Mempool: {} pending transactions\n",
            self.mempool.len()
        ));

        output
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn mempool(&self) -> &Mempool {
        &self.mempool
    }
}

pub fn validate_blocks(blocks: &[Block]) -> Result<(), String> {
    if blocks.is_empty() {
        return Err("chain has no genesis block".to_string());
    }

    let genesis = &blocks[0];
    if genesis.index != 0 {
        return Err("genesis block index must be 0".to_string());
    }

    if genesis.previous_hash != GENESIS_PREVIOUS_HASH {
        return Err("genesis previous_hash is invalid".to_string());
    }

    validate_block_hash(genesis, 0)?;

    for (position, pair) in blocks.windows(2).enumerate() {
        let previous = &pair[0];
        let current = &pair[1];
        let expected_index = (position + 1) as u64;

        if current.index != expected_index {
            return Err(format!(
                "chain broken at block {}: expected index {}, found {}",
                position + 1,
                expected_index,
                current.index
            ));
        }

        if current.previous_hash != previous.hash {
            return Err(format!(
                "chain broken at block {}: previous_hash mismatch",
                current.index
            ));
        }

        validate_block_hash(current, current.index)?;
    }

    Ok(())
}

fn validate_block_hash(block: &Block, index_for_message: u64) -> Result<(), String> {
    for tx in &block.transactions {
        if tx.id != tx.calculate_hash() {
            return Err(format!(
                "chain broken at block {index_for_message}: transaction hash mismatch"
            ));
        }
    }

    let expected_hash = block.calculate_hash();
    if block.hash != expected_hash {
        return Err(format!(
            "chain broken at block {index_for_message}: hash mismatch"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::process_command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_log_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("mini-node-chain-{name}-{nanos}.log"))
    }

    #[test]
    fn creates_genesis_when_log_is_missing() {
        let path = temp_log_path("genesis");
        let chain = Blockchain::load_or_create(&path).unwrap();

        assert_eq!(chain.blocks().len(), 1);
        assert!(chain.validate().is_ok());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn mines_persists_and_reloads_block() {
        let path = temp_log_path("persist");
        let mut chain = Blockchain::load_or_create(&path).unwrap();

        process_command("TX alice bob 10", &mut chain);
        process_command("MINE", &mut chain);

        assert_eq!(chain.blocks().len(), 2);
        assert!(chain.validate().is_ok());

        let reloaded = Blockchain::load_or_create(&path).unwrap();

        assert_eq!(reloaded.blocks().len(), 2);
        assert!(reloaded.validate().is_ok());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn detects_broken_previous_hash() {
        let mut genesis = Block::genesis();
        let mut block = Block::new(1, genesis.hash.clone(), vec![], 1);
        block.previous_hash = "bad".to_string();
        genesis.hash = genesis.calculate_hash();

        let result = validate_blocks(&[genesis, block]);

        assert!(result.unwrap_err().contains("previous_hash mismatch"));
    }
}
