use risc0_zkvm::{default_prover, ExecutorEnv, Prover, Receipt};
use risc0_zkvm::sha::Digestible;
use aprova_methods::APROVA_GUEST_ELF;
use aprova_methods::APROVA_GUEST_ID;
use aprova_core::CheckResult;

fn check(s: String) -> (Receipt, String, CheckResult){
    let env = ExecutorEnv::builder()
        .write(&s)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    let receipt = prover.prove(env, APROVA_GUEST_ELF)
        .unwrap().receipt;

    let check_result: CheckResult = receipt.journal.decode().expect("Could not get the check result.");

    println!("==={:?}===", check_result);

    let input = check_result.input.clone();
    println!("{}", input);

    let result = check_result.result;
    match result {
        true => {
            println!("Aprova!")
        },
        false => {
            println!("Rollback!")
        }
    }

    (receipt, s, check_result)
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
    println!("RECEIPT DIGEST: {:?}", receipt.claim().unwrap().digest());



    let (receipt, input, output) = check("This dsafsfdsfis mydsafs secret".to_string());

    receipt.verify(APROVA_GUEST_ID).expect("Verify failed");

    println!("INPUT: {:?}", input);
    println!("OUTPUT: {:?}", output);
    println!("RECEIPT: {:?}", receipt);
    println!("RECEIPT DIGEST: {:?}", receipt.claim().unwrap().digest());


    // let (receipt, input, output) = check("This mydsafs secret".to_string());
    //
    // receipt.verify(APROVA_GUEST_ID).expect("Verify failed");
    //
    // println!("INPUT: {:?}", input);
    // println!("OUTPUT: {:?}", output);
    // println!("OUTPUT: {:?}", receipt);
    // println!("RECEIPT: {:?}", receipt.claim().unwrap().digest());
}
