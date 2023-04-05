use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use strum_macros::Display;

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Run a given binary or assembly file
    Run {
        #[arg(long)]
        auto: bool,
    },

    /// Convert a file from binary to assembly or vice versa
    Convert {
        /// Output path
        #[arg(short, long)]
        out_path: PathBuf,
    },
}

#[derive(ValueEnum, Display, Clone, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum FileType {
    Binary,
    Assembly,
}

impl FileType {
    pub fn swap(&self) -> Self {
        match &self {
            Self::Binary => Self::Assembly,
            Self::Assembly => Self::Binary,
        }
    }
}

/// A Rust Implementation of the Synacor VM
#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub ftype: FileType,

    /// Input file path
    #[arg(short, long)]
    pub path: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}
