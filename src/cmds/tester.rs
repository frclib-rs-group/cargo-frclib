use std::{path::PathBuf, process::Command};

use clap::Parser;

use crate::{
    errors::TestSubcommandError,
    project_config::{self, ProjectConfig},
};

const PACKAGE_SELECTION: &str = "Package Selection";
const TARGET_SELECTION: &str = "Target Selection";
const FEATURE_SELECTION: &str = "Feature Selection";
const COMPILATION_OPTIONS: &str = "Compilation Options";
const MANIFEST_OPTIONS: &str = "Manifest Options";
const MISC_OPTIONS: &str = "Misc Options";
const COMMON_OPTIONS: &str = "Common Options";
const USAGE: &str = "Usage Options";
const TEST_OPTIONS: &str = "Test Options";
const DISPLAY_OPTIONS: &str = "Display Options";

#[derive(Parser)]
pub struct Test {
    //# USAGE
    #[arg(long, help_heading = USAGE)]
    pub no_dev: bool,

    //# TEST OPTIONS
    #[arg(long, help_heading = TEST_OPTIONS)]
    pub no_run: bool,

    #[arg(long, help_heading = TEST_OPTIONS)]
    pub no_fail_fast: bool,

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

    #[arg(long, help_heading = TARGET_SELECTION)]
    pub bins: bool,

    #[arg(long, value_name = "NAME", num_args = 0..=1, help_heading = TARGET_SELECTION)]
    pub example: Option<Option<String>>,

    #[arg(long, value_name = "NAME", num_args = 0..=1, help_heading = TARGET_SELECTION)]
    pub test: Option<Option<String>>,

    #[arg(long, help_heading = TARGET_SELECTION)]
    pub tests: bool,

    #[arg(long, value_name = "NAME", num_args = 0..=1, help_heading = TARGET_SELECTION)]
    pub bench: Option<Option<String>>,

    #[arg(long, help_heading = TARGET_SELECTION)]
    pub benches: bool,

    //# FEATURE SELECTION
    #[arg(short = 'F', long, value_name = "FEATURES", help_heading = FEATURE_SELECTION)]
    pub features: Option<String>,

    #[arg(long, help_heading = FEATURE_SELECTION)]
    pub all_features: bool,

    #[arg(long, help_heading = FEATURE_SELECTION)]
    pub no_default_features: bool,

    //# COMPILATION OPTIONS
    #[arg(long, value_name = "TARGET", help_heading = COMPILATION_OPTIONS)]
    pub target: Option<String>,

    #[arg(long, help_heading = COMPILATION_OPTIONS)]
    pub release: bool,

    #[arg(long, value_name = "PROFILE-NAME", help_heading = COMPILATION_OPTIONS)]
    pub profile: Option<String>,

    #[arg(long, help_heading = COMPILATION_OPTIONS)]
    pub all_targets: bool,

    #[arg(long, value_name = "DIRECTORY", help_heading = COMPILATION_OPTIONS)]
    pub target_dir: Option<PathBuf>,

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

    // Control when colored output is used
    #[arg(long, value_name = "WHEN", help_heading = DISPLAY_OPTIONS)]
    pub color: Option<String>,

    // The output format for diagnostic messages
    #[arg(long, value_name = "FORMAT", help_heading = DISPLAY_OPTIONS)]
    pub message_format: Option<String>,

    //# MISC OPTIONS
    /// Number of parallel jobs, defaults to # of CPUs
    #[arg(short, long, value_name = "N", help_heading = MISC_OPTIONS)]
    pub jobs: Option<u64>,

    #[arg(long, help_heading = MISC_OPTIONS)]
    pub future_incompat_report: bool,

    //capture trailing args
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub trailing: Vec<String>,
}

fn cmd_config(cmd: &mut Command, cfg: &str) {
    cmd.arg("--config");
    cmd.arg(format!("build.rustflags = [\"--cfg\", \"{}\"]", cfg));
}

pub fn cargo_test(test: Test, config: ProjectConfig) -> Result<(), TestSubcommandError> {
    let mut cmd = Command::new(std::env::var("CARGO").unwrap_or("cargo".into()));
    cmd.arg("test");

    cmd.env("FRC_TEAM_NUMBER", config.team.0.to_string());

    cmd_config(&mut cmd, project_config::DEFAULT_CFG);

    if !test.no_dev {
        cmd_config(&mut cmd, project_config::DEV_CFG);
        cmd.arg("--target-dir");
        cmd.arg(config.target_dirs.sim_dev);
    } else {
        cmd.arg("--target-dir");
        cmd.arg(config.target_dirs.sim);
    }

    cmd_config(&mut cmd, project_config::RUNTIME_SIM_CFG);

    if test.no_run {
        cmd.arg("--no-run");
    }

    if test.no_fail_fast {
        cmd.arg("--no-fail-fast");
    }

    if !test.config.is_empty() {
        cmd.arg("--config");
        cmd.arg(test.config.join(","));
    }

    if !test.unstable_flags.is_empty() {
        cmd.arg("-Z");
        cmd.arg(test.unstable_flags.join(","));
    }

    if let Some(dir) = test.change_dir {
        cmd.arg("-C");
        cmd.arg(dir);
    }

    if let Some(package) = test.package {
        cmd.arg("--package");
        if let Some(package) = package {
            cmd.arg(package);
        }
    }

    if test.workspace {
        cmd.arg("--workspace");
    }

    if !test.exclude.is_empty() {
        cmd.arg("--exclude");
        cmd.arg(test.exclude.join(","));
    }

    if test.lib {
        cmd.arg("--lib");
    }

    if let Some(bin) = test.bin {
        cmd.arg("--bin");
        if let Some(bin) = bin {
            cmd.arg(bin);
        }
    }

    if test.bins {
        cmd.arg("--bins");
    }

    if let Some(example) = test.example {
        cmd.arg("--example");
        if let Some(example) = example {
            cmd.arg(example);
        }
    }

    if let Some(test) = test.test {
        cmd.arg("--test");
        if let Some(test) = test {
            cmd.arg(test);
        }
    }

    if test.tests {
        cmd.arg("--tests");
    }

    if let Some(bench) = test.bench {
        cmd.arg("--bench");
        if let Some(bench) = bench {
            cmd.arg(bench);
        }
    }

    if test.benches {
        cmd.arg("--benches");
    }

    if let Some(features) = test.features {
        cmd.arg("-F");
        cmd.arg(features);
    }

    if test.all_features {
        cmd.arg("--all-features");
    }

    if test.no_default_features {
        cmd.arg("--no-default-features");
    }

    if let Some(target) = test.target {
        cmd.arg("--target");
        cmd.arg(target);
    }

    if test.release {
        cmd.arg("--release");
    }

    if let Some(profile) = test.profile {
        cmd.arg("--profile");
        cmd.arg(profile);
    }

    if test.all_targets {
        cmd.arg("--all-targets");
    }

    if let Some(dir) = test.target_dir {
        cmd.arg("--target-dir");
        cmd.arg(dir);
    }

    if let Some(path) = test.manifest_path {
        cmd.arg("--manifest-path");
        cmd.arg(path);
    }

    if test.frozen {
        cmd.arg("--frozen");
    }

    if test.locked {
        cmd.arg("--locked");
    }

    if test.offline {
        cmd.arg("--offline");
    }

    if test.verbose {
        cmd.arg("-v");
    }

    if test.quiet {
        cmd.arg("-q");
    }

    if let Some(color) = test.color {
        cmd.arg("--color");
        cmd.arg(color);
    }

    if let Some(message_format) = test.message_format {
        cmd.arg("--message-format");
        cmd.arg(message_format);
    }

    if let Some(jobs) = test.jobs {
        cmd.arg("-j");
        cmd.arg(jobs.to_string());
    }

    if test.future_incompat_report {
        cmd.arg("--future-incompat-report");
    }

    cmd.args(test.trailing);

    let exit_status = cmd
        .spawn()
        .map_err(|_| TestSubcommandError::FailedToSpawnCargoTest)?
        .wait()
        .map_err(|_| TestSubcommandError::FailedToWaitForCargoTest)?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(TestSubcommandError::FailedCargoTest {
            code: exit_status.code(),
        })
    }
}
