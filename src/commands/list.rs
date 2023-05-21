use std::{
    error,
    fmt::{self, Display, Formatter},
};

use chrono::{DateTime, Utc};
use clap::Args;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::date;

use super::Target;

#[derive(Args, Debug, Clone)]
pub struct Params {
    pub(crate) target: Target,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub browser_download_url: String,
    pub name: String,
    pub content_type: String,
    pub size: u64,

    #[serde(with = "date::github")]
    pub updated_at: DateTime<Utc>,
}

pub async fn run(params: &Params, client: &Client) -> Result<Vec<Release>, Error> {
    let resposne = client
        .get(params.target.releases_endpoint())
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    let releases: Vec<Release> = resposne.json().await?;

    return Ok(releases);
}

#[derive(Debug)]
pub enum Error {
    Request(reqwest::Error),
    Parse(serde_json::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Request(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Parse(err)
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Request(err) => {
                write!(f, "Error making request: ")?;
                if let Some(status) = err.status() {
                    write!(f, "({}) ", status)?;
                }
                write!(f, "{}", err)
            }
            Error::Parse(err) => write!(f, "Error parsing response: {}", err),
        }
    }
}

pub fn view(result: Result<Vec<Release>, Error>) {
    match result {
        Ok(releases) => {
            for release in releases {
                println!("{}", release.tag_name);
            }
        }
        Err(err) => eprintln!("{}", err),
    }
}
