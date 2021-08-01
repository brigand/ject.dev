use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    os::unix::prelude::OsStringExt,
    path::{Path, PathBuf},
    process::Command,
};

type DynError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SshAs {
    User,
    Root,
}

impl SshAs {
    /// Returns the initial args for an SSH command with the selected user.
    ///
    /// For SshAs::User: JECT_SSH_USER defaulting to "ject" (without quotes)
    ///
    /// For SshAs::Root: JECT_SSH_ROOT defaulting to "ject-root" (without quotes)
    pub fn get_ssh_args(self) -> Vec<String> {
        let (default_value, env_key) = match self {
            Self::User => ("ject", "JECT_SSH_USER"),
            Self::Root => ("ject-root", "JECT_SSH_ROOT"),
        };

        let split = |words: &str| {
            words
                .split_whitespace()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_owned())
                .collect()
        };

        match env::var(env_key) {
            Ok(arg) if !arg.trim().is_empty() => split(&arg),
            _ => split(default_value),
        }
    }

    pub fn to_command_minimal(self) -> Command {
        let mut cmd = Command::new("ssh");
        cmd.args(self.get_ssh_args());
        cmd
    }
    pub fn to_command(self) -> Command {
        let mut cmd = self.to_command_minimal();
        cmd.arg("-o");
        cmd.arg("ConnectTimeout=60"); // in seconds
        cmd.arg("-o");
        cmd.arg("BatchMode=yes");

        cmd
    }

    pub fn to_rsync(self) -> (String, Command) {
        let mut cmd = Command::new("rsync");
        cmd.arg("-Pe");
        let mut ssh_args = self.get_ssh_args().into_iter();
        let host = ssh_args.next().unwrap();
        let rest: Vec<_> = ssh_args.collect();
        cmd.arg(format!("ssh {}", rest.join(" ")));

        (host, cmd)
    }

    pub fn name(self) -> &'static str {
        match self {
            SshAs::User => "User",
            SshAs::Root => "Root",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Both {
    pub user: CheckedSsh,
    pub root: CheckedSsh,
}

#[derive(Debug, Clone)]
pub struct CheckedSsh {
    ssh_as: SshAs,
}

impl CheckedSsh {
    pub fn both() -> Result<Both, DynError> {
        // TODO: run concurrently
        let user = Self::user()?;
        let root = Self::root()?;
        Ok(Both { user, root })
    }

    pub fn user() -> Result<Self, DynError> {
        Self::acquire(SshAs::User)
    }

    pub fn root() -> Result<Self, DynError> {
        Self::acquire(SshAs::Root)
    }

    pub fn acquire(ssh_as: SshAs) -> Result<Self, DynError> {
        let mut cmd = ssh_as.to_command();

        cmd.arg(format!(
            r"echo 'Checking connection for SshAs::{}'",
            ssh_as.name()
        ));
        let status = cmd.status()?;

        if !status.success() {
            let message = format!("Command {:?} failed with status {:?}", cmd, status);
            return Err(message.into());
        }

        Ok(Self { ssh_as })
    }

    pub fn to_command(&self) -> Command {
        self.ssh_as.to_command()
    }

    /// Returns (host, rsync cmd).
    pub fn to_rsync(&self) -> (String, Command) {
        self.ssh_as.to_rsync()
    }
}
