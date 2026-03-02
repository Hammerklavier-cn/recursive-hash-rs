use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub mode: Mode,
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Mode {
    Cli {
        #[arg(
            short,
            long,
            help = "Check file hashes according to the checklist file."
        )]
        audit: Option<PathBuf>,

        #[arg(
            short,
            long,
            default_value = ".",
            help = "Path to the file or directory to hash"
        )]
        paths: Vec<PathBuf>,

        #[arg(
            short,
            long,
            required = false,
            value_delimiter = ',',
            default_value = "checklist,checklist.md5,checklist.sha1,checklist.sha256,checklist.sha384,checklist.sha512",
            help = "Path to the file or directory to exclude from hashing"
        )]
        excludes: Vec<PathBuf>,

        #[arg(
            short = 'm',
            long,
            default_value = "sha256",
            help = "Hashing algorithm to use"
        )]
        hasher: HasherMode,

        #[arg(
            short,
            long,
            default_value = "checklist.sha256",
            help = "Path to the output file"
        )]
        out: String,
    },
    Gui,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum HasherMode {
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}
