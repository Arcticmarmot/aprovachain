use std::time::{Duration, Instant};
use risc0_zkvm::{default_executor, default_prover, Digest, ExecutorEnv, ProverOpts, Receipt};
use account::address::ContractAddress;
use apps::ctr_io::{AccessSet, CtrInput, ReadSet};
use db::handle::DBHandle;
use platform::bench::{bench_csv_path, bench_prove_receipt_csv_append};
use platform::config::ProveMode;
use platform::rand::sample_prove_time_ms;
use primitives::hash::{sha256, Hash32};
use tx::attestation::{TxAttestation, TxAttestationWire};
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::TxPayload;
use tx::outcome::TxOutcome;
use crate::error::{EngineError, Result};

pub fn verify_and_build_envelope(envelope_bytes: &[u8]) -> Result<TxEnvelope> {
    // 从字节数组构造 TxEnvelope
    let wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(envelope_bytes)?;
    let envelope = TxEnvelope::try_from(wire)?;
    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    envelope.self_verify()?;
    Ok(envelope)
}

pub fn verify_and_build_tx(tx_bytes: &[u8]) -> Result<TxAttestation> {
    let wire: TxAttestationWire = TxAttestationWire::try_decode_bcs(tx_bytes)?;
    let tx = TxAttestation::try_from(wire)?;
    tx.self_verify()?;
    Ok(tx)
}

pub fn generate_receipt(ctr_input: &CtrInput, elf: &Vec<u8>, enable_prove_receipt_recording: bool,
                        prove_scheme: String, prove_receipt_csv: String) -> Result<Receipt> {
    #[cfg(feature = "cuda")]
    tracing::info!("server: CUDA feature ENABLED (will use GPU backend if possible)");
    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
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
        let csv_name = format!("{prove_receipt_csv}-{prove_scheme}.csv");
        let prove_receipt_csv_path = bench_csv_path(&csv_name);
        bench_prove_receipt_csv_append(&prove_receipt_csv_path, cycles, prove_time, receipt_size).expect("bench csv append failed");
    }
    Ok(receipt)
}

pub fn generate_simulate_receipt(ctr_input: &CtrInput, elf: &Vec<u8>, enable_prove_receipt_recording: bool,
                                 prove_scheme: String, latency: u64, prove_receipt_csv: String) -> Result<Receipt> {
    #[cfg(feature = "cuda")]
    tracing::info!("server: CUDA feature ENABLED (will use GPU backend if possible)");
    let latency_dur = Duration::from_millis(latency);
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
        let csv_name = format!("{prove_receipt_csv}-{prove_scheme}.csv");
        let prove_receipt_csv_path = bench_csv_path(&csv_name);
        bench_prove_receipt_csv_append(&prove_receipt_csv_path, cycles, prove_time, receipt_size).expect("bench csv append failed");
    }
    Ok(receipt)
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

pub fn cycles_by_pre_exec(ctr_input: &CtrInput, elf: &Vec<u8>) -> Result<u32> {
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

pub fn build_tx_outcome(db_handle: &DBHandle, envelope: TxEnvelope, prove_mode: ProveMode) -> Result<TxOutcome> {
    let payload = &envelope.intent.payload;
    match payload {
        TxPayload::Exec { ctr_addr_str, input, access_set} => {
            let receipt = exec_tx(&db_handle, &envelope, ctr_addr_str, input, access_set, prove_mode)?;
            Ok(TxOutcome::create(envelope,  Some(receipt)))
        },
        TxPayload::Deploy{ image_id, elf, elf_hash } => {
            deploy_ctr(image_id, elf, elf_hash)?;
            Ok(TxOutcome::create(envelope,  None))
        },
        TxPayload::Update{ ctr_addr_str, image_id, elf, elf_hash } => {
            update_ctr(&db_handle, &envelope, ctr_addr_str, image_id, elf, elf_hash)?;
            Ok(TxOutcome::create(envelope,  None))
        },
    }
}

pub fn exec_tx(db_handle: &DBHandle, envelope: &TxEnvelope, ctr_addr_str: &String,
               input: &Vec<u8>, access_set: &AccessSet, prove_mode: ProveMode) -> Result<Receipt> {
    // 根据合约地址查找合约
    let intent = &envelope.intent;
    let ctr_addr = ContractAddress::parse_bech32m_with_id(intent.chain_id, ctr_addr_str)?;
    let ctr_addr_bytes = ctr_addr.to_bytes();
    let ctr = match db_handle.load_contract(&ctr_addr_bytes)? {
        Some(ctr) => ctr,
        None => return {
            tracing::warn!(target: "engine::execute", "Contract not found");
            Err(EngineError::ContractNotFound)
        }
    };
    tracing::info!(target: "engine::execute", "Contract: {:?}", ctr);

    // 从合约中获取 elf_hash
    let elf_hash = ctr.elf_hash;

    // 根据 ELF 文件哈希查找 ELF 文件
    let elf = match db_handle.load_elf(elf_hash)? {
        Some(elf) => elf,
        None => return {
            tracing::warn!(target: "engine::execute", "Elf file not found");
            Err(EngineError::ElfFileNotFound)
        }
    };
    tracing::info!("Elf file len: {}", elf.len());

    // 加载 access_set 数据
    let mut read_set: ReadSet = ReadSet::new();
    for ns_key in access_set {
        let value = db_handle.load_data_entry(ns_key)?;
        read_set.push((ns_key.clone(), value));
    }
    tracing::info!(target: "engine::execute", read_set=?read_set);
    let ctr_input = CtrInput {
        chain_id: intent.chain_id,
        input: input.clone(),
        read_set,
    };
    let receipt = match prove_mode {
        ProveMode::Native { scheme, enable_prove_receipt_recording, prove_receipt_csv } => {
            generate_receipt(&ctr_input, &elf, enable_prove_receipt_recording, scheme, prove_receipt_csv)?
        }
        ProveMode::Simulate { scheme, latency, offset, enable_prove_receipt_recording, prove_receipt_csv } => {
            let sample_latency = sample_prove_time_ms(latency, offset);
            tracing::info!(target: "engine::execute", %sample_latency);
            generate_simulate_receipt(&ctr_input, &elf, enable_prove_receipt_recording, scheme, sample_latency, prove_receipt_csv)?
        }
    };
    
    Ok(receipt)
}

pub fn pre_exec_tx(db_handle: &DBHandle, envelope: &TxEnvelope, ctr_addr_str: &String,
               input: &Vec<u8>, access_set: &AccessSet) -> Result<u32> {
    // 根据合约地址查找合约
    let intent = &envelope.intent;
    let ctr_addr = ContractAddress::parse_bech32m_with_id(intent.chain_id, ctr_addr_str)?;
    let ctr_addr_bytes = ctr_addr.to_bytes();
    let ctr = match db_handle.load_contract(&ctr_addr_bytes)? {
        Some(ctr) => ctr,
        None => return {
            tracing::warn!(target: "engine::execute", "Contract not found");
            Err(EngineError::ContractNotFound)
        }
    };
    tracing::info!(target: "engine::execute", "Contract: {:?}", ctr);

    // 从合约中获取 elf_hash
    let elf_hash = ctr.elf_hash;

    // 根据 ELF 文件哈希查找 ELF 文件
    let elf = match db_handle.load_elf(elf_hash)? {
        Some(elf) => elf,
        None => return {
            tracing::warn!(target: "engine::execute", "Elf file not found");
            Err(EngineError::ElfFileNotFound)
        }
    };
    tracing::info!("Elf file len: {}", elf.len());

    // 加载 access_set 数据
    let mut read_set: ReadSet = ReadSet::new();
    for ns_key in access_set {
        let value = db_handle.load_data_entry(ns_key)?;
        read_set.push((ns_key.clone(), value));
    }
    tracing::info!(target: "engine::execute", read_set=?read_set);
    let ctr_input = CtrInput {
        chain_id: intent.chain_id,
        input: input.clone(),
        read_set,
    };

    let scale = cycles_by_pre_exec(&ctr_input, &elf)?;
    tracing::info!(target: "engine::execute", %scale);
    // NOTE: 如果用户谎称 scale 更小，返回错误 InvalidScale
    // if u32::from(intent.scale) < total_po2 {
    //     return Err(ServerError::InvalidScale)
    // }
    Ok(scale)
}

pub fn deploy_ctr(image_id: &Digest, elf: &Vec<u8>, elf_hash: &Hash32) -> Result<()> {
    // 验证 ELF 文件哈希是否对应
    let computed_elf_hash = sha256(elf);
    if &computed_elf_hash != elf_hash {
        return Err(EngineError::ElfHashMismatch)
    }
    // 验证 image_id 是否对应
    let computed_image_id = risc0_zkvm::compute_image_id(elf).map_err(EngineError::ImageIdCompute)?;
    if &computed_image_id != image_id {
        return Err(EngineError::ImageIdMismatch)
    }
    tracing::info!(target: "engine::execute", image_id=?image_id.as_words());
    tracing::info!(target: "engine::execute", elf_hash=?elf_hash);
    Ok(())
}

pub fn update_ctr(db_handle: &DBHandle, envelope: &TxEnvelope, ctr_addr_str: &String, image_id: &Digest,
                  elf: &Vec<u8>, elf_hash: &Hash32) -> crate::error::Result<()> {
    // 根据合约地址查找合约
    let intent = &envelope.intent;
    let ctr_addr = ContractAddress::parse_bech32m_with_id(intent.chain_id, ctr_addr_str)?;
    let ctr_addr_bytes = ctr_addr.to_bytes();
    match db_handle.load_contract(&ctr_addr_bytes)? {
        Some(_) => { },
        None => return {
            tracing::warn!(target: "engine::execute", "Contract not found");
            Err(EngineError::ContractNotFound)
        }
    };
    // 验证 ELF 文件哈希是否对应
    let computed_elf_hash = sha256(elf);
    if &computed_elf_hash != elf_hash {
        return Err(EngineError::ElfHashMismatch)
    }
    // 验证 image_id 是否对应
    let computed_image_id = risc0_zkvm::compute_image_id(elf).map_err(EngineError::ImageIdCompute)?;
    if &computed_image_id != image_id {
        return Err(EngineError::ImageIdMismatch)
    }
    tracing::info!(target: "engine::execute", image_id=?image_id.as_words());
    tracing::info!(target: "engine::execute", elf_hash=?elf_hash);

    Ok(())
}
