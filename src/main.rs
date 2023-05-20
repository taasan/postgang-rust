use clap::Parser;
use postgang::bring_client::mailbox_delivery_dates::Endpoint;
use postgang::calendar::to_calendar;
use postgang::{DeliveryDateProvider, PostalCode, PostalCodeError};

fn postal_code_parser(value: &str) -> Result<PostalCode, PostalCodeError> {
    PostalCode::try_from(value)
}

#[derive(clap::Parser)]
#[clap(version = clap::crate_version!())]
struct Cli {
    #[arg(long, value_parser = postal_code_parser)]
    code: PostalCode,
    #[arg(long)]
    output: Option<std::path::PathBuf>,
    #[arg(long, env = "POSTGANG_API_UID")]
    api_uid: String,
    #[arg(long, env = "POSTGANG_API_KEY")]
    api_key: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match Endpoint::new(&cli.api_key, &cli.api_uid).get(&cli.code) {
        Ok(resp) => match cli.output {
            Some(path) => std::fs::write(path, to_calendar(resp))?,
            None => print!("{}", to_calendar(resp)),
        },
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1)
        }
    }

    Ok(())
}
