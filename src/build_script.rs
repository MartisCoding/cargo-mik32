use std::{env::current_dir, fs, path::PathBuf, process::Command, str::FromStr};
use log::error;

#[derive(Debug)]
pub enum RunError {
    PackageNotInstalled,
    ObjcopyFailed,
    UploadFailed,
    FsFailed,
    NoBinaryFound,
}

pub fn run_exec(release: bool, openocd_path: Option<PathBuf>) -> Result<(), RunError>{
    let project_dir = current_dir().expect("Could not retrieve project directory");

    if !command_exists("cargo-objcopy") {
        eprintln!("
                cargo-objcopy not found. Install it:\n
                cargo install cargo-binutils\n
                rustup component add llvm-tools-preview"
            );
        return Err(RunError::PackageNotInstalled);
    }
    

    if !command_exists("python3") {
        eprintln!(
            "python3 not found. Install it via system package manager."
        );
        return Err(RunError::PackageNotInstalled);
    }

    if !command_exists("gdb-multiarch") {
        eprintln!("Gdb-multiarch not found. Consr installing via system package manager.")
    }
    let flash_dir = project_dir.join("flash");

    let app_path = flash_dir.join("app.hex");

    if !app_path.exists() {
        let mut objcopy = Command::new("cargo objcopy");
        if release {
            objcopy.arg("--release");
        }
        objcopy.arg("--");
        objcopy.arg("-O");
        objcopy.arg("ihex");
        objcopy.arg(app_path.clone());
        let objcp_stat = objcopy
            .status()
            .expect("Failed to cargo-objcopy");

        if !objcp_stat.success() {
            error!("Objcopy failed!");
            return Err(RunError::ObjcopyFailed);
        }
    }

    let uploader_dir = flash_dir.join("mik32-uploader");

    let upload_script = uploader_dir.join("mik32_upload.py");
    if !upload_script.exists() {
        error!("Upload script not found in path, {}", upload_script.to_str().unwrap());
    }
    
    let openocd_path = openocd_path.unwrap_or(PathBuf::from_str("/usr/bin/openocd").unwrap());

    println!("Using openocd {}", openocd_path.to_str().unwrap());

    let mut py_cmd = Command::new("python3");
    py_cmd.arg(upload_script);
    py_cmd.arg("--run-openocd");
    py_cmd.arg("--openocd-exec");
    py_cmd.arg(openocd_path);
    py_cmd.arg("--openocd-scripts");
    py_cmd.arg(uploader_dir.join("oponcd-scripts"));
    py_cmd.arg(app_path);

    let pycmd_stat = py_cmd
        .status()
        .expect("Failed to execute python3");

    if !pycmd_stat.success() {
        error!("Upload failed!");
        return Err(RunError::UploadFailed);
    };

    let gdb_script = r#"EOF
    set mem inaccessible-by-default off
    mem 0x01000000 0x01002000 ro
    mem 0x80000000 0xffffffff ro
    set arch riscv:rv32
    set remotetimeout 10
    set remote hardware-breakpoint-limit 2
    target remote localhost:3333
    load
    EOF"#;

    let mut target = project_dir.join("target");
    if release {
        target = target.join("release");
    } else {
        target = target.join("debug");
    }

    let Some(entries) = fs::read_dir(&target).ok() else {
        eprintln!("No entries in target directory");
        return Err(RunError::FsFailed);
    };


    let mut entry_path: Option<PathBuf> = None;

    for entry in entries.into_iter() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            #[cfg(unix)]
            {
                // На Unix исполняемые обычно без расширения
                if path.extension().is_none() {
                    entry_path = Some(path);
                    break;
                }
            }
        }
    }

    let Some(target_path) = entry_path else {
        eprintln!("No binary found!");
        return Err(RunError::NoBinaryFound);
    };
    let mut gdb = Command::new("gdb-multiarch");
    gdb.arg("-x");
    gdb.arg(gdb_script);
    gdb.arg(target_path);


    let gdbstat = gdb.status().expect("Failed to run gdb_multiarch");

    if !gdbstat.success() {
        error!("Gdb connection failed!");
        return Err(RunError::UploadFailed);
    };
    Ok(())
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version") // любой аргумент, который не сломает команду
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}
