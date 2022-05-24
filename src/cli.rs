use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    pub entry: String,
}

pub fn get_args() -> Args {
    Args::parse()
}