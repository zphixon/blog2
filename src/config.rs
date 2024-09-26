use serde::{de::Visitor, Deserialize};
use std::{
    net::SocketAddr,
    ops::Deref,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Deserialize, Debug)]
pub struct DbConfig {
    pub sqlite_file: ValidPath,
}

#[derive(Deserialize, Debug)]
pub struct NetConfig {
    pub proto_host: Url,
    pub base_path: String,
    pub bind: SocketAddr,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub db: DbConfig,
    pub net: NetConfig,
}

#[derive(Debug)]
pub struct ValidPath(PathBuf);

impl<'de> Deserialize<'de> for ValidPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValidPathVisitor;
        impl Visitor<'_> for ValidPathVisitor {
            type Value = ValidPath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a valid path")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValidPath(
                    PathBuf::from(v).canonicalize().map_err(E::custom)?,
                ))
            }
        }

        Ok(deserializer.deserialize_str(ValidPathVisitor)?)
    }
}

impl Deref for ValidPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_path()
    }
}
