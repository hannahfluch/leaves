use std::{path::PathBuf, time::Duration};

use walkdir::{DirEntry, WalkDir};

use crate::{cli::Args, config::Config};

pub fn get_all(config: &Config, args: &Args) -> Vec<PathBuf> {
    let max_dur = if let Some(days) = args.last_n_days {
        Duration::from_hours(days as u64 * 24)
    } else if let Some(weeks) = args.last_n_weeks {
        Duration::from_hours(weeks as u64 * 7 * 24)
    } else {
        Duration::MAX
    };

    let current = config.exclude_current.as_ref();
    let link = current.and_then(|current| {
        std::fs::read_link(config.old_roots_path.join(current))
            .map(|x| config.old_roots_path.join(x))
            .ok()
    });

    std::fs::read_dir(&config.old_roots_path)
        .unwrap()
        .filter_map(|x| x.ok())
        .filter(|x| {
            current.is_none_or(|c| &*x.file_name().to_string_lossy() != c.as_str())
                && link.as_ref().is_none_or(|p| p != &x.path())
        })
        .filter(|x| {
            let age = x.metadata().unwrap().created().unwrap().elapsed().unwrap();

            age < max_dur
        })
        .map(|x| x.path())
        .collect()
}

pub fn walk_all(config: &Config, args: &Args) -> impl Iterator<Item = DirEntry> {
    let roots = get_all(config, args);

    roots
        .into_iter()
        .flat_map(|x| {
            WalkDir::new(x).into_iter().filter_entry(|e| {
                !args.ignore_hidden || { e.file_name().to_string_lossy().starts_with('.') }
            })
        })
        .filter_map(Result::ok)
}
