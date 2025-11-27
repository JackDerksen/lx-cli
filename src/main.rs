mod cli;
mod file_entry;
mod formatter;
mod icon;
mod reader;

use clap::Parser;
use std::path::Path;

use cli::Args;
use formatter::Formatter;
use reader::read_directory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let target_path = Path::new(&args.target);

    if !target_path.exists() {
        eprintln!(
            "lx: cannot access '{}': No such file or directory",
            args.target
        );
        return Ok(());
    }

    let entries = read_directory(target_path, args.show_hidden)?;

    if args.long {
        Formatter::format_long(entries);
    } else {
        Formatter::format_short(entries);
    }

    Ok(())
}
