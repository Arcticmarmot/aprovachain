use tx::tx_envelope::TxEnvelope;

pub struct BlockHeader {

}

pub struct Block {
    header: BlockHeader,
    txs: Vec<TxEnvelope>
}

impl Block {
    pub fn new(header: BlockHeader, txs: Vec<TxEnvelope>) -> Self {
        Self {
            header,
            txs
        }
    }
}