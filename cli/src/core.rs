use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(
        short,
        long,
        env = "FLAN_SERVER",
        default_value = "http://localhost:8111"
    )]
    pub server: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Register a new user
    Register {
        /// Username for the new account
        #[arg(short, long)]
        username: String,
        /// Admin key for registration
        #[arg(short, long)]
        admin_key: String,
    },
    /// Upload an image
    Upload {
        /// Path to the image file
        file: PathBuf,

        /// Username for authentication (can also use FLAN_USERNAME env var)
        #[arg(short, long, env = "FLAN_USERNAME")]
        username: String,

        /// Access key for authentication (can also use FLAN_ACCESS_KEY env var)
        #[arg(short, long, env = "FLAN_ACCESS_KEY")]
        key: String,
    },
    /// Get an image
    Get {
        /// File ID of the image
        file_id: String,

        /// Optional output path (defaults to current directory with file ID as name)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// List images
    List {
        /// Username for authentication (can also use FLAN_USERNAME env var)
        #[arg(short, long, env = "FLAN_USERNAME")]
        username: String,

        /// Access key for authentication (can also use FLAN_ACCESS_KEY env var)
        #[arg(short, long, env = "FLAN_ACCESS_KEY")]
        key: String,
    },
}
