use log::trace;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use thiserror::Error;
use ucdp::config::Config;

#[derive(Error, Debug)]
pub enum InMemoryDaoError {
    #[error("item not found")]
    ItemNotFound,

    #[error("expired")]
    Expired,

    #[error("config error")]
    Config(#[from] ucdp::config::Error),

    #[error("read lock error")]
    ReadLock,

    #[error("write lock error")]
    WriteLock,

    #[error("time error")]
    Time(#[from] std::time::SystemTimeError),
}

#[derive(Clone, Debug)]
pub struct InMemoryDaoResult<V> {
    pub value: V,
    pub date: SystemTime,
}

pub trait InMemoryDao<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Result<InMemoryDaoResult<V>, InMemoryDaoError>;
    fn put(&self, key: K, value: V);
}

pub struct InMemoryDaoImpl<K, V> {
    hashmap: Arc<RwLock<HashMap<K, InMemoryDaoResult<V>>>>,
}

impl<K: std::fmt::Debug + Eq + std::hash::Hash + Send + Sync, V: Clone + Send + Sync>
    InMemoryDao<K, V> for InMemoryDaoImpl<K, V>
{
    fn get(&self, key: &K) -> Result<InMemoryDaoResult<V>, InMemoryDaoError> {
        trace!("get {:?}", key);
        let hashmap_r = self
            .hashmap
            .read()
            .map_err(|_| InMemoryDaoError::ReadLock)?;
        match hashmap_r.get(key) {
            Some(res) => {
                let duration = SystemTime::now().duration_since(res.date)?;
                // TODO: expiration duration must be data driven
                if duration.as_secs() > 10 {
                    Err(InMemoryDaoError::Expired)
                } else {
                    Ok(res.clone())
                }
            }
            None => Err(InMemoryDaoError::ItemNotFound),
        }
    }

    fn put(&self, key: K, value: V) {
        trace!("put {:?}", key);
        if let Ok(mut hashmap_w) = self
            .hashmap
            .write()
            .map_err(|_| InMemoryDaoError::WriteLock)
        {
            hashmap_w.insert(
                key,
                InMemoryDaoResult {
                    value,
                    date: SystemTime::now(),
                },
            );
        }
    }
}

pub struct InMemoryDaoBuilder<K, V> {
    _k: std::marker::PhantomData<K>,
    _r: std::marker::PhantomData<V>,
}

impl<
        K: 'static + std::fmt::Debug + Eq + std::hash::Hash + Send + Sync,
        V: 'static + Clone + Send + Sync,
    > InMemoryDaoBuilder<K, V>
{
    pub fn build(_: &Config) -> Result<Box<dyn InMemoryDao<K, V>>, InMemoryDaoError> {
        Ok(Box::new(InMemoryDaoImpl {
            hashmap: Arc::new(RwLock::new(HashMap::new())),
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::dal::in_memory_dao::{
        InMemoryDao, InMemoryDaoBuilder, InMemoryDaoError, InMemoryDaoImpl, InMemoryDaoResult,
    };
    use crate::ucdp::dal::Partner;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use std::time::{Duration, SystemTime};
    use ucdp::config::Config;

    #[test]
    fn in_memory_dao_builder_build_ok() {
        let config = config::Config::default();
        let config = Config::from(config);
        let res = InMemoryDaoBuilder::<String, InMemoryDaoResult<Partner>>::build(&config);
        assert!(res.is_ok());
    }

    impl<V: std::cmp::PartialEq + std::clone::Clone> PartialEq for InMemoryDaoResult<V> {
        fn eq(&self, other: &Self) -> bool {
            self.value == other.value // && self.date == other.date
        }
    }

    fn hashmap(time: SystemTime) -> Arc<RwLock<HashMap<String, InMemoryDaoResult<Partner>>>> {
        let mut hashmap = HashMap::<String, InMemoryDaoResult<Partner>>::new();
        hashmap.insert(
            "ABC".into(),
            InMemoryDaoResult {
                value: Partner {
                    name: "ABC".into(),
                    enabled: true,
                },
                date: time,
            },
        );
        hashmap.insert(
            "DEF".into(),
            InMemoryDaoResult {
                value: Partner {
                    name: "DEF".into(),
                    enabled: true,
                },
                date: time,
            },
        );
        Arc::new(RwLock::new(hashmap))
    }

    #[test]
    fn in_memory_dao_get_ok() {
        let dao = InMemoryDaoImpl {
            hashmap: hashmap(SystemTime::now()),
        };

        let res = dao.get(&"ABC".into()).unwrap();
        assert_eq!(
            res,
            InMemoryDaoResult {
                value: Partner {
                    name: "ABC".into(),
                    enabled: true,
                },
                date: SystemTime::UNIX_EPOCH,
            }
        )
    }

    #[test]
    fn in_memory_dao_get_err_not_found() {
        let dao = InMemoryDaoImpl {
            hashmap: hashmap(SystemTime::now()),
        };

        let res = dao.get(&"not found".into());
        match res {
            Err(InMemoryDaoError::ItemNotFound) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn in_memory_dao_get_err_expired() {
        let dao = InMemoryDaoImpl {
            hashmap: hashmap(SystemTime::UNIX_EPOCH),
        };

        let res = dao.get(&"ABC".into());
        match res {
            Err(InMemoryDaoError::Expired) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn in_memory_dao_get_err_time() {
        let dao = InMemoryDaoImpl {
            hashmap: hashmap(SystemTime::now() + Duration::from_secs(60)),
        };

        let res = dao.get(&"ABC".into());
        match res {
            Err(InMemoryDaoError::Time(_)) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn in_memory_dao_put_ok() {
        let dao = InMemoryDaoImpl {
            hashmap: hashmap(SystemTime::now()),
        };
        dao.put(
            "123".into(),
            Partner {
                name: "123".into(),
                enabled: true,
            },
        );
        let res = dao.get(&"123".into()).unwrap();
        assert_eq!(
            res,
            InMemoryDaoResult {
                value: Partner {
                    name: "123".into(),
                    enabled: true,
                },
                date: SystemTime::UNIX_EPOCH,
            }
        )
    }
}
