use risc0_zkvm::guest::env;
use apps::ctr_io::{CtrInput, CtrOutcome, CtrOutput, CtrResult, NamespaceKey, WriteSet, ReadSet, find_entry};
use primitives::hash::{sha256};
use primitives::trans::{u64_from_be_slice, u64_to_be_vec};

fn main() {
    let ctr_input_bytes: Vec<u8> = env::read();
    let input_hash = sha256(&ctr_input_bytes);
    let ctr_output: CtrOutput = match CtrInput::try_decode_bcs(&ctr_input_bytes) {
        Ok(ctr_input) => {
            let iterations = u64_from_be_slice(&ctr_input.input);
            let answer = fibonacci(iterations);
            CtrOutput {
                input_hash,
                read_set: ReadSet::new(),
                ctr_result: CtrResult::Ok {
                    outcome: CtrOutcome {
                        write_set: WriteSet::new(),
                        answer: u64_to_be_vec(answer),
                    }
                }
            }
        }
        Err(err) => {
            CtrOutput {
                input_hash,
                read_set: ReadSet::new(),
                ctr_result: CtrResult::Err {
                    message: format!("{:#}", err)
                }
            }
        }
    };
    env::commit(&ctr_output.encode_bcs());
}

fn fibonacci(n: u64) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 0..n {
        let tmp = a + b;
        a = b;
        b = tmp;
    }
    a
}