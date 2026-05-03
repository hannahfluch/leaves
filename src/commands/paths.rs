use crate::{cli::Args, config::Config, persistent::PersistentLocations};

pub fn paths(_config: Config, _args: &Args) {
    let PersistentLocations {
        directories,
        files,
        mut extra,
    } = PersistentLocations::new().expect("to read /etc/leaves.json");

    let stats = {
        let x: usize = files.values().map(|x| x.len()).sum();
        let y: usize = directories.values().map(|x| x.len()).sum();
        (x, y)
    };

    println!(
        "Total mounted files {}, total mounted directories {}",
        stats.0, stats.1
    );

    let mut joined = directories;
    for (k, v) in files {
        let entry = joined.entry(k).or_default();
        for x in v {
            entry.push(x);
        }
    }

    let mut joined: Vec<_> = joined.into_iter().collect();
    joined.sort_unstable_by_key(|(x, _)| x.clone());
    for (i, mut v) in joined {
        println!("\n{i}");
        v.sort_unstable();
        for x in v {
            println!("\t{x}");
        }
    }

    if !extra.is_empty() {
        println!("\n\nExtra paths:");
        extra.sort_unstable();
        for p in extra {
            println!("\t{p}");
        }
    }
}
