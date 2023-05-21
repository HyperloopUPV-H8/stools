use std::{error, fmt::Display, io, path::PathBuf};

use clap::{Args, ValueEnum};
use reqwest::Client;
use zip::result::ZipError;

use super::{download, mount, Target};

#[derive(Debug, Clone, ValueEnum)]
pub enum FrontTarget {
    #[clap(alias = "eth")]
    Ethernet,
    #[clap(alias = "ctrl")]
    Control,
}

impl Into<Target> for FrontTarget {
    fn into(self) -> Target {
        match self {
            FrontTarget::Control => Target::Control,
            FrontTarget::Ethernet => Target::Ethernet,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct Params {
    target: FrontTarget,

    #[clap(long = "backend")]
    /// Optional version tag for the backend.
    backend_tag: Option<String>,

    #[clap(long = "frontend")]
    /// Optional version tag for the target frontend.
    frontend_tag: Option<String>,

    #[clap(short, default_value = "./stools")]
    /// Output path for the downloaded files.
    output: PathBuf,
}

pub async fn run(params: &Params, client: &Client) -> Result<(), Error> {
    download::run(
        &download::Params {
            target: Target::Backend,
            tag: params.backend_tag.clone(),
            output: params.output.clone(),
        },
        client,
    )
    .await?;

    download::run(
        &download::Params {
            target: params.target.clone().into(),
            tag: params.frontend_tag.clone(),
            output: params.output.clone(),
        },
        client,
    )
    .await?;

    mount::run(&mount::Params {
        target: Target::Backend,
        path: params.output.clone(),
    })?;

    mount::run(&mount::Params {
        target: params.target.clone().into(),
        path: params.output.clone(),
    })?;

    return Ok(());
}

#[derive(Debug)]
pub enum Error {
    Parse(serde_json::Error),
    Request(reqwest::Error),
    File(io::Error),
    TagNotFound(String),
    Zip(ZipError),
}

impl From<download::Error> for Error {
    fn from(err: download::Error) -> Self {
        match err {
            download::Error::Parse(err) => Error::Parse(err),
            download::Error::Request(err) => Error::Request(err),
            download::Error::File(err) => Error::File(err),
            download::Error::TagNotFound(tag) => Error::TagNotFound(tag),
        }
    }
}

impl From<mount::Error> for Error {
    fn from(err: mount::Error) -> Self {
        match err {
            mount::Error::File(err) => Error::File(err),
            mount::Error::Zip(err) => Error::Zip(err),
        }
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(err) => write!(f, "parse error: {}", err),
            Error::Request(err) => write!(f, "request error: {}", err),
            Error::File(err) => write!(f, "file error: {}", err),
            Error::TagNotFound(tag) => write!(f, "tag {} not found", tag),
            Error::Zip(err) => write!(f, "zip error: {}", err),
        }
    }
}

pub fn view(result: Result<(), Error>) {
    match result {
        Ok(_) => println!("App successfully synced!"),
        Err(err) => eprintln!("Error while syncing: {}", err),
    }
}
