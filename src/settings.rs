use serde::{Deserialize, Serialize};

use crate::igdb;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub credentials: igdb::Credentials,
}
