use std::fmt::format;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use risc0_zkvm::{compute_image_id, default_executor, default_prover, ExecutorEnv, ProverOpts, Receipt};
use apps::ctr_io::CtrInput;
use chain::catalog::TxServiceCode;
use platform::bench::{bench_csv_path, bench_prove_receipt_csv_append, bench_receipt_path};
use primitives::hash::sha256;
use tx::envelope::TxEnvelope;
use crate::error::EngineError;
use crate::utils::{load_receipt_from_file, save_receipt_to_file};
use crate::error::Result;

pub fn generate_receipt(ctr_input: &CtrInput, elf: &Vec<u8>, enable_prove_receipt_recording: bool,
                        prove_scheme: String, prove_receipt_csv: String) -> Result<Receipt> {
    #[cfg(feature = "cuda")]
    tracing::info!("server: CUDA feature ENABLED (will use GPU backend if possible)");
    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .segment_limit_po2(19) // NOTE: 8GB 4060 out of memory in po2 20
        .write(&ctr_input.encode_bcs())
        .unwrap()
        .build().map_err(EngineError::ExecutorEnvBuild)?;
    let opt = match prove_scheme.as_str() {
        "succinct" => { ProverOpts::succinct() }
        "fast" => { ProverOpts::fast() }
        "groth16" => { ProverOpts::groth16() }
        _ => { return Err(EngineError::ProveScheme) }
    };
    // 根据虚拟机环境和 ELF 文件生成证明
    let prover = default_prover();

    let start_prove = Instant::now();
    let proof = prover.prove_with_opts(env, &elf, &opt).map_err(EngineError::ProofGenerate)?;
    let elapsed = Instant::now().saturating_duration_since(start_prove);
    tracing::info!(target: "engine::execute", ?elapsed, "prove time: ");
    tracing::info!(target: "engine::execute", ?proof);

    let receipt = proof.receipt;

    if enable_prove_receipt_recording {
        let cycles = proof.stats.total_cycles;
        let prove_time = elapsed.as_millis();
        let receipt_size = bcs::to_bytes(&receipt).expect("encode receipt failed").len();
        let csv_name = format!("{prove_receipt_csv}-{prove_scheme}");
        let prove_receipt_csv_path = bench_csv_path(&csv_name);
        bench_prove_receipt_csv_append(&prove_receipt_csv_path, cycles, prove_time, receipt_size).expect("bench csv append failed");
    }
    Ok(receipt)
}

pub fn generate_receipt_then_save(ctr_input: &CtrInput, elf: &Vec<u8>, envelope: &TxEnvelope, enable_prove_receipt_recording: bool,
                                  prove_scheme: String, prove_receipt_csv: String) -> Result<Receipt> {
    #[cfg(feature = "cuda")]
    tracing::info!("server: CUDA feature ENABLED (will use GPU backend if possible)");
    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .segment_limit_po2(19) // NOTE: 8GB 4060 out of memory in po2 20
        .write(&ctr_input.encode_bcs())
        .unwrap()
        .build().map_err(EngineError::ExecutorEnvBuild)?;
    let opt = match prove_scheme.as_str() {
        "succinct" => { ProverOpts::succinct() }
        "fast" => { ProverOpts::fast() }
        "groth16" => { ProverOpts::groth16() }
        _ => { return Err(EngineError::ProveScheme) }
    };
    // 根据虚拟机环境和 ELF 文件生成证明
    let prover = default_prover();

    let start_prove = Instant::now();
    let proof = prover.prove_with_opts(env, &elf, &opt).map_err(EngineError::ProofGenerate)?;
    let elapsed = Instant::now().saturating_duration_since(start_prove);
    tracing::info!(target: "engine::execute", ?elapsed, "prove time: ");
    tracing::info!(target: "engine::execute", ?proof);

    let receipt = proof.receipt;



    // 保存 receipt 到文件
    let receipt_filename = generate_receipt_filename(ctr_input, &prove_scheme);
    save_receipt_to_file(&receipt, &receipt_filename).expect("save receipt failed");

    if enable_prove_receipt_recording {
        let cycles = proof.stats.total_cycles;
        let prove_time = elapsed.as_millis();
        let receipt_size = bcs::to_bytes(&receipt).expect("encode receipt failed").len();
        let csv_name = format!("{prove_receipt_csv}-{prove_scheme}");
        let prove_receipt_csv_path = bench_csv_path(&csv_name);
        bench_prove_receipt_csv_append(&prove_receipt_csv_path, cycles, prove_time, receipt_size).expect("bench csv append failed");
    }
    Ok(receipt)
}

pub fn generate_receipt_by_load(ctr_input: &CtrInput, _elf: &Vec<u8>, _envelope: &TxEnvelope, _enable_prove_receipt_recording: bool,
                                prove_scheme: String, _prove_receipt_csv: String) -> Result<Receipt> {
    let receipt_filename = generate_receipt_filename(ctr_input, &prove_scheme);
    let start_prove = Instant::now();
    let receipt = load_receipt_from_file(&receipt_filename)
        .expect("load receipt failed");
    let elapsed = Instant::now().saturating_duration_since(start_prove);
    tracing::info!(target: "engine::execute", ?elapsed, "prove time: ");
    tracing::info!(
        target: "engine::execute::load",
        path = %receipt_filename.display(),
        "load receipt from file success"
    );

    Ok(receipt)
}

pub fn generate_simulate_receipt(ctr_input: &CtrInput, elf: &Vec<u8>, enable_prove_receipt_recording: bool,
                                 prove_scheme: String, latency: u64, prove_receipt_csv: String) -> Result<Receipt> {
    #[cfg(feature = "cuda")]
    tracing::info!("server: CUDA feature ENABLED (will use GPU backend if possible)");
    let mut latency_dur = Duration::from_millis(latency);
    let scale = cycles_by_pre_exec(&ctr_input, &elf)?;
    latency_dur *= u32::pow(2, scale - 16);
    tracing::info!(target: "engine::execute", ?latency_dur);
    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .write(&ctr_input.encode_bcs())
        .unwrap()
        .build().map_err(EngineError::ExecutorEnvBuild)?;
    // opts 里选 succinct
    let opt = match prove_scheme.as_str() {
        "succinct" => { ProverOpts::succinct() }
        "fast" => { ProverOpts::fast() }
        "groth16" => { ProverOpts::groth16() }
        _ => { return Err(EngineError::ProveScheme) }
    };    // 根据虚拟机环境和 ELF 文件生成证明
    let prover = default_prover();

    let start_prove = Instant::now();
    let proof = prover.prove_with_opts(env, &elf, &opt).map_err(EngineError::ProofGenerate)?;
    let elapsed = Instant::now().saturating_duration_since(start_prove);
    tracing::info!(target: "engine::execute", ?elapsed, "prove time: ");
    tracing::info!(target: "engine::execute", ?proof);

    let left_dur = latency_dur.saturating_sub(elapsed);
    let iters = burn_cpu_for(left_dur);
    tracing::info!(target: "engine::execute", %iters);
    let receipt = proof.receipt;
    if enable_prove_receipt_recording {
        let cycles = proof.stats.total_cycles;
        let prove_time = elapsed.as_millis();
        let receipt_size = bcs::to_bytes(&receipt).expect("encode receipt failed").len();
        let csv_name = format!("{prove_receipt_csv}-{prove_scheme}");
        let prove_receipt_csv_path = bench_csv_path(&csv_name);
        bench_prove_receipt_csv_append(&prove_receipt_csv_path, cycles, prove_time, receipt_size).expect("bench csv append failed");
    }
    Ok(receipt)
}

fn generate_receipt_filename(ctr_input: &CtrInput, prove_scheme: &String) -> PathBuf {
    let receipt_filename = hex::encode(sha256(ctr_input.encode_bcs()));
    bench_receipt_path(&format!("{prove_scheme}-{receipt_filename}"))
}

fn burn_cpu_for(dur: Duration) -> u64 {
    let start = Instant::now();
    let mut x: u64 = 0x1234_5678_9abc_def0;
    let mut iters: u64 = 0;
    while start.elapsed() < dur {
        // 一点点算术搅动，避免被优化掉
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        std::hint::black_box(x);
        iters += 1;
    }
    iters
}

pub fn cycles_by_pre_exec(ctr_input: &CtrInput, elf: &Vec<u8>) -> crate::error::Result<u32> {
    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .write(&ctr_input.encode_bcs())
        .unwrap()
        .build().map_err(EngineError::ExecutorEnvBuild)?;
    let executor = default_executor();
    let info = executor.execute(env, elf).map_err(EngineError::ExecuteElf)?;
    tracing::info!(target: "engine::execute", ?info);
    let po2_vec: Vec<u32> = info.segments.iter().map(|s| s.po2).collect();
    let mut total_cycles: u128 = 0;
    for po2 in po2_vec {
        total_cycles += 1u128 << po2;
    }
    let total_po2 = 128 - total_cycles.leading_zeros() - 1;
    if total_cycles > (1u128 << total_po2) {
        Ok(total_po2 + 1)
    } else {
        Ok(total_po2)
    }
}