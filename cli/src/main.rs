use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, Color, ContentArrangement,
    Table,
};
use common::{
    list::ListImagesResponse,
    register::{RegisterUserRequest, RegisterUserResponse},
    upload::UploadImageResponse,
};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

use console::{style, Term};
use reqwest::{
    multipart::{Form, Part},
    Client, StatusCode,
};
use tokio::fs::File;
mod core;
use core::{Cli, Commands};

async fn register(
    client: &Client,
    server: &str,
    username: String,
    admin_key: String,
) -> Result<()> {
    let term = Term::stdout();
    term.write_line("")?;

    // Show progress spinner
    term.write_line(&format!(
        "{} Registering user {}...",
        style("[1/2]").bold().dim(),
        style(&username).cyan()
    ))?;

    let url = format!("{}/api/register", server);
    let request = RegisterUserRequest {
        username,
        admin_key,
    };

    let response = client.post(&url).json(&request).send().await?;

    match response.status() {
        StatusCode::OK => {
            let result = response.json::<RegisterUserResponse>().await?;
            term.clear_last_lines(1)?;
            term.write_line(&format!(
                "{} Registration successful!",
                style("✔").green().bold()
            ))?;
            term.write_line("")?;

            // Pretty print the credentials
            println!("{}", style("Your Credentials").bold());
            println!("{}", style("───────────────").dim());
            println!(
                "{} {}",
                style("Username:").bold(),
                style(&result.username).cyan()
            );
            println!(
                "{} {}",
                style("Access Key:").bold(),
                style(&result.key).green()
            );

            // Print warning about saving the key
            println!("{}", style("⚠ Important:").yellow().bold());
            println!(
                "{}",
                style("Save your access key securely. It cannot be recovered if lost.").yellow()
            );

            // Print usage instructions
            println!("{}", style("Quick Start:").bold());
            println!("{}", style("───────────────").dim());
            println!("Export your credentials:");
            println!(
                "{} export FLAN_USERNAME='{}'",
                style("$").dim(),
                result.username
            );
            println!(
                "{} export FLAN_ACCESS_KEY='{}'",
                style("$").dim(),
                result.key
            );
            println!("Upload an image:");
            println!("{} flan-cli upload --file image.jpg", style("$").dim());

            Ok(())
        }
        StatusCode::UNAUTHORIZED => Err(eyre!("{} Invalid admin key", style("✘").red().bold())),
        StatusCode::BAD_REQUEST => Err(eyre!("{} Username already taken", style("✘").red().bold())),
        _ => Err(eyre!(
            "{} Server error: {} - {}",
            style("✘").red().bold(),
            response.status(),
            response.text().await?
        )),
    }
}

async fn upload(
    client: &Client,
    server: &str,
    file_path: PathBuf,
    username: String,
    access_key: String,
) -> Result<()> {
    let term = Term::stdout();
    let file_name = file_path
        .file_name()
        .ok_or_else(|| eyre!("Invalid file name"))?
        .to_string_lossy();

    term.write_line("")?;
    term.write_line(&format!(
        "{} Reading file {}...",
        style("[1/2]").bold().dim(),
        style(&file_name).cyan()
    ))?;

    // Read file
    let mut file = File::open(&file_path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    // Get the mime type
    let mime_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    term.clear_last_lines(1)?;
    term.write_line(&format!(
        "{} Uploading {}...",
        style("[2/2]").bold().dim(),
        style(&file_name).cyan()
    ))?;

    let url = format!("{}/api/upload", server);
    let part = Part::bytes(buffer)
        .file_name(file_name.to_string())
        .mime_str(&mime_type)?;

    let form = Form::new().part("file", part);

    let response = client
        .post(&url)
        .header("X-Username", username)
        .header("X-Access-Key", access_key)
        .multipart(form)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let result = response.json::<UploadImageResponse>().await?;
            term.clear_last_lines(1)?;
            term.write_line(&format!("{} Upload successful!", style("✔").green().bold()))?;
            term.write_line("")?;

            println!(
                "{} {}",
                style("File ID:").bold(),
                style(&result.file_id).cyan()
            );
            println!("{} {}", style("URL:").bold(), style(&result.url).green());
            println!("Download your image:");
            println!("{} flan-cli get {}", style("$").dim(), result.file_id);

            Ok(())
        }
        StatusCode::UNAUTHORIZED => Err(eyre!("{} Invalid credentials", style("✘").red().bold())),
        StatusCode::BAD_REQUEST => {
            Err(eyre!("{} Invalid file or request", style("✘").red().bold()))
        }
        _ => Err(eyre!(
            "{} Server error: {} - {}",
            style("✘").red().bold(),
            response.status(),
            response.text().await?
        )),
    }
}

async fn get(
    client: &Client,
    server: &str,
    file_id: String,
    output: Option<PathBuf>,
) -> Result<()> {
    let term = Term::stdout();
    term.write_line("")?;
    term.write_line(&format!(
        "{} Downloading image {}...",
        style("[1/2]").bold().dim(),
        style(&file_id).cyan()
    ))?;

    let url = format!("{}/images/{}", server, file_id);
    let response = client.get(&url).send().await?;

    match response.status() {
        StatusCode::OK => {
            let content_type = response
                .headers()
                .get("content-type")
                .ok_or_else(|| eyre!("No content-type header"))?
                .to_str()?;

            let extension = content_type.split('/').last().unwrap_or("jpg");
            let output_path = match output {
                Some(path) => path,
                None => PathBuf::from(format!("{}.{}", file_id, extension)),
            };

            term.clear_last_lines(1)?;
            term.write_line(&format!("{} Saving image...", style("[2/2]").bold().dim()))?;

            let bytes = response.bytes().await?;
            tokio::fs::write(&output_path, bytes).await?;

            term.clear_last_lines(1)?;
            term.write_line(&format!(
                "{} Image downloaded successfully!",
                style("✔").green().bold()
            ))?;
            term.write_line("")?;

            println!("{}", style("Download Details").bold());
            println!("{}", style("────────────────").dim());
            println!(
                "{} {}",
                style("Saved to:").bold(),
                style(output_path.display()).cyan()
            );
            Ok(())
        }
        StatusCode::NOT_FOUND => Err(eyre!("{} Image not found", style("✘").red().bold())),
        _ => Err(eyre!(
            "{} Server error: {} - {}",
            style("✘").red().bold(),
            response.status(),
            response.text().await?
        )),
    }
}

async fn list(client: &Client, server: &str, username: String, access_key: String) -> Result<()> {
    let term = Term::stdout();
    term.write_line("")?;
    term.write_line(&format!(
        "{} Listing images...",
        style("[1/2]").bold().dim()
    ))?;

    let url = format!("{}/api/images", server);
    let response = client
        .get(&url)
        .header("X-Username", username)
        .header("X-Access-Key", access_key)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let result = response.json::<ListImagesResponse>().await?;
            term.clear_last_lines(1)?;
            let mut table = Table::new();
            table
                .set_content_arrangement(ContentArrangement::Dynamic)
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec![
                    Cell::new("File ID")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Green),
                    Cell::new("Created At")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Cyan),
                ]);
            for image in &result.images {
                table.add_row(vec![
                    Cell::new(&image.file_id),
                    Cell::new(image.created_at.to_string()),
                ]);
            }

            println!("{table}");

            // Instructions on how to get a image from file-id
            println!("Get a image using the image id:");
            println!("{} flan-cli get <image-id>", style("$").bold().dim());
            Ok(())
        }

        StatusCode::UNAUTHORIZED => Err(eyre!("{} Invalid credentials", style("✘").red().bold())),
        _ => Err(eyre!(
            "{} Server error: {} - {}",
            style("✘").red().bold(),
            response.status(),
            response.text().await?
        )),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let client = Client::new();

    match cli.command {
        Commands::Register {
            username,
            admin_key,
        } => {
            register(&client, &cli.server, username, admin_key).await?;
        }
        Commands::Upload {
            file,
            username,
            key,
        } => {
            upload(&client, &cli.server, file, username, key).await?;
        }
        Commands::Get { file_id, output } => {
            get(&client, &cli.server, file_id, output).await?;
        }
        Commands::List { username, key } => {
            list(&client, &cli.server, username, key).await?;
        }
    }

    Ok(())
}
