use lx_cli::config::load_config;
use lx_cli::filter::EntryFilter;
use lx_cli::formatter::{
    TreeOptions, format_long, format_one_per_line, format_plain, format_recursive, format_short,
    format_short_compact,
};
use lx_cli::git::GitRepository;
use lx_cli::{Args, MetadataMode, read_target};
use std::io::{self, IsTerminal};

fn main() {
    if let Err(error) = run() {
        if error
            .downcast_ref::<io::Error>()
            .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe)
        {
            return;
        }
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_args();
    let mut config = load_config();
    let sort = args.sort_options(config.display.default_sort());
    let filter = EntryFilter::new(args.files, args.directories, args.exclude);

    let target_path = args.target.as_path();

    if !target_path.exists() && !target_path.is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "lx: cannot access '{}': No such file or directory",
                args.target.display()
            ),
        )
        .into());
    }

    let plain_output = args.plain
        || args.zero
        || (!io::stdout().is_terminal()
            && !args.long
            && !args.recursive
            && !args.compact
            && !args.git);
    if plain_output {
        let metadata_mode = if sort.requires_full_metadata() {
            MetadataMode::Full
        } else {
            MetadataMode::Basic
        };
        let entries = filter.apply(read_target(target_path, args.show_hidden, metadata_mode)?);
        format_plain(entries, target_path, sort, args.zero)?;
        return Ok(());
    }

    let repository = if args.git || args.git_ignore {
        match GitRepository::load(target_path, args.git_ignore) {
            Ok(repository) => repository,
            Err(error) => {
                if args.git_ignore {
                    return Err(io::Error::other(format!(
                        "lx: cannot read Git ignore rules: {error}"
                    ))
                    .into());
                }
                eprintln!("lx: cannot read Git status: {error}");
                None
            }
        }
    } else {
        None
    };
    let git = repository.as_ref().filter(|_| args.git);
    if git.is_some() {
        if !config
            .display
            .long_format_fields
            .iter()
            .any(|field| field == "git")
        {
            config
                .display
                .long_format_fields
                .insert(0, "git".to_string());
        }
    } else {
        config
            .display
            .long_format_fields
            .retain(|field| field != "git");
    }

    if args.recursive {
        format_recursive(
            target_path,
            &config,
            &TreeOptions {
                show_hidden: args.show_hidden,
                use_long_format: args.long,
                depth: args.depth,
                ignored: repository.as_ref().filter(|_| args.git_ignore),
            },
            &filter,
            sort,
            git,
        )?;
    } else {
        let metadata_mode = if args.long || sort.requires_full_metadata() {
            MetadataMode::Full
        } else {
            MetadataMode::Basic
        };
        let mut entries = filter.apply(read_target(target_path, args.show_hidden, metadata_mode)?);
        if let Some(repository) = &git {
            repository.annotate(target_path, &mut entries)?;
        }

        if args.long {
            format_long(entries, &config, sort)?;
        } else if args.one_per_line {
            format_one_per_line(entries, &config, sort)?;
        } else if args.compact {
            format_short_compact(entries, &config, sort)?;
        } else {
            format_short(entries, &config, sort)?;
        }
    }

    Ok(())
}
