use chip8::run::run;
use clap::{Clap, ValueHint};
use std::path::PathBuf;

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Run(Run),
}

#[derive(Clap)]
struct Run {
    #[clap(short, long, value_hint = ValueHint::AnyPath)]
    file: PathBuf,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Run(args) => run(args.file),
    };
}
