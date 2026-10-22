use clap::Parser;

fn main() {
    #[cfg(unix)]
    // SAFETY: restores the default action for SIGPIPE before any other thread exists: `rewire sessions | head` ends
    // quietly instead of panicking on a closed pipe
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let cli = rewire::cli::Cli::parse();
    if let Err(e) = rewire::cli::run(cli) {
        eprintln!("rewire: {e}");
        std::process::exit(1);
    }
}
