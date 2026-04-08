mod cli;
mod config;
mod file_entry;
mod formatter;
mod icon;
mod reader;
mod sort;

use clap::Parser;
use std::io;
use std::path::Path;

use cli::Args;
use config::load_config;
use formatter::{format_long, format_one_per_line, format_recursive, format_short};
use reader::{MetadataMode, read_target};

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = load_config();

    let target_path = Path::new(&args.target);

    if !target_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "lx: cannot access '{}': No such file or directory",
                args.target
            ),
        )
        .into());
    }

    if args.recursive {
        format_recursive(target_path, &config, args.show_hidden, args.long);
    } else {
        let metadata_mode = if args.long {
            MetadataMode::Full
        } else {
            MetadataMode::Basic
        };
        let entries = read_target(target_path, args.show_hidden, metadata_mode)?;

        if args.long {
            format_long(entries, &config);
        } else if args.one_per_line {
            format_one_per_line(entries, &config);
        } else {
            format_short(entries, &config);
        }
    }

    Ok(())
}
