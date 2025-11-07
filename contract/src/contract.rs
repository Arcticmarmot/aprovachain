use risc0_zkvm::Digest;
use account::address::ChainAddress;
use primitives::hash::Hash32;

pub struct Contract {
    addr: ChainAddress,
    elf_key: Hash32,
    image_id: Digest,
}