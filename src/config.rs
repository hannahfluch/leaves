use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub old_roots_path: PathBuf,
    pub config_path: PathBuf,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            old_roots_path: PathBuf::from("/persistent/old_roots"),
            config_path: PathBuf::from(shellexpand::tilde("~/nixcfg").into_owned()),
        }
    }
}

// fn main() -> Result<(), confy::ConfyError> {
//     let cfg: MyConfig = confy::load("my-app-name", None)?;
//     Ok(())
// }
