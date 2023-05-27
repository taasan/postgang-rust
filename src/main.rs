use clap::Parser as ClapParser;
use git_version::git_version;
use postgang::bring_client::mailbox_delivery_dates::DeliveryDays;
use postgang::bring_client::ApiKey;
use postgang::bring_client::{InvalidPostalCode, NorwegianPostalCode};
use postgang::calendar::to_calendar_string;
use postgang::io_error_to_string;
use reqwest::header::{HeaderValue, InvalidHeaderValue};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

const VERSION: &str = git_version!(
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);

fn postal_code_parser(value: &str) -> Result<NorwegianPostalCode, InvalidPostalCode> {
    NorwegianPostalCode::try_from(value)
}

fn parse_secret(value: &str) -> Result<ApiKey, InvalidHeaderValue> {
    Ok(ApiKey::new(HeaderValue::from_str(value)?))
}

fn parse_header_value(value: &str) -> Result<HeaderValue, InvalidHeaderValue> {
    HeaderValue::from_str(value)
}

#[derive(ClapParser, Debug)]
enum Commands {
    /// Get delivery dates from Bring API
    Api {
        #[arg(long, env = "POSTGANG_API_UID", value_parser = parse_header_value, hide_env_values = true)]
        api_uid: HeaderValue,
        #[arg(long, env = "POSTGANG_API_KEY", value_parser = parse_secret, hide_env_values = true)]
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

fn try_main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
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
            write!(file, "{}", to_calendar_string(endpoint.get(&cli.code)?))
                .map_err(|err| io_error_to_string(&err, &path))?;
        }
        None => print!("{}", to_calendar_string(endpoint.get(&cli.code)?)),
    }

    Ok(())
}

fn main() -> ExitCode {
    match try_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}
