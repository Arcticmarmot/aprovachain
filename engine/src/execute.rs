use std::time::Instant;
use risc0_zkvm::{default_executor, default_prover, Digest, ExecutorEnv, ProverOpts, Receipt};
use account::address::ContractAddress;
use apps::ctr_io::{AccessSet, CtrInput, ReadSet};
use db::handle::DBHandle;
use primitives::hash::{sha256, Hash32};
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::TxPayload;
use tx::outcome::TxOutcome;
use crate::error::{EngineError, Result};

pub fn build_tx_envelope(envelope_bytes: &[u8]) -> Result<TxEnvelope> {
    // 从字节数组构造 TxEnvelope
    let wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(envelope_bytes)?;
    let envelope = TxEnvelope::try_from(wire)?;
    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    envelope.self_verify()?;
    Ok(envelope)
}

pub fn generate_receipt(ctr_input: &CtrInput, elf: &Vec<u8>) -> Result<Receipt> {
    #[cfg(feature = "cuda")]
    tracing::info!("server: CUDA feature ENABLED (will use GPU backend if possible)");
    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .write(&ctr_input.encode_bcs())
        .unwrap()
        .build().map_err(EngineError::ExecutorEnvBuild)?;
    // opts 里选 succinct
    let opt = ProverOpts::fast();
    // 根据虚拟机环境和 ELF 文件生成证明
    let prover = default_prover();

    let start_prove = Instant::now();
    let proof = prover.prove_with_opts(env, &elf, &opt).map_err(EngineError::ProofGenerate)?;
    let end_prove = Instant::now();
    let elapsed = end_prove - start_prove;
    tracing::info!(target: "engine::execute", ?elapsed, "prove time: ");
    tracing::info!(target: "engine::execute", ?proof);

    let receipt = proof.receipt;
    Ok(receipt)
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

pub fn build_tx_outcome(db_handle: &DBHandle, envelope: TxEnvelope) -> Result<TxOutcome> {
    let payload = &envelope.intent.payload;
    match payload {
        TxPayload::Exec { ctr_addr_str, input, access_set} => {
            let receipt = exec_tx(&db_handle, &envelope, ctr_addr_str, input, access_set)?;
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
               input: &Vec<u8>, access_set: &AccessSet) -> Result<Receipt> {
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

    // NOTE: 如果用户谎称 scale 更小，返回错误 InvalidScale
    // let total_po2 = cycles_by_pre_exec(&ctr_input, &elf)?;
    // tracing::info!(target: "engine::execute", %total_po2);
    // if u32::from(intent.scale) < total_po2 {
    //     return Err(ServerError::InvalidScale)
    // }

    let receipt = generate_receipt(&ctr_input, &elf)?;

    Ok(receipt)
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
