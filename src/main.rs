use risc0_zkvm::{default_prover, ExecutorEnv, Prover, Receipt};
use aprova_methods::APROVA_GUEST_ELF;
use aprova_methods::APROVA_GUEST_ID;
fn check(s: String) -> (Receipt, String, bool){
    let env = ExecutorEnv::builder()
        .write(&s)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    let receipt = prover.prove(env, APROVA_GUEST_ELF).unwrap().receipt;

    let check_result: bool = receipt.journal.decode().expect("Could not get the check result.");

    match check_result {
        true => {
            println!("Aprova!")
        },
        false => {
            println!("Rollback!")
        }
    }

    (receipt, s, check_result.clone())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let (receipt, input, output) = check("This is my scrat".to_string());

    receipt.verify(APROVA_GUEST_ID).expect("Verify failed");

    println!("INPUT: {:?}", input);
    println!("OUTPUT: {:?}", output);
    println!("RECEIPT: {:?}", receipt);

    let (receipt, input, output) = check("This is my secret".to_string());

    receipt.verify(APROVA_GUEST_ID).expect("Verify failed");

    println!("INPUT: {:?}", input);
    println!("OUTPUT: {:?}", output);
    println!("RECEIPT: {:?}", receipt);
}
