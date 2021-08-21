use std::str::FromStr;
use web3::contract::Options;

#[derive(Debug)]
pub struct Partner {
    pub name: String,
    pub enabled: bool,
}

pub struct Contract {
    pub contract: web3::contract::Contract<web3::transports::Http>,
}

impl Contract {
    pub fn new(address: &str, json: &[u8]) -> Self {
        let http = web3::transports::Http::new("http://localhost:9545").unwrap();
        let web3 = web3::Web3::new(http);
        let contract = web3::contract::Contract::from_json(
            web3.eth(),
            web3::types::Address::from_str(address).unwrap_or_default(),
            json,
        )
        .unwrap();
        Contract { contract }
    }

    pub async fn get_partner(&self, address: &str) -> Partner {
        let res: Result<(Vec<u8>, bool), _> = self
            .contract
            .query(
                "partners",
                (web3::types::Address::from_str(address).unwrap_or_default(),),
                None,
                Options::default(),
                None,
            )
            .await;
        let (name, enabled) = res.unwrap_or_default();
        Partner {
            name: String::from_utf8(name)
                .unwrap_or_default()
                .trim_end_matches(char::from(0))
                .into(),
            enabled,
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::ucdp::contract::Contract;
    use crate::ucdp::contract::Partner;

    impl PartialEq for Partner {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.enabled == other.enabled
        }
    }

    #[actix_rt::test]
    async fn contract_get_partner() {
        let contract = Contract::new(
            "0xa80E74Ee52efc3D28CF3778d1B54B4dc0c23028b",
            include_bytes!("../../../smart-contracts/build/contracts/abi/Ucdp.abi.json"),
        );

        let registered_partner = contract
            .get_partner("0x0000000000000000000000000000000000000123")
            .await;
        assert_eq!(
            registered_partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        );

        let unregistered_partner = contract
            .get_partner("0x8888888888888888888888888888888888888888")
            .await;
        assert_eq!(
            unregistered_partner,
            Partner {
                name: "".into(),
                enabled: false
            }
        );
    }
}
*/
