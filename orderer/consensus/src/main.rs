use tokio::sync::mpsc;
use consensus::solo::{start_consensus, InputEvent, OutputEvent};

#[tokio::main]
pub async fn main() {
    let (input_tx, input_rx) =
        mpsc::unbounded_channel::<InputEvent>();
    let (output_tx, mut output_rx) =
        mpsc::unbounded_channel::<OutputEvent>();
    
    start_consensus(input_tx, input_rx, output_tx).await;
}