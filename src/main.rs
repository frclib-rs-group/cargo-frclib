mod actions;
mod cmds;
mod configs;
mod errors;

use clap::Parser;
use errors::RuntimeError;
use configs::project_config;

// # cargo-frc
// cargo-frc is a Cargo subcommand for building and deploying Rust code in the FRC ecosystem.
// It is borderline required for using `frclib`.
//
// ## Commands
//
// ### `cargo frc check`
//    Runs `cargo check` with the correct flags for the FRC ecosystem.]
//
// ### `cargo frc sim`
//    Runs `cargo run` to run code on local machine with the `frc_sim` cfg.
//
// ### `cargo frc deploy`
//    Deploys code to the robot with the `frc_real`.
//
// ### `cargo frc tui`
//    Runs the TUI for managing the robot.
//
// ### `cargo frc test`
//    Runs `cargo test` with the correct flags for the FRC ecosystem.
//
// ### `cargo frc set-team-number`
//    Sets the team number of the connected robot to the one of the current project.
//
// ### `cargo frc webservice`
//    Utilities for interacting with the robot's web service and a custom web service.

#[derive(Parser)]
#[command(bin_name = "frc", version, author, disable_help_subcommand = true)]
enum Commands {
    #[clap(name = "check")]
    Check(cmds::checker::Check),
    #[clap(name = "sim")]
    Sim,
    #[clap(name = "deploy")]
    Deploy,
    #[clap(name = "test")]
    Test(cmds::tester::Test),
    #[clap(name = "tui")]
    Tui,
    #[clap(name = "webservice")]
    Webservice,
    #[clap(name = "set-team-number")]
    SetTeamNumber,
}

cargo_subcommand_metadata::description!(
    "Manage building and deploying Rust code in the FRC ecosystem."
);

fn main() -> Result<(), RuntimeError> {
    let config = project_config::read_config2()?;
    // remove frc from args
    let mut args = std::env::args().collect::<Vec<String>>();
    if args.len() > 1 && args[1] == "frc" {
        args.remove(1);
    }
    let commands = Commands::parse_from(args);
    match commands {
        Commands::Check(check) => cmds::checker::cargo_check(check, config)?,
        Commands::Test(test) => cmds::tester::cargo_test(test, config)?,
        _ => {
            unimplemented!();
        }
    };

    Ok(())
}
