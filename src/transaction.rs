use crate::hash::sha256_hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub id: String,
}

impl Transaction {
    pub fn new(
        sender: impl Into<String>,
        receiver: impl Into<String>,
        amount: u64,
    ) -> Result<Self, String> {
        let sender = sender.into();
        let receiver = receiver.into();

        validate_party("sender", &sender)?;
        validate_party("receiver", &receiver)?;

        if amount == 0 {
            return Err("amount must be greater than zero".to_string());
        }

        let id = Self::calculate_hash_parts(&sender, &receiver, amount);

        Ok(Self {
            sender,
            receiver,
            amount,
            id,
        })
    }

    pub fn from_command(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.len() != 4 || !parts[0].eq_ignore_ascii_case("TX") {
            return Err("invalid transaction format: expected TX <from> <to> <amount>".to_string());
        }

        let amount = parts[3]
            .parse::<u64>()
            .map_err(|_| "amount must be an unsigned integer".to_string())?;

        Self::new(parts[1], parts[2], amount)
    }

    pub fn from_log_fragment(input: &str) -> Result<Self, String> {
        Self::from_command(input)
    }

    pub fn to_log_fragment(&self) -> String {
        format!("TX {} {} {}", self.sender, self.receiver, self.amount)
    }

    pub fn calculate_hash(&self) -> String {
        Self::calculate_hash_parts(&self.sender, &self.receiver, self.amount)
    }

    fn calculate_hash_parts(sender: &str, receiver: &str, amount: u64) -> String {
        sha256_hex(format!("tx|{sender}|{receiver}|{amount}"))
    }
}

fn validate_party(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{label} cannot be empty"));
    }

    if value.contains('|') {
        return Err(format!("{label} cannot contain pipe characters"));
    }

    if value.chars().any(char::is_whitespace) {
        return Err(format!("{label} cannot contain whitespace"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_transaction() {
        let tx = Transaction::from_command("TX alice bob 10").unwrap();

        assert_eq!(tx.sender, "alice");
        assert_eq!(tx.receiver, "bob");
        assert_eq!(tx.amount, 10);
        assert_eq!(tx.id, tx.calculate_hash());
    }

    #[test]
    fn rejects_bad_format() {
        assert!(Transaction::from_command("TX alice bob").is_err());
        assert!(Transaction::from_command("PAY alice bob 10").is_err());
        assert!(Transaction::from_command("TX alice bob nope").is_err());
    }

    #[test]
    fn transaction_hash_is_deterministic() {
        let tx1 = Transaction::from_command("TX alice bob 10").unwrap();
        let tx2 = Transaction::from_command("TX alice bob 10").unwrap();

        assert_eq!(tx1.id, tx2.id);
    }
}
