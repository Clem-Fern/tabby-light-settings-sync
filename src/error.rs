use std::error;
use std::fmt;
use std::io;
use diesel::r2d2;

use crate::models::DbError;

#[derive(Debug)]
pub enum StorageInitializationError {
    Migration(Box<dyn error::Error + Send + Sync + 'static>),
    R2d2(r2d2::PoolError),
    Db(DbError),
    MysqlConnection(diesel::ConnectionError)
}

impl error::Error for StorageInitializationError {}

impl fmt::Display for StorageInitializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Migration(ref err) => write!(f, "Failed to initialize databse storage (diesel migrations): {err}"),
            Self::R2d2(ref err) => write!(f, "Failed to initialize database storage (r2d2 pool manager): {err}"),
            Self::Db(ref err) => write!(f, "Encountered error on database query: {err}"),
            Self::MysqlConnection(ref err) => write!(f, "Encountered error on database connection: {err}"),
        }
    }
}

impl From<Box<dyn error::Error + Send + Sync + 'static>> for StorageInitializationError {
    fn from(err: Box<dyn error::Error + Send + Sync + 'static>) -> StorageInitializationError {
        StorageInitializationError::Migration(err)
    }
}

impl From<r2d2::PoolError> for StorageInitializationError {
    fn from(err: r2d2::PoolError) -> StorageInitializationError {
        StorageInitializationError::R2d2(err)
    }
}

impl From<diesel::ConnectionError> for StorageInitializationError {
    fn from(err: diesel::ConnectionError) -> StorageInitializationError {
        StorageInitializationError::MysqlConnection(err)
    }
}

#[derive(Debug)]
pub enum TlsError {
    Io(io::Error),
    Rustls(rustls::Error)
}

impl error::Error for TlsError {}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Io(ref err) => write!(f, "Encountered IO error while building tls configuration: {err}"),
            Self::Rustls(ref err) => write!(f, "Encountered Rustls error while building tls configuration: {err}"),
        }
    }
}

impl From<rustls::Error> for TlsError {
    fn from(err: rustls::Error) -> TlsError {
        TlsError::Rustls(err)
    }
}

impl From<io::Error> for TlsError {
    fn from(err: io::Error) -> TlsError {
        TlsError::Io(err)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Yaml(serde_yaml::Error),
    #[allow(dead_code)]
    DuplicatedEntry(String),
}

impl error::Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Io(ref err) => write!(f, "Encountered IO error while building deserializing configuration: {err}"),
            Self::Yaml(ref err) => write!(f, "Encountered Yaml error while building deserializing configuration: {err}"),
            Self::DuplicatedEntry(ref entry) => write!(f, "The following data is not unique in configuration: {entry}"),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io(err)
    }
}