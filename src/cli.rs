use clap::{Parser, Subcommand};

/// Interact with old persistent roots
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Only consider roots that were created in the last <N> days
    #[arg(short = 'd', conflicts_with = "last_n_weeks", long, value_name = "N")]
    pub last_n_days: Option<u32>,

    /// Only consider roots that were created in the last <N> weeks
    #[arg(short = 'w', conflicts_with = "last_n_days", long, value_name = "N")]
    pub last_n_weeks: Option<u16>,

    /// Ignore hidden files (all starting with .)
    #[arg(short, long, default_value_t = false)]
    pub ignore_hidden: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Restore a file by its path
    #[command(arg_required_else_help = true, alias = "r")]
    Restore {
        #[arg(short, long)]
        automatic: bool,

        #[arg()]
        path: String,
    },

    /// Find a file by name or regex query
    #[command(arg_required_else_help = true, alias = "q")]
    Query {
        #[arg(short, long)]
        path: bool,

        #[arg()]
        search: String,
    },

    /// Check persistent data for leftover files
    #[command(alias = "c")]
    Check,
}
