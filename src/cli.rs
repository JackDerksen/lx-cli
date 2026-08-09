/// Defines CLI arguments using `clap`.
use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    group(
        ArgGroup::new("display_mode")
            .args(["long", "one_per_line"])
            .multiple(false)
    ),
    after_help = "Examples:\n  lx -la\n  lx -lr\n  lx -alr path/to/dir"
)]
pub struct Args {
    #[arg(default_value = ".")]
    pub target: String,

    #[arg(short, long, help = "Use a long listing format")]
    pub long: bool,

    #[arg(
        short = 'a',
        long = "all",
        help = "Show all files, including hidden ones"
    )]
    pub show_hidden: bool,

    #[arg(
        short = 'f',
        long = "files",
        help = "Show only files",
        conflicts_with = "recursive"
    )]
    pub files: bool,

    #[arg(
        short = 'd',
        long = "directories",
        help = "Show only directories",
        conflicts_with = "recursive"
    )]
    pub directories: bool,

    #[arg(
        short = 'x',
        long,
        value_name = "PATTERN",
        value_delimiter = ',',
        help = "Exclude comma-separated glob patterns (repeatable)"
    )]
    pub exclude: Vec<String>,

    #[arg(
        short = '1',
        help = "Force single column output",
        conflicts_with = "recursive"
    )]
    pub one_per_line: bool,

    #[arg(
        short = 'c',
        long,
        help = "Use compact columns with a limited number of rows",
        conflicts_with_all = ["long", "one_per_line", "recursive"]
    )]
    pub compact: bool,

    #[arg(
        short = 'r',
        long = "recursive",
        help = "Show directory tree recursively"
    )]
    pub recursive: bool,
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;

    #[test]
    fn handles_short_flag_groups_and_conflicts() {
        let args = Args::try_parse_from(["lx", "-alr"]).expect("parse -alr");

        assert!(args.show_hidden);
        assert!(args.long);
        assert!(args.recursive);
        assert_eq!(args.target, ".");
        assert!(
            Args::try_parse_from(["lx", "-c"])
                .expect("parse -c")
                .compact
        );
        assert!(Args::try_parse_from(["lx", "-l1"]).is_err());
        assert!(Args::try_parse_from(["lx", "-1r"]).is_err());
        assert!(Args::try_parse_from(["lx", "-cl"]).is_err());
        let filters = Args::try_parse_from(["lx", "-fd"]).expect("parse -fd");
        assert!(filters.files && filters.directories);
        assert!(Args::try_parse_from(["lx", "-rf"]).is_err());
        let excluded = Args::try_parse_from(["lx", "-x", ".git,target,*.toml"])
            .expect("parse multiple exclusion patterns");
        assert_eq!(excluded.exclude, [".git", "target", "*.toml"]);
    }
}
