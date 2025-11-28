use std::fs;
use anyhow::Context;
use dotenvy::dotenv;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use platform::file::aprova_proj_dir;

pub fn init_logging() -> anyhow::Result<()> {
    // 加载环境变量以获取 RUST_LOG 信息，默认为 info
    let _ = dotenv();
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // 获取系统中 aprova 项目所有相关文件夹
    let proj_dir = aprova_proj_dir().context("resolve project dirs")?;
    let log_dir = proj_dir.data_local_dir().join("app-logs");
    fs::create_dir_all(&log_dir).context("create log dir")?;

    let file_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(rolling::daily(&log_dir, "aprova"));

    let console_layer = fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .try_init()
        .context("tracing subscriber init")?;
    Ok(())
}

pub fn init_env() -> anyhow::Result<()> {
    match dotenv() {
        Ok(path) => {
            tracing::debug!("Loaded environment variables from {:?} for app", path);
            Ok(())
        },
        Err(e) if e.not_found() => {
            tracing::warn!("No .env found for app");
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to load .env file for app: {}", e);
            Err(e).context("load .env file")
        }
    }
}