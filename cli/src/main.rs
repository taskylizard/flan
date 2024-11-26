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
use console::style;
use core::{Cli, Commands};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    multipart::{Form, Part},
    Client, StatusCode,
};
use std::path::PathBuf;
mod core;

async fn register_user(
    client: &Client,
    server_url: &str,
    username: String,
    admin_key: String,
) -> Result<()> {
    let url = format!("{}/api/register", server_url);
    let request = RegisterUserRequest {
        username,
        admin_key,
    };

    let response = client.post(&url).json(&request).send().await?;

    match response.status() {
        StatusCode::OK => {
            let register_response: RegisterUserResponse = response.json().await?;
            println!("User registered successfully:");
            println!("Username: {}", register_response.username);
            println!("Access Key: {}", register_response.key);
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

async fn upload_image(
    client: &Client,
    server_url: &str,
    file_path: PathBuf,
    username: String,
    access_key: String,
) -> Result<()> {
    let file_name = &file_path
        .file_name()
        .ok_or_else(|| eyre!("Invalid file name"))?
        .to_string_lossy();

    // Read file
    let file = tokio::fs::read(file_name.to_string()).await?;

    // Create multipart form
    let form = Form::new().part(
        "file",
        Part::bytes(file)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")?,
    );

    // Prepare headers
    let mut headers = HeaderMap::new();
    headers.insert("X-Username", HeaderValue::from_str(&username)?);
    headers.insert("X-Access-Key", HeaderValue::from_str(&access_key)?);

    let response = client
        .post(format!("{}/api/upload", server_url))
        .headers(headers)
        .multipart(form)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let upload_response: UploadImageResponse = response.json().await?;
            println!("Image uploaded successfully:");
            println!("File ID: {}", upload_response.file_id);
            println!("URL: {}", upload_response.url);
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

async fn list_images(
    client: &Client,
    server_url: &str,
    username: &str,
    access_key: &str,
) -> Result<()> {
    // Prepare headers
    let mut headers = HeaderMap::new();
    headers.insert("X-Username", HeaderValue::from_str(username)?);
    headers.insert("X-Access-Key", HeaderValue::from_str(access_key)?);

    let response = client
        .get(format!("{}/api/list", server_url))
        .headers(headers)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let result: ListImagesResponse = response.json().await?;

            if result.images.is_empty() {
                println!("No images found.");
            } else {
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
            }
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

async fn get_image(
    client: &Client,
    server: &str,
    file_id: String,
    output: Option<PathBuf>,
) -> Result<()> {
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

            let bytes = response.bytes().await?;
            tokio::fs::write(&output_path, bytes).await?;

            println!(
                "{} Image downloaded successfully!",
                style("✔").green().bold()
            );

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

async fn delete_image(
    client: &Client,
    server_url: &str,
    file_id: &str,
    username: &str,
    access_key: &str,
) -> Result<()> {
    // Prepare headers
    let mut headers = HeaderMap::new();
    headers.insert("X-Username", HeaderValue::from_str(username)?);
    headers.insert("X-Access-Key", HeaderValue::from_str(access_key)?);

    let response = client
        .delete(format!("{}/api/delete/{}", server_url, file_id))
        .headers(headers)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::NO_CONTENT => {
            println!("Image deleted successfully: {}", file_id);
            Ok(())
        }
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
            register_user(&client, &cli.server, username, admin_key).await?;
        }
        Commands::Upload {
            file,
            username,
            access_key,
        } => {
            upload_image(&client, &cli.server, file, username, access_key).await?;
        }
        Commands::Get { file_id, output } => {
            get_image(&client, &cli.server, file_id, output).await?;
        }
        Commands::List {
            username,
            access_key,
        } => {
            list_images(&client, &cli.server, &username, &access_key).await?;
        }
        Commands::Delete {
            file_id,
            username,
            access_key,
        } => {
            delete_image(&client, &cli.server, &file_id, &username, &access_key).await?;
        }
    }

    Ok(())
}
