use mini_node_local::chain::Blockchain;
use mini_node_local::command::{process_command, start_server};
use std::env;
use std::io::{self, Read};
use std::sync::{Arc, Mutex};

const DEFAULT_ADDR: &str = "127.0.0.1:8765";
const DEFAULT_LOG_PATH: &str = "blocks.log";

fn main() {
    if let Err(error) = run() {
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let mut addr = env::var("MINI_NODE_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let mut log_path = env::var("MINI_NODE_LOG").unwrap_or_else(|_| DEFAULT_LOG_PATH.to_string());

    consume_option(&mut args, "--addr", &mut addr)?;
    consume_option(&mut args, "--log", &mut log_path)?;

    if args.first().map(String::as_str) == Some("--help") {
        print_usage();
        return Ok(());
    }

    if args.first().map(String::as_str) == Some("--once") {
        args.remove(0);
        let commands = if args.is_empty() {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer.lines().map(str::to_string).collect::<Vec<_>>()
        } else {
            vec![args.join(" ")]
        };

        let mut blockchain = Blockchain::load_or_create(log_path)?;
        for command in commands {
            if command.trim().is_empty() {
                continue;
            }

            print!("{}", process_command(&command, &mut blockchain));
        }
        return Ok(());
    }

    let blockchain = Blockchain::load_or_create(log_path)?;
    start_server(&addr, Arc::new(Mutex::new(blockchain)))?;

    Ok(())
}

fn consume_option(args: &mut Vec<String>, name: &str, target: &mut String) -> Result<(), String> {
    if let Some(index) = args.iter().position(|arg| arg == name) {
        if index + 1 >= args.len() {
            return Err(format!("{name} requires a value"));
        }

        *target = args.remove(index + 1);
        args.remove(index);
    }

    Ok(())
}

fn print_usage() {
    println!(
        "mini-node-local\n\n\
Usage:\n\
  cargo run -- [--addr 127.0.0.1:8765] [--log blocks.log]\n\
  cargo run -- --once <COMMAND>\n\
  echo \"CHECK\" | cargo run -- --once\n\n\
Commands:\n\
  TX <from> <to> <amount>\n\
  MINE\n\
  CHECK\n\
  LIST\n\
  HELP\n"
    );
}
