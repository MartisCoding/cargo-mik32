use std::{env::{self}, path::{absolute, PathBuf}, process::{Command, Stdio}, str::FromStr};

use crate::FlashCmdDescriptor;

#[derive(Debug)]
pub enum RunError {
    PackageNotInstalled,
    ObjcopyFailed,
    UploadFailed,
    NoGdbExec,
    GdbFailed,
    NotAProject,
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version") // любой аргумент, который не сломает команду
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Perform objcopy operation from cargo-binutils. If app_hex_path is provided with flag reuse procedure will check its existance.
/// If app_hex_path provided without reuse procedure will build the app in destination.
/// If no app_hex_path provided procedure will build app in default destination ./flash/app.hex.
fn objcopy(
    app_hex_path: &mut Option<PathBuf>,
    reuse: bool,
    example: Option<String>,
    project_dir: &PathBuf, 
) -> Result<(), RunError> 
{
    if app_hex_path.is_some() && reuse{
        if !absolute(app_hex_path.clone().unwrap()).unwrap().exists() {
            eprintln!("Binary hex path was provided with reuse flag. However, the binary seems to not exist. Build the binary first.");
            return Err(RunError::ObjcopyFailed);
        }
        return Ok(())
    }

    if !command_exists("cargo-objcopy") {
        eprintln!("
                cargo-objcopy not found. Install it:\n
                cargo install cargo-binutils\n
                rustup component add llvm-tools-preview"
            );
        return Err(RunError::PackageNotInstalled);
    }

    let app_path: PathBuf;
    if app_hex_path.is_some() {
        app_path = app_hex_path.clone().unwrap();
    } else {
        app_path = project_dir.join("flash").join("app.hex");
    }

    let mut objcopy = Command::new("cargo");
    objcopy.arg("objcopy");
    objcopy.arg("--release");
    if example.is_some() {
        objcopy.args([
            "--example",
            &example.unwrap()
        ]);
    }
    objcopy.args([
        "--",
        "-O",
        "ihex",
        absolute(app_path.clone()).unwrap().to_str().unwrap()
    ]);
    
    let obcp_stat = objcopy.status().expect("Failed to run objcopy");
    if !obcp_stat.success() {
        eprintln!("Objcopy failed due to error");
        return Err(RunError::ObjcopyFailed);
    }
    *app_hex_path = Some(absolute(app_path).unwrap());
    Ok(())
}


///Upload procedure accepts openocd path as well as uploader path, to initiate upload script.
///If no uploader path was provided it will seek it in MIK32_UPLOADER_PATH env variable and in project folder and in case of failure will panic.
///Whereas, if no openocd path was provided it will seek it in env variable MIK32_OPENOCD_PATH and then via which command.
///Also it checks mik32_upload.py existance.
fn upload(
    openocd_path: Option<PathBuf>,
    uploader_path: Option<PathBuf>,
    app_hex_path: Option<PathBuf>,
    use_quad_spi: bool,
    openocd_host: Option<String>,
    openocd_port: Option<String>,
    adapter_speed: Option<String>,
    openocd_scripts: Option<PathBuf>,
    openocd_interface: Option<PathBuf>,
    openocd_target: Option<PathBuf>,
    project_dir: &PathBuf
    
) -> Result<(), RunError> 
{
    let mut uploader_final_path: Option<PathBuf> = None;
    let mut openocd_final_path: Option<PathBuf> = None;
    println!("Fetching mik32 uploader path...");

    let mut uploader_fetch_success = false;
    if uploader_path.is_none() {
        println!("Fetching uplaoder path from MIK32_UPLOADER_PATH variable...");
        match env::var("MIK32_UPLOADER_PATH") {
            Ok(path) => {
                uploader_final_path = Some(PathBuf::from_str(&path)
                    .expect("Failed to make PathBuf out of environment variable"));
                uploader_fetch_success = true
            }
            Err(e) => {
                eprintln!("Fetching from MIK32_UPLOADER_PATH environment variable failed. Consider setting it to desired path. {}", e);
            }
        }
    } 
    if uploader_path.is_none() && !uploader_fetch_success {
        println!("Fetching uploader path from project directory...");
        let uploader_project_path = project_dir
            .join("flash")
            .join("mik32_uploader");
        if !uploader_project_path.exists() {
            eprintln!("Fetching from project directory failed. Consider cloning uploader to ./flash directory or setting MIK32_UPLOADER_PATH to desired value");
            return Err(RunError::UploadFailed);
        }
        uploader_final_path = Some(uploader_project_path);
    }

    if uploader_path.is_some() {
        println!("Fetching uploader path from provided value...");
        uploader_final_path = uploader_path.clone();
    }

    if uploader_final_path.is_none() {
        eprintln!("Failed to fetch uploader path due to unexpected error...");
        return Err(RunError::UploadFailed);
    }

    let uploader_final_path = uploader_final_path.unwrap();
    print!("Validating uploader path... ");
    if !uploader_final_path.join("mik32_uploader.py").exists() {
        println!("ERROR\n");
        eprintln!("Could not find mik32_uploader.py in provided path. Make sure to clone mik32_uploader correctly...");
        return Err(RunError::UploadFailed);
    }
    println!("OK\n");
    println!("Successfuly fetched mik32 uploader path!");

    println!("Fetching openocd path...");

    let mut openocd_fetch_success = false;
    if openocd_path.is_none() {
        println!("Fetching openocd from env variable MIK32_OPENOCD_PATH...");
        match env::var("MIK32_OPENOCD_PATH") {
            Ok(path) => {
                openocd_final_path = Some(
                    PathBuf::from_str(&path).expect("Failed to make PathBuf out of environment variable")
                );
                openocd_fetch_success = true;
            }
            Err(e) => {
                eprintln!("Fetchig from MIK32_OPENOCD_PATH environment variable failed. Consider setting it to desired path. {}", e);
            }
        }
    }
    if openocd_path.is_none() && !openocd_fetch_success {
        println!("Fetching openocd path through which command...");
        let which_cmd = Command::new("which")
            .arg("openocd")
            .output()
            .expect("Failed to run openocd");

        if !which_cmd.status.success() || which_cmd.stdout.is_empty() {
            eprintln!("'which openocd' failed. This might be caused due to unpropper installation of openocd.");
            return Err(RunError::UploadFailed);
        }
        openocd_final_path = Some(
            PathBuf::from_str(
                &String::from_utf8(which_cmd.stdout)
                    .expect("Failed to make utf-8 string out of stdout of 'which openocd'")
            ).expect("Failed to make PathBuf out of stdout string")
        );
    }

    if openocd_path.is_some() {
        openocd_final_path = openocd_path.clone();
    }
    
    if openocd_final_path.is_none() {
        eprintln!("Failed to fetch openocd path due to unexpected error...");
        return Err(RunError::UploadFailed);
    }
    let openocd_final_path = openocd_final_path.unwrap();

    print!("Validating openocd... ");
    if !command_exists(openocd_final_path.to_str().unwrap()) {
        println!("ERROR");
        eprintln!("Seems that openocd command isn't working or doesn't exist at all, please make sure to install openocd correctly.");
        return Err(RunError::UploadFailed);
    }

    println!("OK\n");
    println!("Successfuly fetched openocd path. Preparing to upload...");

    
    if app_hex_path.is_none() {
        eprintln!("Unexpected error during upload preparation...");
        return Err(RunError::UploadFailed);
    }
    
    if !command_exists("Python3") {
        eprintln!("Python3 command not found. Make sure to properly install it.");
        return Err(RunError::PackageNotInstalled);
    }

    println!("Uploading...");
    let mut upload_cmd = Command::new("python3");
    upload_cmd.arg(absolute(uploader_final_path.join("mik32_uploader.py")).unwrap().to_str().unwrap());
    upload_cmd.arg("--run-openocd");
    upload_cmd.arg("--openocd-exec");
    upload_cmd.arg(absolute(openocd_final_path).unwrap().to_str().unwrap());
    upload_cmd.arg("--openocd-scripts");
    upload_cmd.arg(absolute(uploader_final_path.join("openocd-scripts")).unwrap().to_str().unwrap());
    upload_cmd.arg(absolute(app_hex_path.unwrap()).unwrap().to_str().unwrap());
    
    if use_quad_spi {
        upload_cmd.arg("--use-quad-spi");
    }

    if openocd_host.is_some() {
        upload_cmd.args([
            "--openocd-host",
            &openocd_host.unwrap(),
        ]);
        
    }

    if openocd_port.is_some() {
        upload_cmd.args([
            "--openocd-port", 
            &openocd_port.unwrap()
        ]);
    }

    if adapter_speed.is_some() {
        upload_cmd.args([
            "--adapter-speed",
            &adapter_speed.unwrap()
        ]);
    }

    if openocd_scripts.is_some() {
        upload_cmd.args([
            "--openocd-scripts",
            absolute(openocd_scripts.unwrap()).unwrap().to_str().unwrap()
        ]);
    }

    if openocd_interface.is_some() {
        upload_cmd.args([
            "--openocd-interface",
            openocd_interface.unwrap().to_str().unwrap()
        ]);
    }

    if openocd_target.is_some() {
        upload_cmd.args([
            "--openocd-target",
            openocd_target.unwrap().to_str().unwrap()
        ]);
    }

    println!("Uploading...");

    let upload_status = upload_cmd.status().expect("Failed to run upload");
    if !upload_status.success() {
        eprintln!("Failed to upload application");
        return Err(RunError::UploadFailed);
    }
    
    println!("Aplication uploaded successfully");
    Ok(())
}

fn connect_gdb(gdb_exec: Option<String>, app_hex_path: Option<PathBuf>) -> Result<(), RunError>{
    if gdb_exec.is_none() {
        eprintln!("gdb executable was not provided. Skipping this step.");
        return Err(RunError::NoGdbExec);
    }

    println!("Performing attach to GDB executable provided...");
    let mut gdb_cmd = Command::new(gdb_exec.unwrap());
    gdb_cmd.arg(absolute(app_hex_path.unwrap()).unwrap().to_str().unwrap());
    gdb_cmd.arg("-x");
    gdb_cmd.arg("-");

    let child = gdb_cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn();

    match child {
        Ok(mut child) => {
            use std::io::Write;
            child.stdin.as_mut().unwrap()
                .write_all(
                    r#"EOF
                set mem inaccessible-by-default off
                mem 0x01000000 0x01002000 ro
                mem 0x80000000 0xffffffff ro
                set arch riscv:rv32
                set remotetimeout 10
                set remote hardware-breakpoint-limit 2
                target remote localhost:3333
                load
                EOF
                "#.as_bytes()).unwrap();
            let _ = child.wait().unwrap();
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to run gdb due to unexpected error, {}", e);
            Err(RunError::GdbFailed)
        }
    }
}

pub fn run_wrapper(mut desc: FlashCmdDescriptor) -> Result<(), RunError>{

    if !desc.project_dir.join("Cargo.toml").exists() {
        eprintln!("Not a project directory. Exiting...");
        return Err(RunError::NotAProject);
    }

    if desc.reuse && desc.example.is_some() {
        eprintln!("Unresolved arugents. Using 'reuse' will skip objcopy step completely. 'example' argument here is useless because it aplies itself to objcopy.");
    }

    objcopy(
        &mut desc.app_hex_path,
        desc.reuse,
        desc.example,
        &desc.project_dir
    )?;
    upload(
        desc.openocd_path,
        desc.uploader_path,
        desc.app_hex_path.clone(),
        desc.use_quad_spi,
        desc.openocd_host,
        desc.openocd_port,
        desc.adapter_speed,
        desc.openocd_scripts,
        desc.openocd_interface,
        desc.openocd_target,
        &desc.project_dir
    )?;
    connect_gdb(desc.gdb_exec, desc.app_hex_path)?;
    Ok(())
}