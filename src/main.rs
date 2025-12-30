mod cli;
mod config;
mod file_entry;
mod formatter;
mod icon;
mod reader;

use clap::Parser;
use std::path::Path;

use cli::Args;
use config::load_config;
use formatter::{format_long, format_one_per_line, format_recursive, format_short};
use reader::read_directory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = load_config();

    let target_path = Path::new(&args.target);

    if !target_path.exists() {
        eprintln!(
            "lx: cannot access '{}': No such file or directory",
            args.target
        );
        return Ok(());
    }

    if args.recursive {
        format_recursive(target_path, &config, args.show_hidden);
    } else {
        let entries = read_directory(target_path, args.show_hidden)?;

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
