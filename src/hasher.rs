use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

pub trait Hasher {
    fn get_hash<T: std::io::Read>(&self, reader: &mut T) -> String;
}

pub struct Md5Hasher;
pub struct Sha1Hasher;
pub struct Sha256Hasher;
pub struct Sha384Hasher;
pub struct Sha512Hasher;

#[allow(dead_code)]
const BUFFER_SIZE: usize = 4 * 1024 * 1024;

impl Hasher for Md5Hasher {
    fn get_hash<T: std::io::Read>(&self, reader: &mut T) -> String {
        let mut hasher = Md5::new();
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
        format!("{:x}", result)
    }
}

impl Hasher for Sha1Hasher {
    fn get_hash<T: std::io::Read>(&self, reader: &mut T) -> String {
        let mut hasher = Sha1::new();
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
        format!("{:x}", result)
    }
}

impl Hasher for Sha256Hasher {
    fn get_hash<T: std::io::Read>(&self, reader: &mut T) -> String {
        let mut hasher = Sha256::new();
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
        format!("{:x}", result)
    }
}

impl Hasher for Sha384Hasher {
    fn get_hash<T: std::io::Read>(&self, reader: &mut T) -> String {
        let mut hasher = Sha384::new();
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
        format!("{:x}", result)
    }
}

impl Hasher for Sha512Hasher {
    fn get_hash<T: std::io::Read>(&self, reader: &mut T) -> String {
        let mut hasher = Sha512::new();
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
        format!("{:x}", result)
    }
}
