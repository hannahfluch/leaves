use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufReader;

#[derive(Deserialize)]
pub struct PersistentLocations {
    pub directories: HashMap<String, Vec<String>>,
    pub files: HashMap<String, Vec<String>>,
}

impl PersistentLocations {
    pub fn new() -> Option<Self> {
        let Ok(file) = std::fs::File::open("/etc/leaves.json") else {
            return None;
        };

        let rdr = BufReader::new(file);
        serde_json::from_reader(rdr).unwrap()
    }

    pub fn all_locations(&self) -> impl Iterator<Item = String> {
        let mut x: Vec<_> = self
            .directories
            .keys()
            .chain(self.files.keys())
            .cloned()
            .collect();

        x.sort();
        x.dedup();

        x.into_iter()
    }

    pub fn should_be_persisted(&self, location: &str, path: &str) -> PersistStatus {
        let mut location_found = false;
        if let Some(files) = self.files.get(location) {
            if files.iter().any(|x| x.starts_with(path)) {
                return PersistStatus::ParentOfDir;
            }

            if files.iter().any(|x| x == path) {
                return PersistStatus::ChildOrExplicit;
            }
        }

        if let Some(dirs) = self.directories.get(location) {
            location_found = true;
            if dirs.iter().any(|x| path.starts_with(x)) {
                return PersistStatus::ChildOrExplicit;
            }

            if dirs.iter().any(|x| x.starts_with(path)) {
                return PersistStatus::ParentOfDir;
            }
        }

        assert!(location_found);
        PersistStatus::NotPersisted
    }
}

pub enum PersistStatus {
    ParentOfDir,
    ChildOrExplicit,
    NotPersisted,
}
