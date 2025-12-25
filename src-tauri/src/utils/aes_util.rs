use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use cbc::{Decryptor, Encryptor};

type Aes128CbcEnc = Encryptor<aes::Aes128>;
type Aes128CbcDec = Decryptor<aes::Aes128>;

pub fn aes_encrypt(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let pt_len = plaintext.len();
    let mut buffer = vec![0u8; pt_len];
    buffer[..pt_len].copy_from_slice(plaintext);
    let encryptor = match Aes128CbcEnc::new_from_slices(key, iv) {
        Ok(enc) => enc,
        Err(e) => {
            eprintln!("Failed to create encryptor: {}", e);
            return vec![];
        }
    };
    let ciphertext = match encryptor.encrypt_padded_mut::<Pkcs7>(&mut buffer, pt_len) {
        Ok(ct) => ct,
        Err(e) => {
            eprintln!("Encryption failed: {}", e);
            return vec![];
        }
    };
    ciphertext.to_vec()
}

pub fn aes_decrypt(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut buffer = vec![0u8; ciphertext.len()];
    let decrypted_text = match Aes128CbcDec::new_from_slices(key, iv) {
        Ok(decryptor) => {
            buffer.copy_from_slice(&ciphertext);
            let decrypted = decryptor.decrypt_padded_mut::<Pkcs7>(&mut buffer).unwrap();
            decrypted
        }
        Err(e) => {
            eprintln!("Failed to create decryptor: {}", e);
            return vec![];
        }
    };
    decrypted_text.to_vec()
}
