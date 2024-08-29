use serde_with::{serde_as, DisplayFromStr};
use serde::{Deserialize, Serialize};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct JWTResponse {
    #[serde_as(as = "DisplayFromStr")]
    pub jwt_token: String,
    pub expiration: u64,
}
