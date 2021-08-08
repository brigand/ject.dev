mod ssh;

use ssh::CheckedSsh;
use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    os::unix::prelude::OsStringExt,
    path::{Path, PathBuf},
    process::Command,
};

type DynError = Box<dyn std::error::Error>;

// impl is later in this file

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().skip(1).next();

    let mut only = false;
    for arg in env::args().skip(1) {
        if arg == "--only" {
            only = true;
        }
    }

    match task.as_deref() {
        Some("dist") => dist()?,
        Some("deploy") => {
            let both = CheckedSsh::both()?;
            if !only {
                dist()?;
            }
            deploy(both)?
        }
        Some("provision") => provision()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
dist                       builds application to target/dist/ject-server
provision                  sets up a server to be able to run ject.dev
deploy                     transfers cargo and webpack output to the server and restarts it
    --only: if passed, skip running dist first
"
    )
}

fn dist() -> Result<(), DynError> {
    let _ = fs::remove_dir_all(&dist_dir());
    fs::create_dir_all(&dist_dir())?;

    docker_build_musl()?;
    dist_binary()?;
    dist_webpack()?;
    dist_manpage()?;

    Ok(())
}

fn dist_binary() -> Result<(), DynError> {
    // let image = "ekidd/rust-musl-builder:nightly-2021-02-13";

    // let status = Command::new("docker").args(&["pull", image]).status()?;
    // if !status.success() {
    //     Err("docker pull failed")?;
    // }

    let status = docker_run_builder(&[
        "sudo",
        "chown",
        "-R",
        "rust:rust",
        "/home/rust/.cargo/git",
        "/home/rust/.cargo/registry",
    ])
    .status()?;
    if !status.success() {
        Err("failed to set ownership of .cargo/git and .cargo/registry")?;
    }
    println!("Updated permissions");
    let status = docker_run_builder(&["cargo", "build", "--release", "-p", "server"]).status()?;

    if !status.success() {
        Err("docker musl build failed")?;
    }

    let dst = project_root().join("target/x86_64-unknown-linux-musl/release/server");

    fs::copy(&dst, dist_dir().join("ject-server"))?;

    Ok(())
}

fn dist_manpage() -> Result<(), DynError> {
    Ok(())
}

fn dist_webpack() -> Result<(), DynError> {
    let status = Command::new("npm").args(&["run", "build"]).status().map_err(|err| format!("npm run build couldn't execute. Likely node/npm not being installed.\nSource: {:?}", err))?;

    if !status.success() {
        Err("npm run build returned a non-zero exit code")?;
    }

    Ok(())
}

fn deploy(ssh: ssh::Both) -> Result<(), DynError> {
    {
        let (host, mut cmd) = ssh.user.to_rsync();
        cmd.arg("target/dist/ject-server");
        let dest = format!("{}:/home/ject/app/", host);
        cmd.arg(&dest);
        println!("Transferring ject-server with command: {:?}", cmd);
        let status = cmd.status()?;
        if !status.success() {
            Err("Failed first rsync command")?;
        }
    }

    {
        let (host, mut cmd) = ssh.user.to_rsync();
        // TODO: support ssh_args like "user@host -i custom.pem"
        cmd.arg("--delete");
        cmd.arg("-a");
        cmd.arg("dist/");
        let dest = format!("{}:/home/ject/app/dist/", host);
        cmd.arg(&dest);
        println!("Transferring ject webpack output with command: {:?}", cmd);
        let status = cmd.status()?;
        if !status.success() {
            Err("Failed second rsync command")?;
        }
    }

    {
        let mut cmd = ssh.root.to_command();

        let bash_commands = vec![
            "systemctl restart ject",
            "echo 'Restarted. Waiting 3 seconds to read logs'",
            "sleep 3",
            "echo 'Last 50 logs:'",
            "journalctl -u ject.service -n 50 --no-pager",
            "echo 'SSH Done'",
        ]
        .join(" && ");
        cmd.arg(&bash_commands);
        let status = cmd.status()?;
        if !status.success() {
            Err("ssh command to ject-root failed")?;
        }
    }

    Ok(())
}

fn provision() -> Result<(), DynError> {
    let root_ssh = CheckedSsh::root()?;
    let mut cmd = root_ssh.to_command();

    let write_authorized_key = format!(
        r#"key="{}"; grep -F "$(printf '%s' "$key" | awk '{{print $2}}')" /home/ject/.ssh/authorized_keys || printf '%s\n' "$key" >> /home/ject/.ssh/authorized_keys"#,
        get_ssh_pub()?
    );

    let write_systemd_file = bash_write_file(
        "/etc/systemd/system/ject.service",
        r#"[Service]
User=ject
Group=ject
ExecStart=/home/ject/app/ject-server
WorkingDirectory=/home/ject/app
Environment=JECT_IS_PROD=1
Environment=JECT_DOMAIN_MAIN=ject.dev
Environment=JECT_DOMAIN_FRAME=ject.link

## AmbientCapabilities=CAP_NET_BIND_SERVICE
## SecureBits=keep-caps
"#,
    );

    let bash_commands = vec![
        // Create users
        "id -u ject >/dev/null 2>&1 || ( echo 'Creating group and user' ; groupadd ject ; useradd -g ject --home-dir /home/ject --shell /bin/bash ject )",

        // Install nginx/certbot
        "apt-get update",
        "apt-get install -y nginx",
        "snap install core",
        "snap refresh core",
        "snap install --classic certbot",
        "ln -s /snap/bin/certbot /usr/bin/certbot",

        // Setup ject user home dir
        "mkdir -p /home/ject/app/letsencrypt/{ssl,nonce}",
        "mkdir -p /home/ject/.ssh",
        "touch /home/ject/.ssh/authorized_keys",
        &write_authorized_key,
        "chown ject:ject -R /home/ject/{app,.ssh}",
        "chmod -R 600 /home/ject/app/letsencrypt/",
        &write_systemd_file,
        "systemctl daemon-reload",
        "echo 'All commands executed!'",
    ]
    .join(" && ");
    cmd.arg(&bash_commands);

    println!("Executing command: {:#?}", cmd);

    let status = cmd.status()?;
    if !status.success() {
        Err("The provision ssh command failed")?;
    }

    Ok(())
}

fn bash_write_file(file_name: &str, contents: &str) -> String {
    let fmt = contents
        .split('\n')
        .map(|_| "%s")
        .collect::<Vec<_>>()
        .join("\n");
    let mut cmd = format!("printf \"{}\" ", fmt);

    for line in contents.split('\n') {
        cmd.push_str("\'");
        cmd.push_str(line);

        cmd.push_str("\' ");
    }

    cmd.push_str(" > ");
    cmd.push_str(file_name);

    cmd
}

// static BUILDER_TAG: &str = "ject-musl-builder";
// static BUILDER_IMAGE: &str = BUILDER_TAG;
static BUILDER_IMAGE: &str = "brigand/rust-musl-builder";

fn docker_build_musl() -> Result<(), DynError> {
    return Ok(());
    // println!("Building ject/musl/Dockerfile with tag {}", BUILDER_TAG);
    // let status = docker_command()
    //     .current_dir(&musl_dir())
    //     .args(&["build", "-t", BUILDER_TAG, "."])
    //     .status()?;

    // if !status.success() {
    //     Err("building musl/Dockerfile failed")?;
    // }

    // Ok(())
}

fn docker_run_builder(command_args: &[&str]) -> Command {
    let volume_1 = format!("{}:/home/rust/src", project_root().display());
    let mut args = vec![
        "run",
        "--rm",
        "-it",
        "-v",
        &volume_1,
        "-v",
        "cargo-git:/home/rust/.cargo/git",
        "-v",
        "cargo-registry:/home/rust/.cargo/registry",
        BUILDER_IMAGE,
    ];
    args.extend(command_args.into_iter());
    let mut cmd = docker_command();
    cmd.args(args);
    cmd
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn project_relative(path: impl AsRef<Path>) -> PathBuf {
    project_root().join(path.as_ref())
}

fn dist_dir() -> PathBuf {
    project_relative("target/dist")
}

fn musl_dir() -> PathBuf {
    project_relative("musl")
}

// fn cargo_command() -> Command {
//     let path = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
//     let mut cmd = Command::new(path);
//     cmd.current_dir(project_root());
//     cmd
// }

fn docker_command() -> Command {
    let mut cmd = Command::new("docker");
    cmd.current_dir(project_root());
    cmd
}

fn get_home() -> Result<PathBuf, DynError> {
    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg("printf '%s' ~")
        .output()
        .map_err(|err| format!("[xtask/get_home] Unable to execute sh, error: {:?}", err))?;
    if !output.status.success() {
        eprintln!("[xtask/get_home] output: {:?}", output);
        Err("[xtask/get_home] shell command unsuccessful")?;
    }
    if output.stdout.is_empty() {
        Err("[xtask/get_home] shell command returned no output")?;
    }
    if output.stdout.len() == 1 {
        Err("[xtask/get_home] shell command returned a single byte (too short)")?;
    }

    Ok(PathBuf::from(OsString::from_vec(output.stdout)))
}

fn get_ssh_pub() -> Result<String, DynError> {
    let key = std::fs::read_to_string(get_home()?.join(".ssh/id_rsa.pub"))?;
    Ok(key.trim().split('\n').next().unwrap().trim().to_string())
}
