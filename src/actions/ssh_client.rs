use std::{sync::Arc, net::{Ipv4Addr, SocketAddrV4}, collections::HashMap, io};

use tracing::{error, info};
use russh::{*, client::Handle};
use russh_sftp::{client::SftpSession, protocol::FileType};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::configs::project_config::ProjectConfig;

use super::deploy_descriptor::Descriptor;

pub type Path = relative_path::RelativePath;
pub type PathBuf = relative_path::RelativePathBuf;
pub trait PathType: AsRef<Path> + Send {}
impl<T: AsRef<Path> + Send + ?Sized> PathType for T {}

#[derive(Debug, thiserror::Error)]
pub enum SSHErrors {
    #[error("{0}")]
    SSHError(#[from] russh::Error),
    #[error("{0}")]
    SFTPError(#[from] russh_sftp::client::error::Error),
    #[error("{0}")]
    IOError(#[from] io::Error),
}

struct SSHClientImpl;
impl client::Handler for SSHClientImpl {
    type Error = russh::Error;
}

pub struct SSHConnection {
    addr: SocketAddrV4,
    ssh: Handle<SSHClientImpl>,
    sftp: SftpSession,
    file_buffers: HashMap<String, Vec<u8>>
}

impl SSHConnection {
    pub async fn call(&mut self, command: &str) -> Result<u32, SSHErrors> {
        let mut channel = self.ssh.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut code = 0;
        let mut stdout = tokio::io::stdout();

        loop {
            // There's an event available on the session channel
            let Some(msg) = channel.wait().await else {
                break;
            };
            match msg {
                // Write data to the terminal
                ChannelMsg::Data { ref data } => {
                    stdout.write_all(data).await?;
                    stdout.flush().await?;
                }
                // The command has returned an exit code
                ChannelMsg::ExitStatus { exit_status } => {
                    code = exit_status;
                    channel.eof().await?;
                    break;
                }
                _ => {}
            }
        }
        Ok(code)
    }

    pub async fn upload_file(&mut self, path: impl PathType, data: &[u8]) -> Result<(), SSHErrors> {
        let mut file = self.sftp.create(path.as_ref().to_string()).await?;
        file.write_all(data).await?;
        file.shutdown().await?;
        Ok(())
    }

    pub async fn download_file(&mut self, path: impl PathType) -> Result<Vec<u8>, SSHErrors> {
        let mut file = self.sftp.open(path.as_ref().to_string()).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    pub async fn list_dir(&mut self, path: impl PathType) -> Result<Vec<String>, SSHErrors> {
        let mut dir = self.sftp.read_dir(path.as_ref().to_string()).await?;
        let mut files = Vec::new();
        loop {
            let entry = dir.next();
            if entry.is_none() {
                break;
            }
            let entry = entry.unwrap();
            files.push(entry.file_name());
        }
        Ok(files)
    }

    pub async fn get_file_type(&mut self, path: impl PathType) -> Result<FileType, SSHErrors> {
        let metadata = self.sftp.metadata(path.as_ref().to_string()).await?;
        Ok(metadata.file_type())
    }

    pub async fn get_file_size(&mut self, path: impl PathType) -> Result<u64, SSHErrors> {
        let metadata = self.sftp.metadata(path.as_ref().to_string()).await?;
        Ok(metadata.len())
    }
}

pub async fn connect_ssh_client(config: ProjectConfig, descriptor: Descriptor) -> Result<SSHConnection, SSHErrors> {
    let ipv4 = if let Some(ipv4) = config.address {
        ipv4
    } else {
        let team = config.team.0;
        Ipv4Addr::new(10, team as u8 / 100, team as u8 % 100, 2)
    };

    let ssh_config = russh::client::Config::default();
    let ssh_client = SSHClientImpl {};

    info!("Connecting to {}...", ipv4);

    let mut ssh_session = russh::client::connect(
        Arc::new(ssh_config),
        (ipv4, 22),
        ssh_client
    ).await?;

    info!("Connected to {}", ipv4);

    if !ssh_session.authenticate_password(descriptor.root_user, descriptor.root_password).await? {
        error!("Failed to authenticate as root");
        ssh_session.disconnect(
            Disconnect::AuthCancelledByUser,
            "Failed to authenticate as root",
            "English"
        ).await?;
        return Err(russh::Error::NotAuthenticated.into());
    }

    info!("Authenticated as root, starting SFTP session");

    let channel = ssh_session.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    info!("SFTP opened at {:?}", sftp.canonicalize(".").await?);

    Ok(
        SSHConnection {
            addr: SocketAddrV4::new(ipv4, 22),
            ssh: ssh_session,
            sftp,
            file_buffers: HashMap::new()
        }
    )
}