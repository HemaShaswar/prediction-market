use anchor_lang::solana_program::hash::hash;

//helper function to hash long string into the max seed length of 32
pub fn hash_to_bytes(data: &[u8]) -> [u8; 32] {
    hash(data).to_bytes()
}
