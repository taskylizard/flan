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
        #[arg(long, env = "FLAN_ADMIN_KEY")]
        admin_key: String,
    },
    /// Upload an image
    Upload {
        /// Path to the image file
        file: PathBuf,

        /// Username for authentication
        #[arg(short, long, env = "FLAN_USERNAME")]
        username: String,

        /// Access key for authentication
        #[arg(long, env = "FLAN_ACCESS_KEY")]
        access_key: String,
    },
    /// List uploaded images
    List {
        /// Username for authentication
        #[arg(long, env = "FLAN_USERNAME")]
        username: String,

        /// Access key for authentication
        #[arg(long, env = "FLAN_ACCESS_KEY")]
        access_key: String,
    },
    /// Get an image
    Get {
        /// File ID of the image
        file_id: String,

        /// Optional output path (defaults to current directory with file ID as name)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Delete an image
    Delete {
        /// File ID of the image to delete
        file_id: String,

        /// Username for authentication
        #[arg(long, env = "FLAN_USERNAME")]
        username: String,

        /// Access key for authentication
        #[arg(long, env = "FLAN_ACCESS_KEY")]
        access_key: String,
    },
}
