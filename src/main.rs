use clap::Parser;
use lx_cli::config::load_config;
use lx_cli::filter::EntryFilter;
use lx_cli::formatter::{
    format_long, format_one_per_line, format_recursive, format_short, format_short_compact,
};
use lx_cli::{Args, MetadataMode, read_target};
use std::io;
use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = load_config();
    let filter = EntryFilter::new(args.files, args.directories, args.exclude);

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
        format_recursive(target_path, &config, args.show_hidden, args.long, &filter)?;
    } else {
        let metadata_mode = if args.long {
            MetadataMode::Full
        } else {
            MetadataMode::Basic
        };
        let entries = filter.apply(read_target(target_path, args.show_hidden, metadata_mode)?);

        if args.long {
            format_long(entries, &config);
        } else if args.one_per_line {
            format_one_per_line(entries, &config);
        } else if args.compact {
            format_short_compact(entries, &config);
        } else {
            format_short(entries, &config);
        }
    }

    Ok(())
}
