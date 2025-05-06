// Test utility function skeleton

pub fn gen_metadata(desc: &str) -> [u8; 64] {
    let mut data = [0u8; 64];
    let bytes = desc.as_bytes();
    let len = bytes.len().min(64);
    data[..len].copy_from_slice(&bytes[..len]);
    data
} 