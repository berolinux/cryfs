use anyhow::Result;
use generic_array::ArrayLength;

use crate::data::Data;
use crate::data::GrowableData;

pub trait Cipher: Sized {
    type KeySize: ArrayLength<u8>;

    fn new(key: EncryptionKey<Self::KeySize>) -> Self;

    // How many bytes is a ciphertext larger than a plaintext?
    const CIPHERTEXT_OVERHEAD: usize;

    fn encrypt<const PREFIX_BYTES: usize>(
        &self,
        data: GrowableData<PREFIX_BYTES, 0>,
    ) -> Result<GrowableData<{ PREFIX_BYTES - Self::CIPHERTEXT_OVERHEAD }, 0>>;

    fn decrypt(&self, data: Data) -> Result<Data>;
}

fn encrypt<const PrefixBytes: usize>(
    data: GrowableData<PrefixBytes, 0>,
) -> GrowableData<{ PrefixBytes - 5 }, 0> {
    todo!();
}

// TODO https://github.com/shadowsocks/crypto2 looks pretty fast, maybe we can use them for faster implementations?

mod aead_crate_wrapper;
mod aesgcm;
mod cipher_crate_wrapper;
mod key;

#[cfg(test)]
mod cipher_tests;

pub use key::EncryptionKey;

// export ciphers
pub use aesgcm::{Aes128Gcm, Aes256Gcm};
pub type XChaCha20Poly1305 = aead_crate_wrapper::AeadCipher<chacha20poly1305::XChaCha20Poly1305>;
