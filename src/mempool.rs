use crate::transaction::Transaction;

pub const DEFAULT_MAX_MEMPOOL_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct Mempool {
    transactions: Vec<Transaction>,
    max_size: usize,
}

impl Mempool {
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: Vec::new(),
            max_size,
        }
    }

    pub fn add(&mut self, tx: Transaction) -> Result<(), String> {
        if self.transactions.len() >= self.max_size {
            return Err("mempool is full".to_string());
        }

        if self.transactions.iter().any(|pending| pending.id == tx.id) {
            return Err("transaction already exists in mempool".to_string());
        }

        self.transactions.push(tx);
        Ok(())
    }

    pub fn drain_batch(&mut self, size: usize) -> Vec<Transaction> {
        let batch_size = size.min(self.transactions.len());
        self.transactions.drain(0..batch_size).collect()
    }

    pub fn prepend_batch(&mut self, batch: Vec<Transaction>) {
        for tx in batch.into_iter().rev() {
            self.transactions.insert(0, tx);
        }
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn pending(&self) -> &[Transaction] {
        &self.transactions
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_MEMPOOL_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(command: &str) -> Transaction {
        Transaction::from_command(command).unwrap()
    }

    #[test]
    fn adds_and_drains_fifo_batch() {
        let mut mempool = Mempool::new(10);
        let first = tx("TX alice bob 10");
        let second = tx("TX bob carol 5");

        mempool.add(first.clone()).unwrap();
        mempool.add(second.clone()).unwrap();

        let batch = mempool.drain_batch(1);

        assert_eq!(batch, vec![first]);
        assert_eq!(mempool.pending(), &[second]);
    }

    #[test]
    fn rejects_duplicates() {
        let mut mempool = Mempool::new(10);
        let tx = tx("TX alice bob 10");

        mempool.add(tx.clone()).unwrap();

        assert!(mempool.add(tx).is_err());
    }
}
