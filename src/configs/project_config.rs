use std::{collections::HashSet, net::Ipv4Addr, path::PathBuf};

use crate::errors::ProjectConfigError;

pub const RUNTIME_REAL_CFG: &str = "frc_real";
pub const RUNTIME_SIM_CFG: &str = "frc_sim";
pub const DEV_CFG: &str = "frc_dev";
pub const DEFAULT_CFG: &str = "frc";

#[derive(Debug)]
pub struct TeamNumber(pub u16);

#[derive(Debug)]
pub struct Robot {
    pub name: String,
    pub serials: HashSet<String>,
}

#[derive(Debug)]
pub struct TargetDirs {
    pub real_dev: PathBuf,
    pub real: PathBuf,
    pub sim_dev: PathBuf,
    pub sim: PathBuf,
}
impl TargetDirs {
    fn from_rel_target_dir(rel: PathBuf) -> Self {
        Self {
            real_dev: rel.join("real-dev"),
            real: rel.join("real"),
            sim_dev: rel.join("sim-dev"),
            sim: rel.join("sim"),
        }
    }
}

#[derive(Debug)]
pub enum Runtimes {
    Real,
    Sim,
}

#[derive(Debug)]
pub struct ProjectConfig {
    pub team: TeamNumber,
    pub robots: Vec<Robot>,
    pub address: Option<Ipv4Addr>,
    pub deploy_dir: Option<String>,
    pub default_check: Runtimes,
    pub target_dirs: TargetDirs,
}

pub fn read_config2() -> Result<ProjectConfig, ProjectConfigError> {
    let raw = cargo_metadata::MetadataCommand::new().exec()?;

    let target_dirs = TargetDirs::from_rel_target_dir(raw.target_directory.clone().into());

    let package = raw
        .root_package()
        .ok_or(ProjectConfigError::MissingRootPackage)?;

    let frc_cfg = package
        .metadata
        .get("frc")
        .ok_or(ProjectConfigError::MissingFrcMetadata)?;

    let team = TeamNumber({
        const TEAM_HINT: &str =
            "a number or a string that can be parsed as a number thats less than 65536";
        const TEAM: &str = "team";
        let attr = frc_cfg
            .get("team")
            .ok_or(ProjectConfigError::MissingAttribute(TEAM))?;
        match attr {
            serde_json::Value::Number(i) => u16::try_from(
                i.as_u64()
                    .ok_or(ProjectConfigError::ParseAttribute(TEAM, TEAM_HINT))?,
            )
            .map_err(|_| ProjectConfigError::ParseAttribute(TEAM, TEAM_HINT))?,
            serde_json::Value::String(s) => s
                .parse::<u16>()
                .map_err(|_| ProjectConfigError::ParseAttribute(TEAM, TEAM_HINT))?,
            _ => return Err(ProjectConfigError::ParseAttribute(TEAM, TEAM_HINT)),
        }
    });

    let robots = {
        const ROBOTS_HINT: &str = "an array of objects with a name and serials attribute";
        const ROBOTS_NAME_HINT: &str = "a string";
        const ROBOTS_SERIALS_HINT: &str = "an array of strings";
        const ROBOTS: &str = "robots";
        const ROBOTS_NAME: &str = "robots[i].name";
        const ROBOTS_SERIALS: &str = "robots[i].serials";
        frc_cfg
            .get("robots")
            .ok_or(ProjectConfigError::MissingAttribute(ROBOTS))?
            .as_array()
            .ok_or(ProjectConfigError::ParseAttribute(ROBOTS, ROBOTS_HINT))?
            .iter()
            .map(|robot| {
                let name = robot
                    .get("name")
                    .ok_or(ProjectConfigError::MissingAttribute(ROBOTS_NAME))?
                    .as_str()
                    .ok_or(ProjectConfigError::ParseAttribute(
                        ROBOTS_NAME,
                        ROBOTS_NAME_HINT,
                    ))?
                    .to_owned();
                let serials = robot
                    .get("serials")
                    .ok_or(ProjectConfigError::MissingAttribute(ROBOTS_SERIALS))?
                    .as_array()
                    .ok_or(ProjectConfigError::ParseAttribute(
                        ROBOTS_SERIALS,
                        ROBOTS_SERIALS_HINT,
                    ))?
                    .iter()
                    .map(|serial| {
                        serial
                            .as_str()
                            .ok_or(ProjectConfigError::ParseAttribute(
                                ROBOTS_SERIALS,
                                ROBOTS_SERIALS_HINT,
                            ))
                            .map(|s| s.to_owned())
                    })
                    .collect::<Result<Vec<String>, ProjectConfigError>>()?
                    .into_iter()
                    .collect::<HashSet<String>>();
                Ok(Robot { name, serials })
            })
            .collect::<Result<Vec<Robot>, ProjectConfigError>>()?
    };

    let address = {
        const OVERRIDE_ADDRESS_HINT: &str = "a string that can be parsed as an ipv4 address";
        const OVERRIDE_ADDRESS: &str = "override-address";
        frc_cfg
            .get(OVERRIDE_ADDRESS)
            .map(|addr| {
                addr.as_str()
                    .ok_or(ProjectConfigError::ParseAttribute(
                        OVERRIDE_ADDRESS,
                        OVERRIDE_ADDRESS_HINT,
                    ))
                    .and_then(|s| {
                        s.parse::<Ipv4Addr>().map_err(|_| {
                            ProjectConfigError::ParseAttribute(
                                OVERRIDE_ADDRESS,
                                OVERRIDE_ADDRESS_HINT,
                            )
                        })
                    })
            })
            .transpose()?
    };

    let deploy_dir = {
        const DEPLOY_DIR_HINT: &str = "a string that can be used as a unix path, will be used relative to the deploy user's home directory";
        const DEPLOY_DIR: &str = "deploy-dir";
        frc_cfg
            .get(DEPLOY_DIR)
            .map(|dir| {
                dir.as_str()
                    .ok_or(ProjectConfigError::ParseAttribute(
                        DEPLOY_DIR,
                        DEPLOY_DIR_HINT,
                    ))
                    .map(|s| s.to_owned())
            })
            .transpose()?
    };

    let default_check = {
        const DEFAULT_CHECK_HINT: &str = "a string that is either \"real\" or \"sim\"";
        const DEFAULT_CHECK: &str = "default-check";
        match frc_cfg
            .get(DEFAULT_CHECK)
            .ok_or(ProjectConfigError::MissingAttribute(DEFAULT_CHECK))?
            .as_str()
            .ok_or(ProjectConfigError::ParseAttribute(
                DEFAULT_CHECK,
                DEFAULT_CHECK_HINT,
            ))? {
            "real" => Runtimes::Real,
            "sim" => Runtimes::Sim,
            _ => {
                return Err(ProjectConfigError::ParseAttribute(
                    DEFAULT_CHECK,
                    DEFAULT_CHECK_HINT,
                ))
            }
        }
    };

    Ok(ProjectConfig {
        team,
        robots,
        address,
        deploy_dir,
        default_check,
        target_dirs,
    })
}
