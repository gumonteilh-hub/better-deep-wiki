use better_deep_wiki;
use clap::{Parser, arg, command};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = ".")]
    path: String,
}
fn main() {
    let args = Args::parse();
    println!("Chemin à traiter : {}", args.path);
    better_deep_wiki::scan_repo(args.path);
}
