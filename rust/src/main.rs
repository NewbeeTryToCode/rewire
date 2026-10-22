use clap::Parser;

fn main() {
    // `rewire sessions | head`: a closed stdout ends the command quietly. SIGPIPE stays ignored (Rust's default), since
    // a child that exits before reading its stdin must fail as an error, not kill rewire.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let text = info.payload().downcast_ref::<String>().map(String::as_str).or_else(|| info.payload().downcast_ref::<&str>().copied());
        if text.is_some_and(|t| t.contains("failed printing to stdout") && t.contains("Broken pipe")) {
            std::process::exit(0);
        }
        default_hook(info);
    }));
    let cli = rewire::cli::Cli::parse();
    if let Err(e) = rewire::cli::run(cli) {
        eprintln!("rewire: {e}");
        std::process::exit(1);
    }
}
