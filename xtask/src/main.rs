use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    os::unix::prelude::OsStringExt,
    path::{Path, PathBuf},
    process::Command,
};

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    let arg1 = env::args().nth(2);
    match task.as_deref() {
        Some("dist") => dist()?,
        Some("provision") => provision(arg1.as_deref())?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
dist                       builds application to target/dist/ject-server
provision [ssh_args]       sets up a server to be able to run ject.dev
    ssh_args: space-delimited list of initial ssh arguments
              defaults to \"ject-root\" (without quotes)
              define a ject-root host in ~/.ssh/config to have the default work
"
    )
}

fn dist() -> Result<(), DynError> {
    let _ = fs::remove_dir_all(&dist_dir());
    fs::create_dir_all(&dist_dir())?;

    docker_build_musl()?;
    dist_binary()?;
    dist_manpage()?;

    Ok(())
}

fn provision(ssh_args: Option<&str>) -> Result<(), DynError> {
    let ssh_args = ssh_args.unwrap_or("ject-root");
    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args.split_whitespace());
    // cmd.arg("bash");
    // cmd.arg("-c");

    let write_authorized_key = format!(
        r#"key="{}"; grep -F "$(printf '%s' "$key" | awk '{{print $2}}')" /home/ject/.ssh/authorized_keys || printf '%s\n' "$key" >> /home/ject/.ssh/authorized_keys"#,
        get_ssh_pub()?
    );
    let bash_commands = vec![
        "id -u ject >/dev/null 2>&1 || ( echo 'Creating group and user' ; groupadd ject ; useradd -g ject --home-dir /home/ject --shell /bin/bash ject )",
        "mkdir -p /home/ject/app/letsencrypt/{ssl,nonce}",
        "mkdir -p /home/ject/.ssh",
        "touch /home/ject/.ssh/authorized_keys",
        &write_authorized_key,
        "chown ject -R /home/ject/{app,.ssh}",
        "chmod -R 600 /home/ject/app/letsencrypt/",
        "echo 'End of bash commands!'",
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

fn dist_binary() -> Result<(), DynError> {
    // let cargo = ;

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

    // if Command::new("strip")
    //     .arg("--version")
    //     .stdout(Stdio::null())
    //     .status()
    //     .is_ok()
    // {
    //     eprintln!("stripping the binary");
    //     let status = Command::new("strip").arg(&dst).status()?;
    //     if !status.success() {
    //         Err("strip failed")?;
    //     }
    // } else {
    //     eprintln!("no `strip` utility found")
    // }

    Ok(())
}

fn dist_manpage() -> Result<(), DynError> {
    // let page = Manual::new("hello-world")
    //     .about("Greets the world")
    //     .render();
    // fs::write(dist_dir().join("hello-world.man"), &page.to_string())?;
    Ok(())
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
