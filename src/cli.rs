use crate::sort::{SortField, SortOptions, SortOrder};
/// Defines CLI arguments using `clap`.
use clap::{ArgGroup, Parser};
use std::ffi::OsString;

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
        short = 's',
        long,
        value_enum,
        value_name = "FIELD",
        help = "Sort by name, size, modified time, type, or long-format field"
    )]
    pub sort: Option<SortField>,

    #[arg(
        long,
        value_enum,
        value_name = "ORDER",
        requires = "sort",
        help = "Sort ascending or descending (ascending by default)"
    )]
    pub sort_order: Option<SortOrder>,

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

impl Args {
    pub fn parse_args() -> Self {
        Self::try_parse_args_from(std::env::args_os()).unwrap_or_else(|error| error.exit())
    }

    pub fn try_parse_args_from<I, T>(arguments: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        <Self as Parser>::try_parse_from(normalize_sort_order(arguments))
    }

    pub fn sort_options(&self, configured_default: SortOptions) -> SortOptions {
        self.sort
            .map(|field| SortOptions::new(Some(field), self.sort_order.unwrap_or_default()))
            .unwrap_or(configured_default)
    }
}

fn normalize_sort_order<I, T>(arguments: I) -> Vec<OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
    let mut normalized = Vec::with_capacity(arguments.len() + 2);
    let mut index = 0;

    while let Some(argument) = arguments.get(index) {
        match sort_argument_form(argument) {
            Some(SortArgumentForm::SeparateValue) => {
                normalized.push(argument.clone());
                if let Some(field) = arguments.get(index + 1) {
                    normalized.push(field.clone());
                    if let Some(order) = arguments
                        .get(index + 2)
                        .filter(|argument| is_sort_order(argument))
                    {
                        normalized.push("--sort-order".into());
                        normalized.push(order.clone());
                        index += 3;
                        continue;
                    }
                    index += 2;
                    continue;
                }
                index += 1;
                continue;
            }
            Some(SortArgumentForm::AttachedValue) => {
                normalized.push(argument.clone());
                if let Some(order) = arguments
                    .get(index + 1)
                    .filter(|argument| is_sort_order(argument))
                {
                    normalized.push("--sort-order".into());
                    normalized.push(order.clone());
                    index += 2;
                    continue;
                }
                index += 1;
                continue;
            }
            None => {}
        }

        normalized.push(argument.clone());
        index += 1;
    }

    normalized
}

enum SortArgumentForm {
    SeparateValue,
    AttachedValue,
}

fn sort_argument_form(argument: &OsString) -> Option<SortArgumentForm> {
    let argument = argument.to_str()?;

    if argument == "--sort" || argument == "-s" || is_short_flag_group_ending_in_sort(argument) {
        return Some(SortArgumentForm::SeparateValue);
    }

    if argument.starts_with("--sort=") || (argument.starts_with("-s") && argument.len() > 2) {
        return Some(SortArgumentForm::AttachedValue);
    }

    None
}

fn is_short_flag_group_ending_in_sort(argument: &str) -> bool {
    let Some(flags) = argument.strip_prefix('-') else {
        return false;
    };

    flags.len() > 1
        && flags.ends_with('s')
        && flags[..flags.len() - 1]
            .chars()
            .all(|flag| matches!(flag, 'a' | 'l' | 'f' | 'd' | '1' | 'c' | 'r'))
}

fn is_sort_order(argument: &OsString) -> bool {
    matches!(argument.to_str(), Some("asc" | "desc"))
}

#[cfg(test)]
mod tests {
    use super::Args;
    use crate::sort::{SortField, SortOrder};
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
        let sort = Args::try_parse_from(["lx", "-s", "size", "--sort-order", "desc"])
            .expect("parse sort options");
        assert_eq!(sort.sort, Some(SortField::Size));
        assert_eq!(sort.sort_order, Some(SortOrder::Desc));
        assert!(Args::try_parse_from(["lx", "--sort-order", "desc"]).is_err());
    }
}
