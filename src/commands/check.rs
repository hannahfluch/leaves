use walkdir::WalkDir;

use crate::{
    cli::Args,
    config::Config,
    persistent::{PersistStatus, PersistentLocations},
};

pub fn check(_config: Config, _args: &Args) {
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
                    println!("Leftover path: {}{}", store_path, location_in_store);
                    found_any = true;
                    false
                }
            }
        }) {}
    }

    if !found_any {
        println!("Didn't find any leftover paths!");
    }
}
