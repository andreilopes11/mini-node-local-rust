use crate::block::short_hash;
use crate::chain::Blockchain;
use crate::transaction::Transaction;
use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn process_command(input: &str, blockchain: &mut Blockchain) -> String {
    let command = input.trim();

    if command.is_empty() {
        return "ERROR: empty command\n".to_string();
    }

    let parts: Vec<&str> = command.split_whitespace().collect();
    let name = parts[0].to_ascii_uppercase();

    match name.as_str() {
        "TX" => match Transaction::from_command(command).and_then(|tx| {
            let tx_id = tx.id.clone();
            blockchain.add_transaction(tx)?;
            Ok(tx_id)
        }) {
            Ok(tx_id) => format!("OK: TX added to mempool ({})\n", short_hash(&tx_id)),
            Err(error) => format!("ERROR: {error}\n"),
        },
        "MINE" if parts.len() == 1 => match blockchain.mine_block() {
            Ok(block) => format!(
                "OK: Block #{} created with hash {} ({} transactions)\n",
                block.index,
                short_hash(&block.hash),
                block.transactions.len()
            ),
            Err(error) => format!("ERROR: {error}\n"),
        },
        "CHECK" if parts.len() == 1 => blockchain.validation_report(),
        "LIST" if parts.len() == 1 => blockchain.list_blocks(),
        "HELP" if parts.len() == 1 => help_text(),
        "QUIT" if parts.len() == 1 => "OK: bye\n".to_string(),
        "MINE" | "CHECK" | "LIST" | "HELP" | "QUIT" => {
            format!("ERROR: {name} does not accept arguments\n")
        }
        _ => "ERROR: unknown command. Try HELP\n".to_string(),
    }
}

pub fn help_text() -> String {
    [
        "Commands:",
        "  TX <from> <to> <amount>  Add a transaction to the mempool",
        "  MINE                     Build and persist one block",
        "  CHECK                    Validate chained hashes",
        "  LIST                     Show blocks and mempool size",
        "  HELP                     Show this help",
        "  QUIT                     Close a TCP client connection",
        "",
    ]
    .join("\n")
}

pub fn start_server(addr: &str, blockchain: Arc<Mutex<Blockchain>>) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("mini-node-local listening on {addr}");
    println!("Try: echo \"TX alice bob 10\" | nc 127.0.0.1 8765");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let blockchain = Arc::clone(&blockchain);
                thread::spawn(move || {
                    if let Err(error) = handle_client(stream, blockchain) {
                        eprintln!("client error: {error}");
                    }
                });
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }

    Ok(())
}

fn handle_client(stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>) -> io::Result<()> {
    let reader_stream = stream.try_clone()?;
    let mut reader = io::BufReader::new(reader_stream);
    let mut writer = io::BufWriter::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;

        if bytes == 0 {
            break;
        }

        let should_quit = line.trim().eq_ignore_ascii_case("QUIT");
        let response = {
            let mut blockchain = blockchain
                .lock()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "blockchain lock poisoned"))?;
            process_command(&line, &mut blockchain)
        };

        writer.write_all(response.as_bytes())?;
        writer.flush()?;

        if should_quit {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::Blockchain;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_log_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("mini-node-command-{name}-{nanos}.log"))
    }

    #[test]
    fn full_command_flow() {
        let path = temp_log_path("flow");
        let mut chain = Blockchain::load_or_create(&path).unwrap();

        assert!(process_command("TX alice bob 10", &mut chain).starts_with("OK"));
        assert!(process_command("MINE", &mut chain).starts_with("OK"));
        assert!(process_command("CHECK", &mut chain).contains("Chain is valid"));

        let list = process_command("LIST", &mut chain);
        assert!(list.contains("Block #0"));
        assert!(list.contains("Block #1"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn reports_unknown_command() {
        let path = temp_log_path("unknown");
        let mut chain = Blockchain::load_or_create(&path).unwrap();

        assert!(process_command("NOPE", &mut chain).starts_with("ERROR"));

        let _ = std::fs::remove_file(path);
    }
}
