use clap::{Parser, Subcommand, ValueEnum};
use std::{env::current_dir, path::PathBuf};

use crate::{build_script::run_wrapper};


mod build_script;
mod init_script;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}



#[derive(Subcommand)]
enum Commands {
    Init {
        name: String,
        //#[arg(long, help="Pass a chip model. Not yet implemented.")]
        //chip: Option<String>,
    },
    Run {
        #[arg(short, long, help="Pass an example. Will upload example application. Rebuilding is necessary. If you provide 'reuse' flag will skip objcopy therefore will not build example.")]
        example: Option<String>,
        #[arg(long, help="Reuse flag. If and only if app-hex-path was provided will skip objcopy, check binary existance and perform upload.")]
        reuse: bool,
        #[arg(short, long, help="Pass a gdb executable. If provided will try to connect to a board with internal gdb script.")]
        gdb_exec: Option<String>,
        #[arg(short, long, help="Pass an openocd path. Otherwise will seek in MIK32_OPENOCD_PATH environment variable and then using 'which openocd' command.")]
        openocd_path: Option<PathBuf>,
        #[arg(short, long, help="Pass a uploader path manually. Otherwise will seek in MIK32_UPLOADER_PATH environment variable and then in project directory.")]
        uploader_path: Option<PathBuf>,
        #[arg(short, long, help="Pass a hex binary manually. Will skip objcopy step and upload application immideatly. Otherwise will automatically rebuild app.")]
        app_hex_path: Option<PathBuf>,

        //All essential uploader arguments are passed.
        #[arg(long, help="Direct argument pass from uploader. Use QuadSPI mode while programming external flash memory.")]
        use_quad_spi: bool,
        #[arg(long, help="Direct argument pass from uploader. Connection address to openocd server. 127.0.0.1 by default")]
        openocd_host: Option<String>,
        #[arg(long, help="Direct argument pass from uploader. Port of tcl openocd server. 6666 by default.")]
        openocd_port: Option<String>,
        #[arg(long, help="Direct argument pass from uploader. Speed of debugger in kHz. 500 bu default")]
        adapter_speed: Option<String>,
        #[arg(long, help="Pass openocd scripts manually. Will ignore default location of 'scripts' directory and use provided instead.")]
        openocd_scripts: Option<PathBuf>,
        #[arg(long, help="Direct argument pass from uploader. Path to configuration file of debugger relative to 'scripts' path. 'interface/ftdi/m-link.cfg' by default")]
        openocd_interface: Option<PathBuf>,
        #[arg(long, help="Direct argument pass from uploader. Path to configuration file of target MCU relative to 'scripts' path. 'target/mik32.cfg' by default")]
        openocd_target: Option<PathBuf>,
        #[arg(short, long, help="Select memory type. Not yet implemented.")]
        boot_mode: Option<BootMode>,
        #[arg(short, long, help="MCU type selection. Not yet implemented.")]
        mcu_type: Option<MCUType>,
    },
}

#[derive(ValueEnum, Clone)]
enum BootMode {
    Undefined,
    Eeprom,
    Ram,
    Spifi,
}

#[derive(ValueEnum, Clone)]
enum MCUType {
    MIK32V0,
    MIK32V2,
}

#[allow(dead_code)]
struct FlashCmdDescriptor {
    example: Option<String>,
    reuse: bool,
    gdb_exec: Option<String>,
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
    boot_mode: Option<BootMode>,
    mcu_type: Option<MCUType>,

    project_dir: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let current_dir = current_dir().expect("Failed to get project directory");
    match cli.command {
        Commands::Init { name } => {
            init_script::make_project(name, current_dir).unwrap();
        }
        Commands::Run { 
            example, 
            reuse,
            gdb_exec, 
            openocd_path, 
            uploader_path, 
            app_hex_path, 
            use_quad_spi, 
            openocd_host, 
            openocd_port, 
            adapter_speed, 
            openocd_scripts, 
            openocd_interface, 
            openocd_target, 
            boot_mode, 
            mcu_type } => {
                run_wrapper(FlashCmdDescriptor { 
                    example,
                    reuse, 
                    gdb_exec, 
                    openocd_path,
                    uploader_path, 
                    app_hex_path, 
                    use_quad_spi, 
                    openocd_host, 
                    openocd_port, 
                    adapter_speed, 
                    openocd_scripts, 
                    openocd_interface, 
                    openocd_target, 
                    boot_mode, 
                    mcu_type, 
                    project_dir: current_dir,
                }).unwrap()
            }
    }
}




