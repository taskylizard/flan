use clap::CommandFactory;
use clap::Parser;
use clap::ValueEnum;
use clap_complete::generate_to;
use clap_complete::Shell as CShell;
use common::config::AppConfig;
use confique::toml::{self, FormatOptions};
use eyre::Result;
use flan_cli::core::Cli as FlanCli;
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
    /// Generate shell completions
    Completions {
        /// Output directory
        #[arg(long)]
        out: Option<String>,
    },
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
                generate_completions(None)?;
            }
            cmd!(sh, "cargo {args...}").run()?
        }
        Cli::Clean => {
            cmd!(sh, "cargo clean").run()?;
        }
        Cli::Completions { out } => {
            generate_completions(out.as_deref())?;
        }
        Cli::Config {} => generate_config_template(),
    }

    Ok(())
}

fn generate_completions(out: Option<&str>) -> Result<()> {
    let target_dir = out.unwrap_or("target/completions");
    std::fs::create_dir_all(target_dir)?;
    for &shell in CShell::value_variants() {
        generate_to(shell, &mut FlanCli::command(), "flan-cli", target_dir)?;
    }

    Ok(())
}

fn generate_config_template() {
    let tmpl = toml::template::<AppConfig>(FormatOptions::default());
    std::fs::write("config.template.toml", tmpl).unwrap();

    println!("Config template file generated.");
}
