/// Defines CLI arguments using `clap`.
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".")]
    pub target: String,

    #[arg(short, long, help = "Use a long listing format")]
    pub long: bool,
}
