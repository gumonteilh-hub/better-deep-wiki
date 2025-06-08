use better_deep_wiki;
use clap::{Parser, Subcommand, arg, command};

// #[derive(Parser, Debug)]
// #[command(author, version, about)]
// struct Args {
//     #[arg(short, long, default_value = ".")]
//     path: String,
// }
// fn main() {
//     let args = Args::parse();
//     dotenvy::dotenv().ok();
//     println!("Chemin à traiter : {}", args.path);
//     better_deep_wiki::scan_repo(args.path);
// }

#[derive(Parser)]
#[command(name = "better-deepwiki")]
#[command(about = "Augmented codebase search & doc tool", long_about = None)]
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

fn main() {
    let cli = Cli::parse();
    dotenvy::dotenv().ok();

    match &cli.command {
        Commands::Embed { repo_path } => {
            let opts = repo_path.clone();
            better_deep_wiki::scan_repo(opts);
        }
        Commands::Query {
            question,
            repo_path,
            instructions,
        } => match better_deep_wiki::ask_repo(question.clone(), instructions.clone(), repo_path.clone()) {
            Ok(r) => println!("Response : {}", r),
            Err(err) => eprintln!("Error : {err}"),
        },
    }
}
