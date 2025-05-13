mod inline;
mod logger;
mod openai;
mod translator;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gettext-translator")]
#[command(about = "Translates gettext() strings or .po files using OpenAI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Inline {
        /// Path to scan
        folder: std::path::PathBuf,

        /// Dry run
        #[arg(long)]
        dry_run: bool,

        /// API Key
        #[arg(long)]
        api_key: Option<String>,
    },
    Translator {
        folder: std::path::PathBuf,

        /// Comma-separated list of target languages
        #[arg(long)]
        lang: String,

        /// If set, no files are modified
        #[arg(long)]
        dry_run: bool,

        /// If set, all entries are re-translated, even if they have a value
        #[arg(long)]
        force: bool,

        /// Extra context for the prompt. If not set, the program will look for a file named context.txt in the root folder
        #[arg(long)]
        context: Option<std::path::PathBuf>,

        /// API Key
        #[arg(long)]
        api_key: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inline {
            folder,
            dry_run,
            api_key,
        } => {
            inline::run(folder, dry_run, api_key).await?;
        }
        Commands::Translator {
            folder,
            lang,
            dry_run,
            force,
            context,
            api_key,
        } => {
            translator::run(folder, &lang, dry_run, force, context, api_key).await?;
        }
    }

    Ok(())
}
