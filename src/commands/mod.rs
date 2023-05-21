use clap::{Subcommand, ValueEnum};
use reqwest::{Client, Url};

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// List all the available versions for the target.
    ///
    /// Outputs a list of version tags available to download, this tag can be used on other commands
    /// to download specific versions of the target.
    List(list::Params),

    /// Download all the target files.
    ///
    /// Downloads all the artifacts from the github release to the target directory (./stools by
    /// default), creating it if it doesn't exists. This might take a while depending on the
    /// file size.
    Download(download::Params),

    /// Prepare all the target files.
    ///
    /// After downloading all the artifacts run this command to put every file where it should go,
    /// for the frontend it means extracting the zip file, and for the backend all the files remain
    /// the same.
    Mount(mount::Params),

    /// Sync the target version.
    ///
    /// This will download both the backend and the specified frontend and mount both. This is the
    /// same as manually calling download and mount for the specific targets.
    Sync(sync::Params),
}

impl Commands {
    pub async fn run(&self, client: Client) {
        match self {
            Commands::List(params) => list::view(list::run(params, &client).await),
            Commands::Download(params) => download::view(download::run(params, &client).await),
            Commands::Mount(params) => mount::view(mount::run(params)),
            Commands::Sync(params) => sync::view(sync::run(params, &client).await),
        }
    }
}

pub mod download;
pub mod list;
pub mod mount;
pub mod sync;

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum Target {
    #[clap(alias = "eth")]
    Ethernet,
    #[clap(alias = "ctrl")]
    Control,
    #[clap(alias = "back")]
    Backend,
}

impl Target {
    fn releases_endpoint(&self) -> Url {
        Url::parse(match self {
            Target::Ethernet => "https://api.github.com/repos/HyperloopUPV-H8/ev-frontend/releases",
            Target::Control => "https://api.github.com/repos/HyperloopUPV-H8/cs-frontend/releases",
            Target::Backend => "https://api.github.com/repos/HyperloopUPV-H8/h8-backend/releases",
        })
        .unwrap()
    }
}
