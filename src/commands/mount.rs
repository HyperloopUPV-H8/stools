use std::{error, fmt::Display, fs::File, io, path::PathBuf};

use clap::Args;
use zip::{result::ZipError, ZipArchive};

use super::Target;

#[derive(Args, Debug, Clone)]
pub struct Params {
    pub(crate) target: Target,

    #[clap(short, default_value = "./stools")]
    /// Path where the files are stored.
    pub(crate) path: PathBuf,
}

pub fn run(params: &Params) -> Result<(), Error> {
    match params.target {
        Target::Backend => mount_backend(),
        Target::Control | Target::Ethernet => mount_frontend(params),
    }
}

fn mount_backend() -> Result<(), Error> {
    return Ok(());
}

const FRONTEND_FILE_NAME: &str = "static.zip";

fn mount_frontend(params: &Params) -> Result<(), Error> {
    let file = File::open(params.path.join(FRONTEND_FILE_NAME))?;

    let mut archive = ZipArchive::new(file)?;

    archive.extract(&params.path)?;

    return Ok(());
}

#[derive(Debug)]
pub enum Error {
    File(io::Error),
    Zip(ZipError),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        return Error::File(err);
    }
}

impl From<ZipError> for Error {
    fn from(err: ZipError) -> Self {
        return Error::Zip(err);
    }
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::File(err) => write!(f, "file error: {}", err),
            Error::Zip(err) => write!(f, "zip error: {}", err),
        }
    }
}

pub fn view(result: Result<(), Error>) {
    match result {
        Ok(_) => println!("Target mounted correctly"),
        Err(err) => eprintln!("Error mounting: {}", err),
    }
}
