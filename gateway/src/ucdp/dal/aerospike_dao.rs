use async_trait::async_trait;
use thiserror::Error;
use ucdp::config::Config;

#[derive(Error, Debug)]
pub enum AerospikeDaoError {
    #[error("aerospike error")]
    Aerospike(#[from] aerospike::Error),

    #[error("item not found")]
    ItemNotFound,

    #[error("invalid type: {0}")]
    InvalidType(String),

    #[error("config error")]
    Config(#[from] ucdp::config::Error),
}

pub struct AerospikeDaoResult {
    pub value: Option<Vec<u8>>,
    pub ttl: Option<std::time::Duration>,
}

#[async_trait]
pub trait AerospikeDao: Send + Sync {
    async fn get(&self, key: &str) -> Result<AerospikeDaoResult, AerospikeDaoError>;
    async fn put(&self, key: &str, value: Vec<u8>);
}

pub struct AerospikeDaoImpl {
    client: aerospike::Client,
    set_name: String,
    read_policy: aerospike::ReadPolicy,
    write_policy: aerospike::WritePolicy,
}

#[async_trait]
impl AerospikeDao for AerospikeDaoImpl {
    async fn get(&self, key: &str) -> Result<AerospikeDaoResult, AerospikeDaoError> {
        let key = aerospike::as_key!("ucdp", self.set_name.as_str(), key);
        match self
            .client
            .get(&self.read_policy, &key, aerospike::Bins::All)
        {
            // Item has been fetched
            Ok(record) => {
                let data = record
                    .bins
                    .get("0")
                    .ok_or(AerospikeDaoError::ItemNotFound)?;
                match data {
                    aerospike::Value::Blob(bytes) => Ok(AerospikeDaoResult {
                        value: Some(bytes.to_vec()),
                        ttl: record.time_to_live(),
                    }),
                    v => Err(AerospikeDaoError::InvalidType(v.to_string())),
                }
            }
            // Item does not exist
            Err(aerospike::Error(
                aerospike::ErrorKind::ServerError(aerospike::ResultCode::KeyNotFoundError),
                _,
            )) => Ok(AerospikeDaoResult {
                value: None,
                ttl: None,
            }),
            // Other errors
            Err(e) => Err(AerospikeDaoError::Aerospike(e)),
        }
    }

    async fn put(&self, key: &str, value: Vec<u8>) {
        let key = aerospike::as_key!("ucdp", self.set_name.as_str(), key);
        let bytes: aerospike::Value = value.into();
        let bin = aerospike::as_bin!("0", bytes);
        let _ = self.client.put(&self.write_policy, &key, &[bin]);
    }
}

pub struct AerospikeDaoBuilder {}

impl AerospikeDaoBuilder {
    pub fn build(config: &Config) -> Result<Box<dyn AerospikeDao>, AerospikeDaoError> {
        let set_name = config.get_str("aerospike.set")?;
        let host = config.get_str("aerospike.host")?;

        let mut client_policy = aerospike::ClientPolicy::default().clone();
        client_policy.fail_if_not_connected = false; // it makes testing easier
        let client = aerospike::Client::new(&client_policy, &host)?;

        Ok(Box::new(AerospikeDaoImpl {
            client,
            set_name,
            read_policy: aerospike::ReadPolicy::default(),
            write_policy: aerospike::WritePolicy::default(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::dal::aerospike_dao::{AerospikeDaoBuilder, AerospikeDaoError};
    use ucdp::config::Config;

    #[test]
    fn aerospike_dao_builder_build_aerospike_ok() {
        let mut config = config::Config::default();
        let _ = config.set("aerospike.set", "default");
        let _ = config.set("aerospike.host", "http://aerospike");
        let config = Config::from(config);

        let res = AerospikeDaoBuilder::build(&config);
        assert!(res.is_ok());
    }

    #[test]
    fn aerospike_dao_builder_build_err_config() {
        let config = config::Config::default();
        let config = Config::from(config);

        let res = AerospikeDaoBuilder::build(&config);
        match res {
            Err(AerospikeDaoError::Config(_)) => (),
            _ => unreachable!(),
        }
    }
}
