use risc0_zkvm::{default_prover, ExecutorEnv, Prover, Receipt};
use risc0_zkvm::sha::Digestible;
use aprova_methods::APROVA_GUEST_ELF;
use aprova_methods::APROVA_GUEST_ID;
use aprova_core::CheckResult;

fn check() -> (Receipt, CheckResult){
    let request = include_str!("../res/uav-request.json");

    let env = ExecutorEnv::builder()
        .write(&request)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    let receipt = prover.prove(env, APROVA_GUEST_ELF)
        .unwrap().receipt;

    let check_result: CheckResult = receipt.journal.decode().expect("Could not get the check result.");

    let result = check_result.result;
    match result {
        true => {
            println!("审核结果：Aprova!")
        },
        false => {
            println!("审核结果：Rollback!")
        }
    }

    (receipt, check_result)
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let (receipt, check_result) = check();

    receipt.verify(APROVA_GUEST_ID).expect("Verify failed");

    println!("INPUT: {:?}", check_result.input);
    println!("OUTPUT: {:?}", check_result.result);
    println!("RECEIPT: {:?}", receipt);
    println!("RECEIPT DIGEST: {:?}", receipt.claim().unwrap().digest());

}
