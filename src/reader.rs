use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ReadError {
    Io(IoError),
    Parse(String),
}

pub trait Reader<T> {
    fn read(&self) -> Result<T, ReadError>;
}

#[derive(Debug)]
pub struct FileReader(PathBuf);

impl From<PathBuf> for FileReader {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}

impl Reader<String> for FileReader {
    fn read(&self) -> Result<String, ReadError> {
        self.read_str().map(|value| value.trim().to_string())
    }
}

impl Reader<u8> for FileReader {
    fn read(&self) -> Result<u8, ReadError> {
        self.read_str().and_then(|value| {
            value
                .trim()
                .parse::<u8>()
                .map_err(|err| ReadError::Parse(err.to_string()))
        })
    }
}

impl Reader<u64> for FileReader {
    fn read(&self) -> Result<u64, ReadError> {
        self.read_str().and_then(|value| {
            value
                .trim()
                .parse::<u64>()
                .map_err(|err| ReadError::Parse(err.to_string()))
        })
    }
}

impl Reader<bool> for FileReader {
    fn read(&self) -> Result<bool, ReadError> {
        self.read().map(|value: u8| value > 0)
    }
}

impl FileReader {
    fn read_str(&self) -> Result<String, ReadError> {
        fs::read_to_string(&self.0).map_err(ReadError::Io)
    }
}
