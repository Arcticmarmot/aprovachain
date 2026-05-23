use risc0_zkvm::{Digest, Receipt};
use account::address::ContractAddress;
use apps::ctr_io::{AccessSet, CtrInput, ReadSet};
use db::handle::DBHandle;
use platform::config::ProveMode;
use platform::rand::sample_prove_time_ms;
use primitives::hash::{sha256, Hash32};
use tx::attestation::{TxAttestation, TxAttestationWire};
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::TxPayload;
use tx::outcome::TxOutcome;
use crate::error::{EngineError, Result};
use crate::prove::{cycles_by_pre_exec, generate_receipt, generate_receipt_by_load, generate_receipt_then_save, generate_simulate_receipt};

pub fn verify_and_build_envelope(envelope_bytes: &[u8]) -> Result<TxEnvelope> {
    // 从字节数组构造 TxEnvelope
    let wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(envelope_bytes)?;
    let envelope = TxEnvelope::try_from(wire)?;
    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    envelope.self_verify()?;
    Ok(envelope)
}

pub fn verify_envelope_wire(wire: TxEnvelopeWire) -> Result<TxEnvelope> {
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
        ProveMode::NativeThenSave { scheme, enable_prove_receipt_recording, prove_receipt_csv } => {
            generate_receipt_then_save(&ctr_input, &elf, &envelope, enable_prove_receipt_recording, scheme, prove_receipt_csv)?
        }
        ProveMode::NativeByLoad { scheme, enable_prove_receipt_recording, prove_receipt_csv } => {
            generate_receipt_by_load(&ctr_input, &elf, &envelope, enable_prove_receipt_recording, scheme, prove_receipt_csv)?
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
