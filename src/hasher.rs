use digest::Digest as _;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};
use std::io::Read;

pub trait Hasher {
    type Digest: digest::Digest;

    fn get_hash<T: Read>(&self, reader: &mut T) -> String {
        let mut hasher = Self::Digest::new();
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            let bytes_read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => panic!("Read error: {}", e),
            };
            hasher.update(&buffer[..bytes_read]);
        }
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

pub struct Md5Hasher;
pub struct Sha1Hasher;
pub struct Sha256Hasher;
pub struct Sha384Hasher;
pub struct Sha512Hasher;

const BUFFER_SIZE: usize = 4 * 1024 * 1024;

impl Hasher for Md5Hasher {
    type Digest = Md5;
}

impl Hasher for Sha1Hasher {
    type Digest = Sha1;
}

impl Hasher for Sha256Hasher {
    type Digest = Sha256;
}

impl Hasher for Sha384Hasher {
    type Digest = Sha384;
}

impl Hasher for Sha512Hasher {
    type Digest = Sha512;
}
