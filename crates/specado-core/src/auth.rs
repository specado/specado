use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthScheme {
    Bearer { token_env: String },
    ApiKey { header: String, key_env: String },
}
