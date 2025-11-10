//! Reflections is a command-line tool for Solidity analysis
use reflections_commands::{Args, commands::Parser as _, run};
use yansi::{Condition, Paint as _};

const HAVE_COLOR: Condition = Condition(|| {
    std::env::var_os("NO_COLOR").is_none()
        && (Condition::CLICOLOR_LIVE)()
        && Condition::stdouterr_are_tty_live()
});

#[tokio::main]
async fn main() {
    // disable colors if unsupported
    yansi::whenever(HAVE_COLOR);
    let args = Args::parse();
    if !args.verbose.is_present() {
        banner();
    }
    if let Err(err) = run(args.command, args.verbose).await {
        eprintln!("{}", err.to_string().red())
    }
}

/// Generate and print a banner
fn banner() {
    println!(
        "{}",
        format!(
            "
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    ╦═╗╔═╗╔═╗╦  ╔═╗╔═╗╔╦╗╦╔═╗╔╗╔╔═╗    Solidity Reflections
    ╠╦╝║╣ ╠╣ ║  ║╣ ║   ║ ║║ ║║║║╚═╗    API Tool
    ╩╚═╚═╝╚  ╩═╝╚═╝╚═╝ ╩ ╩╚═╝╝╚╝╚═╝
           v{}        github.com/GoldenSylph/solidity-reflections
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
",
            env!("CARGO_PKG_VERSION")
        )
        .bright_cyan()
    );
}
