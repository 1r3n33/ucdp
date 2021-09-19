use crate::ucdp::config::Config;

pub struct Error {}

pub struct CacheEntry {
    pub value: Option<Vec<u8>>,
    pub ttl: Option<std::time::Duration>,
}

pub trait CacheDao: Send + Sync {
    fn get(&self, key: &str) -> Result<CacheEntry, Error>;
    fn put(&self, key: &str, value: Vec<u8>);
}

pub struct AerospikeCacheDao {
    client: aerospike::Client,
    set_name: String,
    read_policy: aerospike::ReadPolicy,
    write_policy: aerospike::WritePolicy,
}

impl CacheDao for AerospikeCacheDao {
    fn get(&self, key: &str) -> Result<CacheEntry, Error> {
        let key = aerospike::as_key!("ucdp", self.set_name.as_str(), key);
        match self
            .client
            .get(&self.read_policy, &key, aerospike::Bins::All)
        {
            // Item has been fetched
            Ok(record) => {
                let data = record.bins.get("0");
                match data {
                    Some(aerospike::Value::Blob(bytes)) => Ok(CacheEntry {
                        value: Some(bytes.to_vec()),
                        ttl: record.time_to_live(),
                    }),
                    _ => Err(Error {}),
                }
            }
            // Item does not exist
            Err(aerospike::Error(
                aerospike::ErrorKind::ServerError(aerospike::ResultCode::KeyNotFoundError),
                _,
            )) => Ok(CacheEntry {
                value: None,
                ttl: None,
            }),
            // Other errors
            _ => Err(Error {}),
        }
    }

    fn put(&self, key: &str, value: Vec<u8>) {
        let key = aerospike::as_key!("ucdp", self.set_name.as_str(), key);
        let bytes: aerospike::Value = value.into();
        let bin = aerospike::as_bin!("0", bytes);
        let _ = self.client.put(&self.write_policy, &key, &[bin]);
    }
}

pub struct CacheBuilder {}

impl CacheBuilder {
    pub fn build(config: &Config, prefix: &str) -> Result<Box<dyn CacheDao>, Error> {
        let cache_type_key = String::new() + prefix + ".cache";
        match config.get_str(cache_type_key.as_str()) {
            Ok(cache) if cache == "aerospike" => {
                let set_name_key = String::new() + prefix + ".aerospike.set";
                let set_name = config
                    .get_str(set_name_key.as_str())
                    .unwrap_or_else(|_| "default".into());
                let host = config
                    .get_str("cache.aerospike.host")
                    .unwrap_or_else(|_| "127.0.0.1:3000".into());

                let mut client_policy = aerospike::ClientPolicy::default().clone();
                client_policy.fail_if_not_connected = false; // it makes testing easier

                let aerospike_client = aerospike::Client::new(&client_policy, &host);
                match aerospike_client {
                    Ok(client) => Ok(Box::new(AerospikeCacheDao {
                        client,
                        set_name,
                        read_policy: aerospike::ReadPolicy::default(),
                        write_policy: aerospike::WritePolicy::default(),
                    })),
                    _ => Err(Error {}),
                }
            }
            _ => Err(Error {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::cache::CacheBuilder;
    use crate::ucdp::config::Config;

    #[test]
    fn cachebuilder_build_aerospike_ok() {
        let mut config = config::Config::default();
        let _ = config.set("prefix.cache", "aerospike");
        let config = Config::from(config);

        let res = CacheBuilder::build(&config, "prefix");
        assert!(res.is_ok());
    }

    #[test]
    fn cachebuilder_build_unknown_err() {
        let mut config = config::Config::default();
        let _ = config.set("prefix.cache", "does not exist");
        let config = Config::from(config);

        let res = CacheBuilder::build(&config, "prefix");
        assert!(res.is_err());
    }
}
