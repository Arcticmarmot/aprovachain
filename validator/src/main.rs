use anyhow::Result;
use std::process::exit;
use clap::{arg, Parser};
use tokio::{signal, spawn};
use db::runtime::{init_db, close_db, DBFileMode};
use network::handle::{P2pCmd, P2pEvent, P2pEventHandle};
use network::runtime::{init_p2p, run_p2p};
use tokio::sync::{mpsc, watch};
use db::handle::DBHandle;
use network::behaviour::behaviour::PeerRole;
use platform::bench::{bench_csv_path, bench_validate_block_csv_begin, bench_verify_receipt_csv_begin};
use platform::config::ValidateMode;
use validator::bootstrap::{init_env, init_logging};
use validator::handle::{on_block_received};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct NodeArgs {
    #[clap(next_help_heading = "database file mode")]
    #[arg(short, long, env, value_enum)]
    db_file_mode: DBFileMode,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging()?;

    // 初始化环境变量
    init_env()?;

    // 解析 NodeArgs
    let args = NodeArgs::parse();
    let base = platform::config::load_base_config();
    let workload_config = base.workload;
    let prover_config = base.prove;
    let provers = base.provers;
    let dispatch_config = base.dispatch;
    let verify_receipt_csv = base.verify_receipt_csv;
    let enable_verify_receipt_recording = base.enable_verify_receipt_recording;
    let validate_block_csv = base.validate_block_csv;
    let enable_validate_block_recording = base.enable_validate_block_recording;

    if enable_verify_receipt_recording {
        let prove_scheme = prover_config.scheme.clone();
        let csv_name = format!("{verify_receipt_csv}-{prove_scheme}");
        let csv_path = bench_csv_path(&csv_name);
        bench_verify_receipt_csv_begin(&csv_path)?;
    }
    
    if enable_validate_block_recording {
        let prove_scheme = prover_config.scheme.clone();
        let csv_name = format!("{validate_block_csv}-{prove_scheme}");
        let csv_path = bench_csv_path(&csv_name);
        bench_validate_block_csv_begin(&csv_path)?;
    }

    let validate_mode = ValidateMode {
        prove_scheme: prover_config.scheme,
        provers,
        simulate_size: base.consensus.simulate_size,
        simulate_size_array: base.consensus.simulate_size_array,
        enable_validate_block_recording,
        validate_block_csv,
        enable_verify_receipt_recording,
        verify_receipt_csv,
    };

    // 初始化数据库
    let db_file_mode = args.db_file_mode;
    let _ = init_db(db_file_mode)?;
    let db_handle = DBHandle::new()?;
    tracing::info!(target:"verifier::init", "rocksdb({db_file_mode:?}) init success...");

    let (_p2p_cmd_tx, p2p_cmd_rx) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (p2p_event_tx, mut p2p_event_rx) =
        mpsc::unbounded_channel::<P2pEvent>();
    // let p2p_cmd_hdl = P2pCmdHandle::new(p2p_cmd_tx.clone());
    let p2p_event_hdl = P2pEventHandle::new(p2p_event_tx.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let (_sk, peer_set, swarm) = init_p2p(PeerRole::Verifier)?;
    // p2p 接收P2pCmd命令，发出P2pEvent事件
    let p2p_shutdown_rx = shutdown_rx.clone();
    let p2p_handle = spawn(async move {
        let _ = run_p2p(peer_set, swarm, p2p_cmd_rx, p2p_event_hdl, p2p_shutdown_rx).await;
    });
    tracing::info!(target:"node::init", "p2p init success...");

    loop {
        tokio::select! {
            Some(cmd) = p2p_event_rx.recv() => {
                match cmd {
                    P2pEvent::BlockReceived(block_bytes) => {
                        tracing::info!(target:"node::event", "node received block");
                        if let Err(err) = on_block_received(&db_handle, block_bytes, 
                            validate_mode.clone(), dispatch_config.clone(), workload_config.clone()).await {
                            tracing::warn!(target:"node::event::block", %err);
                        }
                    }
                    _ => { }
                }
            },

            _ = signal::ctrl_c() => {
                tracing::info!(target:"node::signal", "ctrl-c received, shutting down");
                let _ = shutdown_tx.send(true);
                let _ = p2p_handle.await;
                let _ = close_db(db_file_mode);
                exit(0);
            }
        }
    }
}
