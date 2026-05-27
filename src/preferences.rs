use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::igdb;

pub type Preferences = Arc<RwLock<PreferencesInner>>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PreferencesInner {
    pub credentials: igdb::Credentials,
}
