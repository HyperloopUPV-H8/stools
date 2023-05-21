use clap::Parser;
use reqwest::ClientBuilder;
use stools::commands::Commands;

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new().user_agent("stools").build().unwrap();

    let args = Args::parse();

    args.command.run(client).await;
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
/// The power of software at the palm of your hands.
///
/// Sync with a selection of the software applications easily. Currently, it can
/// sync the back end, ethernet view and the control station.
///
/// To start working run `stools sync <target>`.
///
/// For more help run `stools help <command>`.
struct Args {
    #[clap(subcommand)]
    command: Commands,
}
