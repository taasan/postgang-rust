use std::{error::Error, io::Write, path::PathBuf, process::ExitCode};

use clap::{Parser as ClapParser, ValueEnum};
use git_version::git_version;

use postgang::bring_client::mailbox_delivery_dates::{ApiResponse, DeliveryDate};
use postgang::{
    bring_client::{mailbox_delivery_dates::DeliveryDays, ApiKey, ApiUid, NorwegianPostalCode},
    calendar::Calendar,
    io_error_to_string,
};

pub struct ApiResponseWithPostalCode {
    pub response: ApiResponse,
    pub postal_code: NorwegianPostalCode,
}

impl From<ApiResponseWithPostalCode> for Vec<DeliveryDate> {
    fn from(value: ApiResponseWithPostalCode) -> Self {
        value
            .response
            .delivery_dates
            .iter()
            .map(|date| DeliveryDate::new(value.postal_code, *date))
            .collect()
    }
}

const VERSION: &str = git_version!(
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);

fn postal_code_parser(value: &str) -> Result<NorwegianPostalCode, String> {
    NorwegianPostalCode::try_from(value).map_err(|err| err.to_string())
}

fn parse_api_key(value: &str) -> Result<ApiKey, String> {
    ApiKey::try_from(value).map_err(|err| format!("{err:?}"))
}

fn parse_api_uid(value: &str) -> Result<ApiUid, String> {
    ApiUid::try_from(value).map_err(|err| format!("{err:?}"))
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
        /// File path, read from stdin of omitted
        input: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Ical,
    Json,
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
    /// File path, print to stdout if omitted
    output: Option<PathBuf>,
    /// Output format
    #[arg(value_enum, long, default_value_t = OutputFormat::Ical)]
    format: OutputFormat,
}

async fn try_main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    log::debug!("Got CLI args: {:?}", cli);
    let endpoint = match cli.command {
        Commands::Api { api_key, api_uid } => DeliveryDays::api(api_key, api_uid),
        Commands::File { input } => DeliveryDays::file(input),
    };
    let output = match cli.format {
        OutputFormat::Ical => {
            let response: ApiResponse = endpoint.get(cli.code).await?;
            log::debug!("Got: {:?}", response);
            let delivery_dates: Vec<DeliveryDate> = ApiResponseWithPostalCode {
                response,
                postal_code: cli.code,
            }
            .into();
            let cal: Calendar = delivery_dates.into();
            format!("{cal}")
        }
        OutputFormat::Json => {
            let response: serde_json::Value = endpoint.get(cli.code).await?;
            log::debug!("Got: {:?}", response);
            serde_json::to_string(&response)?
        }
    };
    match cli.output {
        Some(path) => {
            // Try to create file before we do any network requests
            let mut file =
                std::fs::File::create(&path).map_err(|err| io_error_to_string(&err, &path))?;
            write!(file, "{output}").map_err(|err| io_error_to_string(&err, &path))?;
        }
        None => std::io::stdout().write_fmt(format_args!("{output}"))?,
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    env_logger::init();

    match try_main().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}
