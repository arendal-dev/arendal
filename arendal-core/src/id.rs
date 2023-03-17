// We use SHA3-256 hashes as ids when using content-adressing
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Id {
    bytes: [u8; 32],
}

impl From<u64> for Id {
    fn from(value: u64) -> Self {
        let mut bytes = [0u8; 32];
        let le = value.to_le_bytes();
        for i in 0..le.len() {
            bytes[i] = le[i];
        }
        Id { bytes }
    }
}
