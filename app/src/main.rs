use clap::Parser;
use anyhow::{bail, Result};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    // 用户调用的 image_id
    #[clap(short, long, next_help_heading = "The ImageId to be invoked")]
    image_id: String,

    // 用户的输入数据
    #[clap(short, long, next_help_heading = "The input of user")]
    data: u32
}



fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match dotenvy::dotenv() {
        Ok(path) => tracing::debug!("Loaded environment variables from {:?}", path),
        Err(e) if e.not_found() => tracing::error!("No .env found"),
        Err(e) => bail!("failed to load .env file: {}", e),
    }

    let args = Args::parse();

    println!("{:?}", args);

    Ok(())

}
