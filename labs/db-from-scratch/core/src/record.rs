use crate::storage::page::PageError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub id: u64,
    pub name: String,
}

/// Binary layout:
///
/// id:       u64, 8 bytes, little-endian
/// name_len: u16, 2 bytes, little-endian
/// name:     raw UTF-8 bytes
///
/// Example:
///
/// UserRecord { id: 42, name: "alice" }
///
/// becomes:
///
/// [8 bytes id][2 bytes name_len][5 bytes name]
pub fn encode_user_record(record: &UserRecord) -> Vec<u8> {
    let name_bytes = record.name.as_bytes();

    let mut bytes = Vec::with_capacity(8 + 2 + name_bytes.len());

    bytes.extend_from_slice(&record.id.to_le_bytes());
    bytes.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
    bytes.extend_from_slice(name_bytes);

    bytes
}

pub fn decode_user_record(bytes: &[u8]) -> Result<UserRecord, PageError> {
    if bytes.len() < 10 {
        return Err(PageError::CorruptPage);
    }

    let id = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);

    let name_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let name_start: usize = 10;
    let name_end = name_start
        .checked_add(name_len)
        .ok_or(PageError::CorruptPage)?;

    if name_end > bytes.len() {
        return Err(PageError::CorruptPage);
    }

    let name = String::from_utf8(bytes[name_start..name_end].to_vec())
        .map_err(|_| PageError::CorruptPage)?;

    Ok(UserRecord { id, name })
}
