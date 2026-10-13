use clap::Parser;

fn main() {
    let cli = rewire::cli::Cli::parse();
    if let Err(e) = rewire::cli::run(cli) {
        eprintln!("rewire: {e}");
        std::process::exit(1);
    }
}
