use thiserror::Error;

#[derive(Debug, Error, Clone, Copy)]
pub enum RuntimeError {
    #[error("`frc check` failed: {0:?}")]
    Check(#[from] CheckSubcommandError),
    #[error("`frc deploy` failed: {0:?}")]
    Deploy(#[from] DeploySubcommandError),
    #[error("`frc sim` failed: {0:?}")]
    Sim(#[from] SimSubcommandError),
    #[error("`frc test` failed: {0:?}")]
    Test(#[from] TestSubcommandError),
    #[error("`frc tui` failed: {0:?}")]
    Tui(#[from] TuiSubcommandError),
    #[error("`frc webservice` failed: {0:?}")]
    Webservice(#[from] WebserviceSubcommandError),
    #[error("`frc set-team-number` failed: {0:?}")]
    SetTeamNumber(#[from] SetTeamNumberSubcommandError),
    #[error("Failed to read config: {0:?}")]
    Config(#[from] ProjectConfigError),
}

#[derive(Debug, Error, Clone, Copy)]
pub enum CheckSubcommandError {
    #[error("Failed to spawn `cargo check`")]
    FailedToSpawnCargoCheck,
    #[error("Failed to wait for `cargo check`")]
    FailedToWaitForCargoCheck,
    #[error("Failed to run `cargo check`: error {code:?}")]
    FailedCargoCheck { code: Option<i32> },
}

#[derive(Debug, Error, Clone, Copy)]
pub enum DeploySubcommandError {}

#[derive(Debug, Error, Clone, Copy)]
pub enum SimSubcommandError {}

#[derive(Debug, Error, Clone, Copy)]
pub enum TestSubcommandError {
    #[error("Failed to spawn `cargo test`")]
    FailedToSpawnCargoTest,
    #[error("Failed to wait for `cargo test`")]
    FailedToWaitForCargoTest,
    #[error("Failed to run `cargo test`: error {code:?}")]
    FailedCargoTest { code: Option<i32> },
}

#[derive(Debug, Error, Clone, Copy)]
pub enum TuiSubcommandError {}

#[derive(Debug, Error, Clone, Copy)]
pub enum WebserviceSubcommandError {}

#[derive(Debug, Error, Clone, Copy)]
pub enum SetTeamNumberSubcommandError {}

#[derive(Debug, Error, Clone, Copy)]
pub enum ProjectConfigError {
    #[error("Error reading cargo metadata")]
    CargoMetadata,
    #[error("Failed to get root package")]
    MissingRootPackage,
    #[error("[package.metadata.frc] not found in cargo metadata")]
    MissingFrcMetadata,
    #[error("Failed to get attribute {0:?} from [package.metadata.frc]")]
    MissingAttribute(&'static str),
    #[error("Failed to parse attribute {0:?} from [package.metadata.frc], should be {1:?}")]
    ParseAttribute(&'static str, &'static str),
}
impl From<cargo_metadata::Error> for ProjectConfigError {
    fn from(_: cargo_metadata::Error) -> Self {
        Self::CargoMetadata
    }
}
