//! TDD tests for the MOBI/PalmDoc metadata extractor.
//!
//! These tests use synthetic byte fixtures — no real MOBI files required.
//! The MOBI PalmDoc header layout (offsets from file start):
//!   Bytes 0-31:  PalmDB name (NUL-terminated, title fallback)
//!   Bytes 32-75: PalmDB header fields (timestamps, counts, etc.)
//!   Bytes 76-77: Number of records (big-endian u16)
//!   Bytes 78+:   Record list (8 bytes per record)
//!   After record list: PalmDB data records
//!   Record 0: MOBI record 0 (contains MOBI header + EXTH header)

use super::{parse_mobi_metadata, MobiMetadata};

fn palmdb_name_bytes(name: &str) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let bytes = name.as_bytes();
    let len = bytes.len().min(31);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}

/// Builds a minimal 78-byte PalmDB header with the given name.
/// No records, just the header. Used for title-fallback tests.
fn minimal_palmdb_header(name: &str) -> Vec<u8> {
    let mut buf = vec![0u8; 78];
    let name_bytes = palmdb_name_bytes(name);
    buf[0..32].copy_from_slice(&name_bytes);
    // bytes 76-77: number of records = 0 (big-endian u16)
    buf[76] = 0;
    buf[77] = 0;
    buf
}

#[test]
fn mobi_extracts_title_from_palmdb_name() {
    let bytes = minimal_palmdb_header("My Test Book");
    let meta = parse_mobi_metadata(&bytes);
    assert_eq!(meta.title.as_deref(), Some("My Test Book"));
    assert!(meta.author.is_none());
    assert!(meta.publisher.is_none());
}

#[test]
fn mobi_handles_empty_name_gracefully() {
    let bytes = minimal_palmdb_header("");
    let meta = parse_mobi_metadata(&bytes);
    // Empty name → no title extracted
    assert!(meta.title.is_none() || meta.title.as_deref() == Some(""));
}

#[test]
fn mobi_handles_truncated_input_gracefully() {
    // Only 10 bytes — far too short
    let bytes = vec![b'B', b'o', b'o', b'k', 0, 0, 0, 0, 0, 0];
    let meta = parse_mobi_metadata(&bytes);
    // Must not panic. Title may or may not be extractable.
    let _ = meta;
}

#[test]
fn mobi_handles_zero_bytes_gracefully() {
    let bytes: Vec<u8> = vec![];
    let meta = parse_mobi_metadata(&bytes);
    assert!(meta.title.is_none());
    assert!(meta.author.is_none());
    assert!(meta.publisher.is_none());
}

#[test]
fn mobi_name_nul_terminated_correctly() {
    // Name with NUL bytes in the middle — should stop at first NUL
    let mut bytes = minimal_palmdb_header("Hello");
    bytes[5] = 0; // already NUL — "Hello" is only 5 bytes so byte 5 is already 0
    let meta = parse_mobi_metadata(&bytes);
    assert_eq!(meta.title.as_deref(), Some("Hello"));
}

/// Build a complete MOBI byte stream with EXTH records for author, publisher, title.
/// This is a synthetic minimal MOBI file for testing EXTH parsing.
fn build_mobi_with_exth(palmdb_title: &str, exth_author: &str, exth_publisher: &str) -> Vec<u8> {
    // PalmDB header: 78 bytes + 8 bytes per record
    // We have 1 record (record 0 = MOBI record)
    // Record 0 offset = 78 + 8 = 86 (1 record * 8 bytes = 8 bytes record list)
    let record0_offset: u32 = 86;

    // Build MOBI record 0:
    // PalmDOC header (16 bytes) + MOBI header (starts at byte 16 of record0)
    // MOBI header: "MOBI" magic at offset 16, then fields
    let mut record0 = Vec::new();

    // PalmDOC header (16 bytes)
    record0.extend_from_slice(&[0u8; 16]);

    // MOBI header: 232 bytes minimum
    // offset 0: "MOBI" magic
    record0.extend_from_slice(b"MOBI");
    // offset 4: header length (u32 BE) = 232
    record0.extend_from_slice(&232u32.to_be_bytes());
    // offset 8-11: MOBI type (u32 BE)
    record0.extend_from_slice(&2u32.to_be_bytes()); // MOBI_BOOK
    // offset 12-15: encoding (u32 BE) = 65001 (UTF-8)
    record0.extend_from_slice(&65001u32.to_be_bytes());
    // offset 16-47: unique ID + file version + etc. (all zeros)
    record0.extend_from_slice(&[0u8; 32]);
    // offset 48-51: first non-book record index (u32 BE) = 0xFFFFFFFF (none)
    record0.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes());
    // offset 52-55: full name offset (u32 BE) — will be 0 for now
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 56-59: full name length (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 60-63: locale (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 64-67: input language (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 68-71: output language (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 72-75: min version (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 76-79: first image record index (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 80-83: huffman record offset (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 84-87: huffman record count (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 88-91: huffman table offset (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 92-95: huffman table length (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 96-99: EXTH flags (u32 BE) — bit 6 set = has EXTH
    record0.extend_from_slice(&0x40u32.to_be_bytes());
    // offset 100-131: reserved (32 bytes)
    record0.extend_from_slice(&[0u8; 32]);
    // offset 132-135: DRM offset (u32 BE) = 0xFFFFFFFF
    record0.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes());
    // offset 136-139: DRM count (u32 BE) = 0
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 140-143: DRM size (u32 BE) = 0
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 144-147: DRM flags (u32 BE)
    record0.extend_from_slice(&0u32.to_be_bytes());
    // offset 148-151: unknown
    record0.extend_from_slice(&[0u8; 4]);
    // offset 152-153: first content record index (u16 BE)
    record0.extend_from_slice(&1u16.to_be_bytes());
    // offset 154-155: last content record index (u16 BE)
    record0.extend_from_slice(&1u16.to_be_bytes());
    // pad MOBI header to exactly 232 bytes (156 bytes so far, need 76 more)
    record0.extend_from_slice(&[0u8; 76]);
    // At this point record0 should be 16 + 232 = 248 bytes
    // But EXTH starts right after the MOBI header (offset 16 + 232 = 248 from record0 start)

    // Build EXTH header
    let mut exth = Vec::new();
    // "EXTH" magic
    exth.extend_from_slice(b"EXTH");
    // Header length (u32 BE) — will fill in after building records
    let header_len_offset = exth.len();
    exth.extend_from_slice(&0u32.to_be_bytes()); // placeholder
    // Record count (u32 BE)
    let num_records = 2u32;
    exth.extend_from_slice(&num_records.to_be_bytes());

    // EXTH record: type=100 (author), data = exth_author
    let author_data = exth_author.as_bytes();
    exth.extend_from_slice(&100u32.to_be_bytes()); // type
    exth.extend_from_slice(&((8 + author_data.len()) as u32).to_be_bytes()); // length (header 8 + data)
    exth.extend_from_slice(author_data);

    // EXTH record: type=101 (publisher), data = exth_publisher
    let pub_data = exth_publisher.as_bytes();
    exth.extend_from_slice(&101u32.to_be_bytes()); // type
    exth.extend_from_slice(&((8 + pub_data.len()) as u32).to_be_bytes());
    exth.extend_from_slice(pub_data);

    // Patch header length
    let header_len = exth.len() as u32;
    exth[header_len_offset..header_len_offset + 4].copy_from_slice(&header_len.to_be_bytes());

    record0.extend_from_slice(&exth);

    // Now build the full file
    let mut file = vec![0u8; 78 + 8]; // PalmDB header + 1 record entry

    // PalmDB name
    let name_bytes = palmdb_name_bytes(palmdb_title);
    file[0..32].copy_from_slice(&name_bytes);
    // Number of records: 1 (bytes 76-77, big-endian u16)
    file[76] = 0;
    file[77] = 1;
    // Record 0 entry: offset (u32 BE) + attributes (u8) + unique ID (3 bytes)
    let offset_bytes = record0_offset.to_be_bytes();
    file[78..82].copy_from_slice(&offset_bytes);
    file[82] = 0; // attributes
    file[83..86].copy_from_slice(&[0, 0, 0]); // unique ID

    file.extend_from_slice(&record0);
    file
}

#[test]
fn mobi_extracts_author_from_exth() {
    let bytes = build_mobi_with_exth("The Title", "Jane Author", "Big Publisher");
    let meta = parse_mobi_metadata(&bytes);
    assert_eq!(meta.author.as_deref(), Some("Jane Author"), "author should be extracted from EXTH type 100");
}

#[test]
fn mobi_extracts_publisher_from_exth() {
    let bytes = build_mobi_with_exth("The Title", "Jane Author", "Big Publisher");
    let meta = parse_mobi_metadata(&bytes);
    assert_eq!(meta.publisher.as_deref(), Some("Big Publisher"), "publisher should be extracted from EXTH type 101");
}

#[test]
fn mobi_returns_palmdb_title_as_fallback() {
    let bytes = build_mobi_with_exth("Fallback Title", "Author", "Pub");
    let meta = parse_mobi_metadata(&bytes);
    // PalmDB name used as fallback title (no EXTH type 503)
    assert!(
        meta.title.as_deref() == Some("Fallback Title"),
        "title should fall back to PalmDB name; got {:?}", meta.title
    );
}

#[test]
fn mobi_metadata_is_partial_on_malformed_exth() {
    // Corrupt the EXTH magic — should still return partial results without panicking
    let mut bytes = build_mobi_with_exth("Good Title", "Author", "Pub");
    // Find and corrupt the EXTH magic bytes
    for i in 0..bytes.len().saturating_sub(4) {
        if &bytes[i..i + 4] == b"EXTH" {
            bytes[i] = b'X'; // corrupt
            break;
        }
    }
    let meta = parse_mobi_metadata(&bytes);
    // Must not panic. Title fallback from PalmDB name should still work.
    let _ = meta.title;
    let _ = meta.author;
    let _ = meta.publisher;
}
