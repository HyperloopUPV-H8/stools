use std::{
    error,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::Args;
use reqwest::Client;
use tokio::task::{JoinError, JoinHandle};

use super::{
    list::{self, Asset},
    Target,
};

#[derive(Args, Debug, Clone)]
pub struct Params {
    pub(crate) target: Target,

    /// Optional version tag to download.
    pub(crate) tag: Option<String>,

    #[clap(short, default_value = "./stools")]
    /// Output path for the downloaded files.
    pub(crate) output: PathBuf,
}

pub async fn run(
    params: &Params,
    client: &Client,
) -> Result<Vec<Result<WorkerOutput, JoinError>>, Error> {
    let releases = list::run(
        &list::Params {
            target: params.target.clone(),
        },
        client,
    )
    .await?;

    let index = match &params.tag {
        Some(tag) => releases
            .iter()
            .position(|release| &release.tag_name == tag)
            .ok_or(Error::TagNotFound(tag.clone()))?,
        None => 0,
    };

    if !params.output.exists() {
        fs::create_dir_all(&params.output)?;
    }

    let mut handler = Handler::new();

    for asset in releases[index].assets.iter() {
        let worker = Worker::new(asset.clone(), client.clone(), params.output.clone())?;
        handler.add_worker(worker);
    }

    return Ok(handler.join().await);
}

struct Handler {
    workers: Vec<JoinHandle<WorkerOutput>>,
}

impl Handler {
    fn new() -> Self {
        return Self {
            workers: Vec::new(),
        };
    }

    fn add_worker(&mut self, worker: Worker) {
        self.workers.push(tokio::spawn(worker.download()))
    }

    async fn join(self) -> Vec<Result<WorkerOutput, JoinError>> {
        let mut result = Vec::new();
        for worker in self.workers {
            result.push(worker.await);
        }
        return result;
    }
}

type WorkerOutput = (Worker, Result<(), Error>);

pub struct Worker {
    asset: Asset,
    client: Client,
    file: File,
}

impl Worker {
    fn new<P>(asset: Asset, client: Client, path: P) -> Result<Worker, Error>
    where
        P: AsRef<Path>,
    {
        let output = path.as_ref().join(&asset.name);
        let file = File::create(output)?;

        return Ok(Self {
            asset,
            client,
            file,
        });
    }

    async fn download(mut self) -> WorkerOutput {
        let mut request = match self
            .client
            .get(&self.asset.browser_download_url)
            .header("Accept", &self.asset.content_type)
            .send()
            .await
        {
            Ok(req) => req,
            Err(err) => return (self, Err(err.into())),
        };

        loop {
            let next = match request.chunk().await {
                Ok(chunk) => chunk,
                Err(err) => return (self, Err(err.into())),
            };

            let Some(chunk) = next else {
                return (self, Ok(()));
            };

            let mut n = 0;
            while n < chunk.len() {
                match self.file.write(&chunk[n..]) {
                    Ok(w) => n += w,
                    Err(err) => return (self, Err(err.into())),
                };
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Parse(serde_json::Error),
    Request(reqwest::Error),
    File(io::Error),
    TagNotFound(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        return Error::Request(err);
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        return Error::File(err);
    }
}

impl From<list::Error> for Error {
    fn from(err: list::Error) -> Self {
        match err {
            list::Error::Parse(err) => Error::Parse(err),
            list::Error::Request(err) => Error::Request(err),
        }
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Request(err) => write!(f, "request error: {}", err),
            Error::File(err) => write!(f, "file error: {}", err),
            Error::Parse(err) => write!(f, "parse error: {}", err),
            Error::TagNotFound(err) => write!(f, "tag {} not found", err),
        }
    }
}

pub fn view(result: Result<Vec<Result<WorkerOutput, JoinError>>, Error>) {
    match result {
        Ok(downloads) => view_downloads(downloads),
        Err(err) => eprintln!("{}", err),
    }
}

fn view_downloads(downloads: Vec<Result<WorkerOutput, JoinError>>) {
    for download in downloads {
        match download {
            Ok(output) => view_worker_output(output),
            Err(err) => eprintln!("worker panic: {}", err),
        }
    }
}

fn view_worker_output((worker, output): WorkerOutput) {
    match output {
        Ok(_) => println!("{}", worker.asset.name),
        Err(err) => eprintln!("error downloading {}: {}", worker.asset.name, err),
    }
}
