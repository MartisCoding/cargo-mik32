use std::{env::current_dir, fs};

use std::process::Command;
use log::{error, info};

static MIK32_UPLOADER_GIT_URL: &str = "https://github.com/MikronMIK32/mik32-uploader.git";

#[derive(Debug)]
pub enum InitError {
    FsFailed,
    FetchFailed,
}

pub fn init_script(name: String) -> Result<(), InitError> {
    
    info!("Initializing project: {}", name);

    let current_dir = current_dir().expect("Failed to get current directory.");
    let project_dir = current_dir.join(&name);
    fs::create_dir_all(project_dir.join("src")).expect("Failed to create src dir");
    fs::create_dir_all(project_dir.join("flash")).expect("Failed to create flash dir");
    fs::create_dir_all(project_dir.join(".cargo")).expect("Failed to create .cargo dir");

    info!("Fetching external dependencies");

    let mut git_cmd = Command::new("git");
    git_cmd.args([
            "clone",
            "--depth=1",
            MIK32_UPLOADER_GIT_URL,
            project_dir.join("flash").join("mik32-uploader").to_str().unwrap()
    ]);
    let gitstat = git_cmd.status().expect("Unable to run git");

    if !gitstat.success() {
        error!("Failed to fetch external dependencies.");
        return Err(InitError::FetchFailed);
    }

    let cargo = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
mik32v2-pac = {{ git = "https://github.com/mik32-rs/mik32v2-pac.git" }}
mik32-rt = {{ git = "https://github.com/mik32-rs/mik32-rt.git" }}
riscv = {{ version = "*", features = ["critical-section-single-hart"] }}


[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
"#, name=name
    );

    fs::write(project_dir.join("Cargo.toml"), cargo).unwrap();
    fs::write(project_dir.join("src/main.rs"), src_main).unwrap();
    fs::write(project_dir.join(".cargo/config.toml"), cargo_config).unwrap();
    Ok(())
}


const src_main: &str = 
r#"#![no_std]
#![no_main]

use mik32_hal::*;

#[mik32_rt::entry]
fn main() -> ! {
    loop {}
}
"#;

const cargo_config: &str = 
r#"[alias]
run = "mik32 run"
"#;

