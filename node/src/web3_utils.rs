use sha3::{Digest, Keccak256};

pub fn checksum_address(address: &str, chain_id: Option<u32>) -> String {
    let hex_address = match chain_id {
        Some(id) => format!("{}{}", id, address.to_lowercase()),
        None => address[2..].to_lowercase(),
    };

    let mut hasher = Keccak256::new();
    hasher.update(hex_address.as_bytes());
    let hash = hasher.finalize();

    let mut address_chars: Vec<char> = match chain_id {
        Some(id) => hex_address[id.to_string().len() + 2..].chars().collect(),
        None => hex_address.chars().collect(),
    };

    for i in (0..40).step_by(2) {
        if (hash[i / 2] >> 4) >= 8 && address_chars[i].is_ascii() {
            address_chars[i] = address_chars[i].to_ascii_uppercase();
        }
        if (hash[i / 2] & 0x0f) >= 8 && address_chars[i + 1].is_ascii() {
            address_chars[i + 1] = address_chars[i + 1].to_ascii_uppercase();
        }
    }

    format!("0x{}", address_chars.iter().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_address() {
        assert_eq!(
            checksum_address("0x5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c", None),
            "0x5A0b54D5dc17e0AadC383d2db43B0a0D3E029c4c"
        );
        assert_eq!(
            checksum_address("0x5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c", Some(1)),
            "0x5A0B54d5dC17e0AAdC383d2db43b0a0d3E029C4c"
        );
        assert_eq!(
            checksum_address("0x5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c", Some(4)),
            "0x5A0B54D5dC17e0AaDC383D2DB43b0A0d3e029C4c"
        );
    }
}
