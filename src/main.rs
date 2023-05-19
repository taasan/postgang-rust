use clap::Parser;
use postgang::calendar::to_calendar;
use postgang::DeliveryDateProvider;

#[derive(clap::Parser)]
#[clap(version = clap::crate_version!())]
struct Cli {
    #[arg(long)]
    code: String,
    #[arg(long)]
    output: Option<std::path::PathBuf>,
    #[arg(long, env = "POSTGANG_API_UID")]
    api_uid: String,
    #[arg(long, env = "POSTGANG_API_KEY")]
    api_key: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    use postgang::bring_client::mailbox_delivery_dates::Endpoint;
    let provider = Endpoint::new(cli.api_key, cli.api_uid);
    match provider.get(cli.code.into()) {
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
