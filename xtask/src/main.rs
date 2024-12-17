use clap::Parser;
use common::config::AppConfig;
use confique::toml::{self, FormatOptions};
use eyre::Result;
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(name = "xtask")]
enum Cli {
    /// Build the project
    Build {
        #[arg(long)]
        release: bool,
    },
    /// Clean build artifacts
    Clean,
    /// Generate config template
    Config,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli {
        Cli::Build { release } => {
            let mut args = vec!["build"];
            if release {
                args.push("--release");
            }
            cmd!(sh, "cargo {args...}").run()?
        }
        Cli::Clean => {
            cmd!(sh, "cargo clean").run()?;
        }
        Cli::Config {} => generate_config_template(),
    }

    Ok(())
}

fn generate_config_template() {
    let tmpl = toml::template::<AppConfig>(FormatOptions::default());
    std::fs::write("config.template.toml", tmpl).unwrap();

    println!("Config template file generated.");
}
