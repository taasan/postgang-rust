use clap::Parser as ClapParser;
use postgang::bring_client::mailbox_delivery_dates::{ApiKey, Endpoint};
use postgang::calendar::to_calendar;
use postgang::{PostalCode, PostalCodeError};
use reqwest::header::{HeaderValue, InvalidHeaderValue};
use std::path::PathBuf;

fn postal_code_parser(value: &str) -> Result<PostalCode, PostalCodeError> {
    PostalCode::try_from(value)
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
        #[arg(long, env = "POSTGANG_API_UID", value_parser = parse_header_value)]
        api_uid: HeaderValue,
        #[arg(long, env = "POSTGANG_API_KEY", value_parser = parse_secret)]
        api_key: ApiKey,
    },
    /// Get delivery dates from JSON file
    File { input: PathBuf },
}

#[derive(ClapParser, Debug)]
#[clap(version = clap::crate_version!())]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, value_parser = postal_code_parser)]
    /// Postal code
    code: PostalCode,
    #[arg(long)]
    /// Output file
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();
    log::info!("Got CLI args: {:?}", cli);
    let endpoint = match cli.command {
        Commands::Api { api_key, api_uid } => Endpoint::api(api_key, api_uid),
        Commands::File { input } => Endpoint::file(input),
    };
    match endpoint.get(&cli.code) {
        Ok(resp) => match cli.output {
            Some(path) => std::fs::write(path, to_calendar(resp))?,
            None => print!("{}", to_calendar(resp)),
        },
        Err(err) => {
            log::error!("{err}");
            std::process::exit(1)
        }
    }

    Ok(())
}
