use clap::Parser;
use mameda::command_run::RunCommand;
use orfail::OrFail;

#[derive(Parser)]
struct Args {
    // TODO?: use a random free port if omitted
    #[clap(short, long, default_value_t = 4343)]
    port: u16,

    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Run(RunCommand),
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Run(x) => x.run(args.port).or_fail()?,
    }
    Ok(())
}
