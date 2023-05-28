use clap::Parser as ClapParser;
use git_version::git_version;
use postgang::bring_client::mailbox_delivery_dates::DeliveryDays;
use postgang::bring_client::NorwegianPostalCode;
use postgang::bring_client::{ApiKey, ApiUid};
use postgang::calendar::Calendar;
use postgang::io_error_to_string;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

const VERSION: &str = git_version!(
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);

fn postal_code_parser(value: &str) -> Result<NorwegianPostalCode, String> {
    NorwegianPostalCode::try_from(value).map_err(|err| err.to_string())
}

fn parse_api_key(value: &str) -> Result<ApiKey, String> {
    ApiKey::try_from(value).map_err(|err| format!("{:?}", err))
}

fn parse_api_uid(value: &str) -> Result<ApiUid, String> {
    ApiUid::try_from(value).map_err(|err| format!("{:?}", err))
}

#[derive(ClapParser, Debug)]
enum Commands {
    /// Get delivery dates from Bring API
    Api {
        #[arg(long, env = "POSTGANG_API_UID", value_parser = parse_api_uid, hide_env_values = true)]
        api_uid: ApiUid,
        #[arg(long, env = "POSTGANG_API_KEY", value_parser = parse_api_key, hide_env_values = true)]
        api_key: ApiKey,
    },
    /// Get delivery dates from JSON file
    File {
        /// File path
        input: PathBuf,
    },
}

#[derive(ClapParser, Debug)]
#[clap(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, value_parser = postal_code_parser)]
    /// Postal code
    code: NorwegianPostalCode,
    #[arg(long)]
    /// File path
    output: Option<PathBuf>,
}

async fn try_main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    log::debug!("Got CLI args: {:?}", cli);
    let endpoint = match cli.command {
        Commands::Api { api_key, api_uid } => DeliveryDays::api(api_key, api_uid),
        Commands::File { input } => DeliveryDays::file(input),
    };
    match cli.output {
        Some(path) => {
            // Try to create file before we do any network requests
            let mut file =
                std::fs::File::create(&path).map_err(|err| io_error_to_string(&err, &path))?;
            let cal: Calendar = endpoint.get(cli.code).await?.into();
            write!(file, "{cal}").map_err(|err| io_error_to_string(&err, &path))?;
        }
        None => {
            let cal: Calendar = endpoint.get(cli.code).await?.into();
            std::io::stdout().write_fmt(format_args!("{cal}"))?
        }
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    env_logger::init();

    match try_main().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}
