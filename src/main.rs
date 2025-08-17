use clap::{error, Parser, Subcommand};
use log::{info, error};
use std::{fs, path::{Path, PathBuf}};

use crate::build_script::run_exec;


mod build_script;
mod init_script;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    init {
        name: String,
    },
    run {
        release: bool,
        openocd_path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    println!("Running command...");
    match cli.command {
        Commands::init { name } => {
            
            match init_script::init_script(name) {
                Ok(()) => {
                    info!("Project initialized!");
                }
                Err(e) => {
                    error!("Initialization failed with Error, {:?}", e);
                }
            }
        }

        Commands::run{
            openocd_path,
            release
        } => {
            match run_exec(release, openocd_path) {
                    Ok(()) => {
                        println!("Running");
                        info!("Build/Flash completed successfully!");
                    }
                    Err(e) => {
                        println!("Failed to run!: {:?}", e);
                        error!(
                            "Build/Flash exited due to error. {:?}", e
                        );
                    }
                };
        }
    }
}




