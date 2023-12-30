use std::{path::PathBuf, process::Command};

use clap::Parser;

use crate::{
    errors::CheckSubcommandError,
    project_config::{self, ProjectConfig, Runtimes},
};

const PACKAGE_SELECTION: &str = "Package Selection";
const TARGET_SELECTION: &str = "Target Selection";
const FEATURE_SELECTION: &str = "Feature Selection";
const COMPILATION_OPTIONS: &str = "Compilation Options";
const MANIFEST_OPTIONS: &str = "Manifest Options";
const MISC_OPTIONS: &str = "Misc Options";
const COMMON_OPTIONS: &str = "Common Options";
const USAGE: &str = "Usage Options";
const DISPLAY_OPTIONS: &str = "Display Options";

#[derive(Parser)]
pub struct Check {
    //# USAGE
    #[arg(long, conflicts_with = "real", help_heading = USAGE)]
    pub sim: bool,

    #[arg(long, conflicts_with = "sim", help_heading = USAGE)]
    pub real: bool,

    #[arg(short, long, help_heading = USAGE)]
    pub dev: bool,

    //# COMMON OPTIONS
    #[arg(long, value_name = "KEY=VALUE", help_heading = COMMON_OPTIONS)]
    pub config: Vec<String>,

    #[arg(short = 'Z', value_name = "FLAG", help_heading = COMMON_OPTIONS)]
    pub unstable_flags: Vec<String>,

    #[arg(short = 'C', long, value_name = "PATH", help_heading = COMMON_OPTIONS)]
    pub change_dir: Option<PathBuf>,

    //# PACKAGE SELECTION
    #[arg(short, long, value_name = "SPEC", num_args = 0..=1, help_heading = PACKAGE_SELECTION)]
    pub package: Option<Option<String>>,

    #[arg(long, help_heading = PACKAGE_SELECTION)]
    pub workspace: bool,

    #[arg(long, value_name = "SPEC", number_of_values = 1, help_heading = PACKAGE_SELECTION)]
    pub exclude: Vec<String>,

    //# TARGET SELECTION
    #[arg(long, help_heading = TARGET_SELECTION)]
    pub lib: bool,

    #[arg(long, value_name = "NAME", num_args = 0..=1, help_heading = TARGET_SELECTION)]
    pub bin: Option<Option<String>>,

    #[arg(long, value_name = "NAME", num_args = 0..=1, help_heading = TARGET_SELECTION)]
    pub example: Option<Option<String>>,

    #[arg(long, value_name = "NAME", num_args = 0..=1, help_heading = TARGET_SELECTION)]
    pub test: Option<Option<String>>,

    #[arg(long, help_heading = TARGET_SELECTION)]
    pub tests: bool,

    #[arg(long, value_name = "NAME", num_args = 0..=1, help_heading = TARGET_SELECTION)]
    pub bench: Option<Option<String>>,

    //# FEATURE SELECTION
    #[arg(short = 'F', long, value_name = "FEATURES", help_heading = FEATURE_SELECTION)]
    pub features: Option<String>,

    #[arg(long, help_heading = FEATURE_SELECTION)]
    pub all_features: bool,

    #[arg(long, help_heading = FEATURE_SELECTION)]
    pub no_default_features: bool,

    //# COMPILATION OPTIONS
    #[arg(long, help_heading = COMPILATION_OPTIONS)]
    pub release: bool,

    #[arg(long, value_name = "PROFILE-NAME", help_heading = COMPILATION_OPTIONS)]
    pub profile: Option<String>,

    #[arg(long, value_name = "TARGET", help_heading = COMPILATION_OPTIONS)]
    pub target: Option<String>,

    #[arg(long, help_heading = COMPILATION_OPTIONS)]
    pub all_targets: bool,

    //# MANIFEST OPTIONS
    #[arg(long, value_name = "PATH", help_heading = MANIFEST_OPTIONS)]
    pub manifest_path: Option<PathBuf>,

    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub frozen: bool,

    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub locked: bool,

    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub offline: bool,

    //# DISPLAY OPTIONS
    #[arg(short, long, help_heading = DISPLAY_OPTIONS)]
    pub verbose: bool,

    #[arg(short, long, help_heading = DISPLAY_OPTIONS)]
    pub quiet: bool,

    #[arg(long, value_name = "WHEN", help_heading = DISPLAY_OPTIONS)]
    pub color: Option<String>,

    #[arg(long, value_name = "FORMAT", help_heading = DISPLAY_OPTIONS)]
    pub message_format: Option<String>,

    //# MISC OPTIONS
    #[arg(short, long, value_name = "N", help_heading = MISC_OPTIONS)]
    pub jobs: Option<u64>,

    #[arg(long, help_heading = MISC_OPTIONS)]
    pub keep_going: bool,

    #[arg(long, help_heading = MISC_OPTIONS)]
    pub future_incompat_report: bool,
}

fn cmd_config(cmd: &mut Command, cfg: &str) {
    cmd.arg("--config");
    cmd.arg(format!("build.rustflags = [\"--cfg\", \"{}\"]", cfg));
}

#[track_caller]
pub fn cargo_check(check: Check, config: ProjectConfig) -> Result<(), CheckSubcommandError> {
    let mut cmd = Command::new(std::env::var("CARGO").unwrap_or("cargo".into()));
    cmd.arg("clippy");

    cmd.env("FRC_TEAM_NUMBER", config.team.0.to_string());

    cmd_config(&mut cmd, project_config::DEFAULT_CFG);

    if check.dev {
        cmd_config(&mut cmd, project_config::DEV_CFG);
    };

    let mode: Runtimes;
    if check.sim {
        mode = Runtimes::Sim;
    } else if check.real {
        mode = Runtimes::Real;
    } else {
        mode = config.default_check;
    }

    match mode {
        Runtimes::Real => {
            cmd_config(&mut cmd, project_config::RUNTIME_REAL_CFG);
            if check.dev {
                cmd.arg("--target-dir");
                cmd.arg(config.target_dirs.real_dev);
            } else {
                cmd.arg("--target-dir");
                cmd.arg(config.target_dirs.real);
            }
        }
        Runtimes::Sim => {
            cmd_config(&mut cmd, project_config::RUNTIME_SIM_CFG);
            if check.dev {
                cmd.arg("--target-dir");
                cmd.arg(config.target_dirs.sim_dev);
            } else {
                cmd.arg("--target-dir");
                cmd.arg(config.target_dirs.sim);
            }
        }
    };

    if !check.config.is_empty() {
        cmd.arg("--config");
        check.config.iter().for_each(|s| {
            cmd.arg(s);
        });
    }

    if !check.unstable_flags.is_empty() {
        cmd.arg("-Z");
        check.unstable_flags.iter().for_each(|s| {
            cmd.arg(s);
        });
    }

    if let Some(dir) = check.change_dir {
        cmd.arg("-C");
        cmd.arg(dir);
    }

    if let Some(package) = check.package {
        cmd.arg("--package");
        if let Some(package) = package {
            cmd.arg(package);
        }
    }

    if check.workspace {
        cmd.arg("--workspace");
    }

    if !check.exclude.is_empty() {
        cmd.arg("--exclude");
        check.exclude.iter().for_each(|s| {
            cmd.arg(s);
        });
    }

    if check.lib {
        cmd.arg("--lib");
    }

    if let Some(bin) = check.bin {
        cmd.arg("--bin");
        if let Some(bin) = bin {
            cmd.arg(bin);
        }
    }

    if let Some(example) = check.example {
        cmd.arg("--example");
        if let Some(example) = example {
            cmd.arg(example);
        }
    }

    if let Some(test) = check.test {
        cmd.arg("--test");
        if let Some(test) = test {
            cmd.arg(test);
        }
    }

    if check.tests {
        cmd.arg("--tests");
    }

    if let Some(bench) = check.bench {
        cmd.arg("--bench");
        if let Some(bench) = bench {
            cmd.arg(bench);
        }
    }

    if let Some(features) = check.features {
        cmd.arg("-F");
        cmd.arg(features);
    }

    if check.all_features {
        cmd.arg("--all-features");
    }

    if check.no_default_features {
        cmd.arg("--no-default-features");
    }

    if check.release {
        cmd.arg("--release");
    }

    if let Some(profile) = check.profile {
        cmd.arg("--profile");
        cmd.arg(profile);
    }

    if let Some(target) = check.target {
        cmd.arg("--target");
        cmd.arg(target);
    }

    if check.all_targets {
        cmd.arg("--all-targets");
    }

    if let Some(path) = check.manifest_path {
        cmd.arg("--manifest-path");
        cmd.arg(path);
    }

    if check.frozen {
        cmd.arg("--frozen");
    }

    if check.locked {
        cmd.arg("--locked");
    }

    if check.offline {
        cmd.arg("--offline");
    }

    if check.verbose {
        cmd.arg("-v");
    }

    if check.quiet {
        cmd.arg("--quiet");
    }

    if let Some(color) = check.color {
        cmd.arg("--color");
        cmd.arg(color);
    }

    if let Some(format) = check.message_format {
        cmd.arg("--message-format");
        cmd.arg(format);
    }

    if let Some(jobs) = check.jobs {
        cmd.arg("-j");
        cmd.arg(jobs.to_string());
    }

    if check.keep_going {
        cmd.arg("--keep-going");
    }

    if check.future_incompat_report {
        cmd.arg("--future-incompat-report");
    }

    tracing::debug!("{:?}", cmd);

    let exit_status = cmd
        .spawn()
        .map_err(|_| CheckSubcommandError::FailedToSpawnCargoCheck)?
        .wait()
        .map_err(|_| CheckSubcommandError::FailedToWaitForCargoCheck)?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(CheckSubcommandError::FailedCargoCheck {
            code: exit_status.code(),
        })
    }
}
