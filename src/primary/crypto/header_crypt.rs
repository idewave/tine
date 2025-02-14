use hmacsha::HmacSha;
use sha1::Sha1;

use super::rc4::RC4;

const ENCRYPTION_KEY: [u8; 16] = [
    0xCC, 0x98, 0xAE, 0x04, 0xE8, 0x97, 0xEA, 0xCA, 0x12, 0xDD, 0xC0, 0x93, 0x42, 0x91, 0x53, 0x57
];

const DECRYPTION_KEY: [u8; 16] = [
    0xC2, 0xB3, 0x72, 0x3C, 0xC6, 0xAE, 0xD9, 0xB5, 0x34, 0x3C, 0x53, 0xEE, 0x2F, 0x43, 0x67, 0xCE
];

#[derive(Debug)]
pub struct HeaderEncryptor {
    _instance: RC4,
}

impl HeaderEncryptor {
    pub fn new(secret: &[u8]) -> Self {
        let mut encryptor = RC4::new(
            HmacSha::new(&ENCRYPTION_KEY, secret, Sha1::default()).compute_digest().to_vec()
        );

        let _ = &encryptor.encrypt(&vec![0; 1024]);

        Self {
            _instance: encryptor,
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self._instance.encrypt(data)
    }
}

#[derive(Debug)]
pub struct HeaderDecryptor {
    _instance: RC4,
}

impl HeaderDecryptor {
    pub fn new(secret: &[u8]) -> Self {
        let mut decryptor = RC4::new(
            HmacSha::new(&DECRYPTION_KEY, secret, Sha1::default()).compute_digest().to_vec()
        );

        let _ = &decryptor.encrypt(&vec![0; 1024]);

        Self {
            _instance: decryptor,
        }
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self._instance.encrypt(data)
    }
}