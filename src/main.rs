use clap::Parser;
use postgang::bring_client::mailbox_delivery_dates::{ApiKey, Endpoint};
use postgang::calendar::to_calendar;
use postgang::{DeliveryDateProvider, PostalCode, PostalCodeError};
use reqwest::header::{HeaderValue, InvalidHeaderValue};

fn postal_code_parser(value: &str) -> Result<PostalCode, PostalCodeError> {
    PostalCode::try_from(value)
}

fn parse_secret(value: &str) -> Result<ApiKey, InvalidHeaderValue> {
    Ok(ApiKey::new(HeaderValue::from_str(value)?))
}

fn parse_header_value(value: &str) -> Result<HeaderValue, InvalidHeaderValue> {
    HeaderValue::from_str(value)
}

#[derive(clap::Parser)]
#[clap(version = clap::crate_version!())]
struct Cli {
    #[arg(long, value_parser = postal_code_parser)]
    code: PostalCode,
    #[arg(long)]
    output: Option<std::path::PathBuf>,
    #[arg(long, env = "POSTGANG_API_UID", value_parser = parse_header_value)]
    api_uid: HeaderValue,
    #[arg(long, env = "POSTGANG_API_KEY", value_parser = parse_secret)]
    api_key: ApiKey,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();
    match Endpoint::new(cli.api_key, cli.api_uid).get(&cli.code) {
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
