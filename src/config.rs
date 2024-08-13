use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Error as IoError;
use std::path::Path;
use toml::de::Error as TomlError;

#[derive(Deserialize, Debug)]
pub struct Channel {
    pub chan_type: String,
    pub config: HashMap<String, toml::Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub query: String,
    pub channels: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    pub opensearch_url: String,
    pub indexes: Vec<String>,
    pub watch_interval: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: GeneralConfig,
    pub channels: HashMap<String, Channel>,
    pub rules: HashMap<String, Rule>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: Option<u16>,
    pub auth: Option<bool>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct WebhookConfig {
    pub url: String,
    pub auth_user: Option<String>,
    pub auth_pass: Option<String>,
}

#[allow(unused)]
#[derive(Debug)]
pub enum ParsingError {
    IoError(IoError),
    TomlError(toml::de::Error),
}

pub struct DSLRule {
    pub name: String,
    pub query: Value,
    pub channels: HashMap<String, String>,
}

impl TryFrom<&Rule> for DSLRule {
    type Error = serde_json::Error;

    fn try_from(r: &Rule) -> Result<Self, Self::Error> {
        Ok(DSLRule {
            name: r.name.clone(),
            query: serde_json::from_str::<Value>(&r.query)?,
            channels: r.channels.clone(),
        })
    }
}

macro_rules! impl_err {
    ($from_err:ident, $to_err: ident) => {
        impl From<$from_err> for $to_err {
            fn from(from: $from_err) -> $to_err {
                $to_err::$from_err(from)
            }
        }
    };
}

impl_err!(IoError, ParsingError);
impl_err!(TomlError, ParsingError);

impl TryFrom<&Path> for Config {
    type Error = ParsingError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let payload = fs::read_to_string(value)?;

        let conf: Config = toml::from_str(payload.as_str())?;
        Ok(conf)
    }
}
