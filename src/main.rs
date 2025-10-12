//! SteamOS Emulator CLI
//! 
//! Command-line interface for the SteamOS emulator

use clap::{Parser, Subcommand};
use steamos_emulator::{SteamOSEmulator, EmulatorConfig, Result};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "steamos-emu")]
#[command(about = "A Rust-based Linux emulator for running SteamOS environments")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a SteamOS image
    Run {
        /// Path to the SteamOS disk image
        #[arg(short, long)]
        image: PathBuf,
        
        /// Memory size in MB
        #[arg(short, long, default_value = "4096")]
        memory: u64,
        
        /// Number of CPU cores to emulate
        #[arg(short, long, default_value = "4")]
        cores: u32,
    },
    
    /// Download the latest SteamOS recovery image
    Download {
        /// Output directory for the downloaded image
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        
        /// Force re-download even if image exists
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show information about a SteamOS image
    Info {
        /// Path to the SteamOS disk image
        image: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    
    // Load configuration
    let config = if let Some(config_path) = cli.config {
        EmulatorConfig::from_file(&config_path)?
    } else {
        EmulatorConfig::default()
    };
    
    match cli.command {
        Commands::Run { image, memory, cores } => {
            run_emulator(image, memory, cores, config).await
        }
        Commands::Download { output, force } => {
            download_steamos_image(output, force).await
        }
        Commands::Info { image } => {
            show_image_info(image).await
        }
    }
}

async fn run_emulator(image: PathBuf, memory_mb: u64, cores: u32, mut config: EmulatorConfig) -> Result<()> {
    log::info!("Starting SteamOS emulator");
    log::info!("Image: {:?}", image);
    log::info!("Memory: {} MB", memory_mb);
    log::info!("CPU cores: {}", cores);
    
    // Update config with command line parameters
    config.memory_size = memory_mb * 1024 * 1024; // Convert MB to bytes
    config.cpu_cores = cores;
    
    // Create and configure emulator
    let mut emulator = SteamOSEmulator::new(config)?;
    
    // Load the SteamOS image
    let image_path = image.to_string_lossy();
    emulator.load_steamos_image(&image_path).await?;
    
    // Start emulation
    emulator.start().await?;
    
    // Run the emulator
    emulator.run().await?;
    
    Ok(())
}

async fn download_steamos_image(output: PathBuf, force: bool) -> Result<()> {
    log::info!("Downloading SteamOS recovery image to {:?}", output);
    
    // This functionality is preserved from the original shell script
    let downloader = steamos_emulator::SteamOSDownloader::new();
    downloader.download_recovery_image(&output, force).await?;
    
    log::info!("Download completed successfully");
    Ok(())
}

async fn show_image_info(image: PathBuf) -> Result<()> {
    log::info!("Analyzing SteamOS image: {:?}", image);
    
    let analyzer = steamos_emulator::ImageAnalyzer::new();
    let info = analyzer.analyze(&image).await?;
    
    println!("SteamOS Image Information:");
    println!("  Path: {:?}", image);
    println!("  Size: {} MB", info.size_mb);
    println!("  Format: {}", info.format);
    println!("  Partitions: {}", info.partition_count);
    println!("  SteamOS Version: {}", info.steamos_version.unwrap_or_else(|| "Unknown".to_string()));
    println!("  Architecture: {}", info.architecture);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(&[
            "steamos-emu",
            "run",
            "--image", "/path/to/image.img",
            "--memory", "8192",
            "--cores", "8"
        ]);
        
        assert!(cli.is_ok());
        
        if let Ok(cli) = cli {
            match cli.command {
                Commands::Run { memory, cores, .. } => {
                    assert_eq!(memory, 8192);
                    assert_eq!(cores, 8);
                }
                _ => panic!("Expected Run command"),
            }
        }
    }
}