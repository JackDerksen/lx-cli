/// Defines CLI arguments using `clap`.
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
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

    #[arg(short = '1', help = "Force single column output")]
    pub one_per_line: bool,
}
