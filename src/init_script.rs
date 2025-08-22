use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::fs;

use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum InitError {
    BadName,
    FetchFailed,
}

pub fn make_project(name: String, project_dir: PathBuf) -> Result<(), InitError>{
    
    let project_dir = project_dir.join(name.clone());

    let message = format!("Making project {}... ", name);
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::style::ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    

    if project_dir.exists() {
        eprintln!("Project with same name exists. Make sure to name it differently.");
        return Err(InitError::BadName);
    }

    pb.set_message(message.clone() + "Creating structure");
    pb.enable_steady_tick(Duration::from_millis(100));

    fs::create_dir_all(project_dir.join("src")).expect("Failed to make src directory in the project.");
    fs::create_dir_all(project_dir.join(".cargo")).expect("Failed to make .cargo directory in the project.");
    fs::create_dir_all(project_dir.join("flash")).expect("Failed to create flash directory in the project.");
    let mut cargo_toml = fs::File::create(project_dir.join("Cargo.toml")).expect("Failed to create Cargo.toml in the project.");
    let mut cargo_config = fs::File::create(project_dir.join(".cargo").join("config.toml")).expect("Failed to create ./.cargo/config.toml in the project.");
    let mut main_rs = fs::File::create(project_dir.join("src").join("main.rs")).expect("Failed to create ./src/main.rs in the project.");

    let _ = cargo_toml.write_all(format!(
        r#"
        [package]
        name = "{name}"
        version = "0.1.0"
        edition = "2021"

        [dependencies]

        [profile.release]
        opt-level = "z"
        lto = true
        codegen-units = 1
        "# 
        ).as_bytes()
    ).unwrap();

    let _ = cargo_config.write_all(r#"
        [target.riscv32imc-unknown-none-elf]
        rustflags = ["-C", "link-arg=-Tlink.x"]

        [build]
        target = "riscv32imc-unknown-none-elf"
        "#.as_bytes()
    );

    let _ = main_rs.write_all(r#"
        #![no_std]
        #![no_main]

        use mik32_hal::*;

        #[mik32_rt::entry]
        fn main() -> ! {
            loop {}
        }
        "#.as_bytes()
    );      

    pb.set_message(message.clone() + "Adding dependencies");
    cargo_add(&project_dir, "https://github.com/mik32-rs/mik32-hal.git".to_owned(), true)?;
    cargo_add(&project_dir, "https://github.com/mik32-rs/mik32-rt.git".to_owned(), true)?;

    pb.set_message(message.clone() + "Verifying paths");

    match env::var("MIK32_UPLOADER_PATH") {
        Ok(_) => (),
        Err(_) => {
            eprintln!("WARN: Ensure you provide mik32-uploader path in MIK32_UPLOADER_PATH environment variable");
        }
    }

    match env::var("MIK32_OPENOCD_PATH") {
        Ok(_) => (),
        Err(_) => {
            eprintln!("WARN: Ensure you provide openocd path in MIK32_OPENOCD_PATH environment variable");
        }
    }

    pb.set_message(message + "Initializing git repo");

    let git_cmd = Command::new("git")
        .arg("init")
        .arg(project_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match git_cmd {
        Ok(stat) => {
            if !stat.success() {
                eprintln!("Failed to initialize git repo.");
            }
        }
        Err(_) => {
            eprintln!("Failed to run git init.")
        }
    }

    pb.finish_with_message("Done!");
    Ok(())
}


#[inline(always)]
fn cargo_add(project_dir: &PathBuf, dependency: String, git: bool) -> Result<(), InitError> {
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg("add");
    if git {
        cargo_cmd.arg("--git");
    }
    cargo_cmd.arg(dependency.clone());
    cargo_cmd.arg("--manifest-path");
    cargo_cmd.arg(project_dir.join("Cargo.toml"));

    let cargo_stat = cargo_cmd
        //.stdout(Stdio::null())
        //.stderr(Stdio::null())
        .status();

    match cargo_stat {
        Ok(stat) => {
            if !stat.success() {
                eprintln!("Failed to add dependency {}", dependency);
                return Err(InitError::FetchFailed);
            } else {
                println!("Added dependency: {}", dependency);
            }
        }
        Err(err) => {
            eprintln!("Failed to run cargo-add on dependency {}, {}", dependency, err);
            return Err(InitError::FetchFailed);
        }
    }
    Ok(())
}