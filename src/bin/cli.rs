#![cfg(feature = "cli")]

use clap::{Parser, Subcommand, arg, command};

#[derive(Parser)]
#[command(name = "better-deepwiki")]
#[command(about = "Augmented codebase search & doc tool (CLI)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Embed {
        #[arg(long)]
        repo_path: String,
    },
    Query {
        #[arg(long)]
        question: String,
        #[arg(long)]
        instructions: String,
        #[arg(long)]
        repo_path: String,
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Embed { repo_path } => {
            better_deep_wiki::scan_repo(repo_path, None).await;
        }

        Commands::Query {
            question,
            instructions,
            repo_path,
        } => {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(16);
            
            tokio::spawn(async move {
                if let Err(err) = better_deep_wiki::ask_repo(question, instructions, repo_path, tx).await {
                    eprintln!("❌ {err}");
                }
            });
            
            while let Some(chunk) = rx.recv().await {
                print!("{}", chunk);
            }
        },
    }
}
