use std::{
    env, fs,
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
    match task.as_ref().map(|it| it.as_str()) {
        Some("dist") => dist()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
dist            builds application and man pages
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

static BUILDER_TAG: &str = "ject-musl-builder";
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
