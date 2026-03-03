use std::{path::PathBuf, process::Command};

use walkdir::WalkDir;

use crate::{
    cli::Args,
    config::Config,
    persistent::{PersistStatus, PersistentLocations},
};

enum RGCheck {
    DidntFind,
    OnlySuffix,
    EntirePath,
}

fn ripgrep(config: &Config, prefix: &str, suffix: &str) -> RGCheck {
    let path = prefix.to_string() + suffix;
    let config_path = config.config_path.as_os_str().to_string_lossy();

    let status = Command::new("rg")
        .args(["-F", &path, &config_path])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if status.success() {
        return RGCheck::EntirePath;
    }

    let suffix = PathBuf::from(suffix)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let status = Command::new("rg")
        .args(["-F", &suffix, &config_path])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if status.success() {
        RGCheck::OnlySuffix
    } else {
        RGCheck::DidntFind
    }
}

pub fn check(config: Config, _args: &Args) {
    let persistent_locs = PersistentLocations::new().expect("to read /etc/leaves.json");

    let stats = {
        let x: usize = persistent_locs.files.values().map(|x| x.len()).sum();
        let y: usize = persistent_locs.directories.values().map(|x| x.len()).sum();
        (x, y)
    };

    println!(
        "Total mounted files {}, total mounted directories {}!",
        stats.0, stats.1
    );

    let mut found_any = false;
    let mut left_over = vec![];
    for store_path in persistent_locs.all_locations() {
        for _ in WalkDir::new(&store_path).into_iter().filter_entry(|e| {
            let f_location = e.path().to_string_lossy();
            if f_location == store_path {
                return true;
            }

            let location_in_store = f_location.strip_prefix(&store_path).unwrap();

            let status = persistent_locs.should_be_persisted(&store_path, location_in_store);

            match status {
                PersistStatus::ParentOfDir => true,
                PersistStatus::ChildOrExplicit => false, // Dont explore persisted directories further
                PersistStatus::NotPersisted => {
                    left_over.push((store_path.to_owned(), location_in_store.to_owned()));
                    found_any = true;
                    false
                }
            }
        }) {}
    }

    let mut didnt_find = vec![];
    let mut just_suffix = vec![];
    let mut full_match = vec![];

    for (prefix, suffix) in left_over {
        match ripgrep(&config, &prefix, &suffix) {
            RGCheck::DidntFind => didnt_find.push((prefix, suffix)),
            RGCheck::OnlySuffix => just_suffix.push((prefix, suffix)),
            RGCheck::EntirePath => full_match.push((prefix, suffix)),
        }
    }

    println!("{} full paths found with ripgrep!", full_match.len());
    println!("{} path names found with ripgrep!", just_suffix.len());

    for (p, s) in just_suffix {
        println!("Could be used in config: {p}{s}");
    }

    for (p, s) in didnt_find {
        println!("Leftover path: {p}{s}");
    }

    if !found_any {
        println!("Didn't find any leftover paths!");
    }
}
