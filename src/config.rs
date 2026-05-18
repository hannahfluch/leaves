use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub old_roots_path: PathBuf,
    pub exclude_current: Option<String>,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            old_roots_path: PathBuf::from("/persistent/old_roots"),
            exclude_current: None,
        }
    }
}

// fn main() -> Result<(), confy::ConfyError> {
//     let cfg: MyConfig = confy::load("my-app-name", None)?;
//     Ok(())
// }
