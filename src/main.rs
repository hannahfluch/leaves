#![feature(normalize_lexically)]

use clap::Parser;

use crate::cli::Commands;

mod cli;
mod commands;
mod config;
mod persistent;
mod roots;

fn main() {
    let args = cli::Args::parse();
    let config: config::Config = confy::load("leaves", None).unwrap();

    match &args.command {
        Commands::Restore { automatic, path } => {
            commands::restore::restore(config, &args, *automatic, path)
        }

        Commands::Query { path, search } => commands::query::query(config, &args, *path, search),

        Commands::Check => commands::check::check(config, &args),
    }
}
