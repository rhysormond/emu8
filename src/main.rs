use chip8::run;
use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Run(Run),
}

#[derive(Parser)]
struct Run {
    #[clap(short, long, value_hint = ValueHint::AnyPath)]
    file: PathBuf,
}

fn main() {
    match Args::parse().subcmd {
        SubCommand::Run(args) => run(args.file),
    };
}
