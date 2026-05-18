use walkdir::WalkDir;

use crate::persistent::{PersistStatus, PersistentLocations};

pub fn check() {
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
                    left_over.push(f_location.into_owned());
                    found_any = true;
                    false
                }
            }
        }) {}
    }

    let mut extra = persistent_locs.extra;
    let extra_initial = extra.len();
    let mut not_in_extra = vec![];
    for p in left_over {
        let same = extra
            .iter()
            .position(|x| x == &p || x.trim_start_matches(&p) == "/");

        if let Some(pos) = same {
            extra.swap_remove(pos);
        } else {
            not_in_extra.push(p);
        }
    }

    if extra_initial != 0 {
        println!("Extra paths used: {}!", extra_initial - extra.len());
    }

    for p in not_in_extra {
        println!("Leftover path: {p}");
    }

    for e in extra {
        println!("Unused extra config: {e}");
    }

    if !found_any {
        println!("Didn't find any leftover paths!");
    }
}
