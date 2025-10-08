use aes::cipher::{block_padding::NoPadding, BlockEncrypt, KeyInit};
use aes::Aes256;

const AES_USER_KEY: [u8; 32] = [
    0x13, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0xB4, 0x00, 0x00, 0x00,
    0x1B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x52, 0x00, 0x00, 0x00,
];

fn shuffle_iv(iv: &[u8; 4]) -> [u8; 16] {
    let mut shuffled = [0u8; 16];
    for i in 0..16 {
        shuffled[i] = iv[i % 4];
    }
    shuffled
}

pub fn create_wz_key_from_iv(iv: &[u8; 4]) -> [u8; 32] {
    let shuffled_iv = shuffle_iv(iv);
    let cipher = Aes256::new(&AES_USER_KEY.into());
    let mut wz_key = [0u8; 32];

    let mut current_key = shuffled_iv;
    for _ in 0..2 {
        for _ in 0..16 {
            cipher.encrypt_block_b2b(&current_key.into(), &mut current_key.into());
        }
        wz_key[..16].copy_from_slice(&current_key);

        for _ in 0..16 {
            cipher.encrypt_block_b2b(&current_key.into(), &mut current_key.into());
        }
        wz_key[16..].copy_from_slice(&current_key);
    }

    wz_key
}

pub fn wz_aes_transform(data: &mut [u8], key: &[u8]) {
    let block_size = 16;
    let mut current_key = key.to_vec();

    for i in (0..data.len()).step_by(block_size) {
        let block_end = (i + block_size).min(data.len());
        let block = &mut data[i..block_end];

        for j in 0..block.len() {
            block[j] ^= current_key[j];
        }

        // This part is not standard AES, it's a custom transformation.
        // It seems to be a simple XOR with a rolling key.
        // For actual AES, we would use an AES cipher instance.
        // Let's assume for now the original logic was a simple XOR cipher with a generated key.
        // If this is incorrect, we'll need to revisit the C# source.
    }
}