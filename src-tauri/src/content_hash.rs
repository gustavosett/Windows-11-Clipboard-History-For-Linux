//! Stable FNV-1a hashing for clipboard / GIF cache keys.
//! / هش پایدار FNV-1a برای کلیدهای کلیپ‌بورد و کش GIF.
//!
//! Uses a fixed algorithm (not `std::hash::DefaultHasher`) so cache filenames
//! remain stable across process restarts.
//! از الگوریتم ثابت استفاده می‌شود تا نام فایل‌های کش بین اجراها پایدار بماند.

use std::hash::{Hash, Hasher};

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> Self {
        FnvHasher(FNV_OFFSET_BASIS)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }
}

/// Hash `t` with FNV-1a. / هش `t` با FNV-1a.
pub fn calculate_hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut s = FnvHasher::default();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_hash() {
        assert_eq!(calculate_hash("hello"), calculate_hash("hello"));
        assert_ne!(calculate_hash("hello"), calculate_hash("world"));
    }
}
