//! PalmDoc/MOBI header parser — extracts title, author, publisher from MOBI/AZW3 files.
//!
//! Parses the binary header without external crates. Gracefully returns partial
//! results on malformed input and never panics.

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MobiMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
}

/// Parse MOBI/PalmDoc bytes and extract title, author, publisher.
///
/// Title falls back to the PalmDB name field (bytes 0–31) if no EXTH type 503.
/// Returns partial results silently on malformed input.
pub fn parse_mobi_metadata(bytes: &[u8]) -> MobiMetadata {
    if bytes.len() < 78 {
        // Too short to contain even the PalmDB header + record count
        let title = extract_palmdb_name(bytes);
        return MobiMetadata { title, author: None, publisher: None };
    }

    let palmdb_name = extract_palmdb_name(bytes);
    let num_records = u16::from_be_bytes([bytes[76], bytes[77]]) as usize;

    if num_records == 0 {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    // Record 0 offset is in the first record list entry (bytes 78..82)
    let record_list_start = 78;
    let record0_entry_start = record_list_start;

    if bytes.len() < record0_entry_start + 8 {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    let record0_offset = u32::from_be_bytes([
        bytes[record0_entry_start],
        bytes[record0_entry_start + 1],
        bytes[record0_entry_start + 2],
        bytes[record0_entry_start + 3],
    ]) as usize;

    // Record 1 offset (or end-of-file) to determine record 0 length
    let record1_offset = if num_records > 1 && bytes.len() >= record_list_start + 16 {
        u32::from_be_bytes([
            bytes[record_list_start + 8],
            bytes[record_list_start + 9],
            bytes[record_list_start + 10],
            bytes[record_list_start + 11],
        ]) as usize
    } else {
        bytes.len()
    };

    if record0_offset >= bytes.len() || record0_offset >= record1_offset {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    let record0 = &bytes[record0_offset..record1_offset.min(bytes.len())];
    parse_mobi_record0(palmdb_name, record0)
}

fn parse_mobi_record0(palmdb_name: Option<String>, record0: &[u8]) -> MobiMetadata {
    // PalmDOC header = 16 bytes; MOBI header starts at offset 16
    let mobi_header_start = 16;

    if record0.len() < mobi_header_start + 8 {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    // Check MOBI magic at record0[mobi_header_start]
    if &record0[mobi_header_start..mobi_header_start + 4] != b"MOBI" {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    // MOBI header length at record0[mobi_header_start + 4]
    let mobi_header_len = u32::from_be_bytes([
        record0[mobi_header_start + 4],
        record0[mobi_header_start + 5],
        record0[mobi_header_start + 6],
        record0[mobi_header_start + 7],
    ]) as usize;

    // EXTH flags at record0[mobi_header_start + 96]
    let exth_flags_offset = mobi_header_start + 96;
    if record0.len() < exth_flags_offset + 4 {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    let exth_flags = u32::from_be_bytes([
        record0[exth_flags_offset],
        record0[exth_flags_offset + 1],
        record0[exth_flags_offset + 2],
        record0[exth_flags_offset + 3],
    ]);

    let has_exth = (exth_flags & 0x40) != 0;
    if !has_exth {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    // EXTH starts at record0[mobi_header_start + mobi_header_len]
    let exth_offset = mobi_header_start + mobi_header_len;
    if record0.len() < exth_offset + 12 {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    parse_exth(palmdb_name, &record0[exth_offset..])
}

fn parse_exth(palmdb_name: Option<String>, exth: &[u8]) -> MobiMetadata {
    if exth.len() < 12 {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    // EXTH magic
    if &exth[0..4] != b"EXTH" {
        return MobiMetadata { title: palmdb_name, author: None, publisher: None };
    }

    let _header_len = u32::from_be_bytes([exth[4], exth[5], exth[6], exth[7]]) as usize;
    let num_records = u32::from_be_bytes([exth[8], exth[9], exth[10], exth[11]]) as usize;

    let mut author: Option<String> = None;
    let mut publisher: Option<String> = None;
    let mut updated_title: Option<String> = None;

    let mut pos = 12usize;
    for _ in 0..num_records {
        if pos + 8 > exth.len() {
            break;
        }
        let rec_type = u32::from_be_bytes([exth[pos], exth[pos + 1], exth[pos + 2], exth[pos + 3]]);
        let rec_len = u32::from_be_bytes([exth[pos + 4], exth[pos + 5], exth[pos + 6], exth[pos + 7]]) as usize;

        if rec_len < 8 || pos + rec_len > exth.len() {
            break;
        }

        let data = &exth[pos + 8..pos + rec_len];
        let text = String::from_utf8_lossy(data).trim_end_matches('\0').to_string();
        let text = if text.is_empty() { None } else { Some(text) };

        match rec_type {
            100 => author = text,
            101 => publisher = text,
            503 => updated_title = text,
            _ => {}
        }

        pos += rec_len;
    }

    let title = updated_title.or(palmdb_name);
    MobiMetadata { title, author, publisher }
}

fn extract_palmdb_name(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let end = bytes[..32.min(bytes.len())]
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(32.min(bytes.len()));
    if end == 0 {
        return None;
    }
    String::from_utf8(bytes[..end].to_vec()).ok()
}

#[cfg(test)]
#[path = "mobi_test.rs"]
mod tests;
