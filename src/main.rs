mod parser;
mod tokenizer;
mod counter;
mod display;
mod server;

use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, clap::ValueEnum)]
enum Speaker {
    User,
    Assistant,
    Both,
}

#[derive(Parser)]
#[command(name = "lexis", about = "Word frequency counter for Claude conversation exports")]
struct Args {
    /// Path to the Claude JSON export file (omit to launch web UI)
    file: Option<PathBuf>,

    /// Normalize tokens to lowercase (disable with --no-normalize)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    normalize: bool,

    /// Filter common English stop words (disable with --no-stop-words)
    #[arg(long, default_value_t = false, action = clap::ArgAction::Set)]
    stop_words: bool,

    /// Which speaker's messages to count
    #[arg(long, value_enum, default_value = "user")]
    speaker: Speaker,

    /// Max rows to display (0 = unlimited)
    #[arg(long, default_value_t = 100)]
    limit: usize,

    /// Launch web UI server
    #[arg(long)]
    serve: bool,

    /// Port for web UI server
    #[arg(long, default_value_t = 3210)]
    port: u16,
}

fn main() {
    let args = Args::parse();

    // If --serve or no file given, start the web server
    if args.serve || args.file.is_none() {
        let port = args.port;
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server::run(port));
        return;
    }

    let file = args.file.unwrap();
    let messages = match parser::parse_export(&file) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing {}: {e}", file.display());
            std::process::exit(1);
        }
    };

    let filtered: Vec<&parser::Message> = messages
        .iter()
        .filter(|m| match args.speaker {
            Speaker::User => m.role == "human",
            Speaker::Assistant => m.role == "assistant",
            Speaker::Both => true,
        })
        .collect();

    let mut all_tokens = Vec::new();
    for msg in &filtered {
        let tokens = tokenizer::tokenize(&msg.content, args.normalize);
        all_tokens.extend(tokens);
    }

    let counts = counter::count_words(all_tokens, args.stop_words);
    display::render_table(&counts, args.limit);
}
