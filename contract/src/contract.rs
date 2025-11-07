use risc0_zkvm::Digest;
use account::address::UserAddress;
use primitives::hash::Hash32;

pub struct Contract {
    addr: UserAddress,
    elf_key: Hash32,
    image_id: Digest,
}