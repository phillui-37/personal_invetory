import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/plugins/mobi_metadata_extractor.dart';
import 'package:personal_inventory_frontend/plugins/metadata_extractor.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/failures.dart';

void main() {
  group('MobiMetadataExtractor', () {
    test('supports .mobi and .azw3 extensions', () {
      final extractor = MobiMetadataExtractor();
      expect(extractor.supportedExtensions, containsAll(['.mobi', '.azw3']));
    });
  });

  group('MobiMetadataExtractorImpl byte parsing', () {
    test('extracts title from PalmDB name field', () async {
      final bytes = _buildMinimalPalmHeader('My Book');
      final result = await extractMobiMetadataFromBytes(bytes);
      expect(result, isA<Success<ExtractedMeta, AppFailure>>());
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, 'My Book');
      expect(meta.author, isNull);
      expect(meta.publisher, isNull);
    });

    test('returns null title when PalmDB name is empty', () async {
      final bytes = _buildMinimalPalmHeader('');
      final result = await extractMobiMetadataFromBytes(bytes);
      expect(result, isA<Success<ExtractedMeta, AppFailure>>());
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, isNull);
    });

    test('returns partial result on truncated input (fewer than 78 bytes)', () async {
      final bytes = Uint8List.fromList(List.filled(10, 0));
      final result = await extractMobiMetadataFromBytes(bytes);
      expect(result, isA<Success<ExtractedMeta, AppFailure>>());
    });

    test('returns empty meta on zero-length input', () async {
      final bytes = Uint8List(0);
      final result = await extractMobiMetadataFromBytes(bytes);
      expect(result, isA<Success<ExtractedMeta, AppFailure>>());
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, isNull);
    });

    test('stops at NUL byte in PalmDB name', () async {
      final bytes = _buildMinimalPalmHeader('Hello\x00World');
      final result = await extractMobiMetadataFromBytes(bytes);
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, 'Hello');
    });

    test('extracts author from EXTH record type 100', () async {
      final bytes = _buildMobiWithExth(
        name: 'Test Book',
        exthRecords: {100: 'Jane Author'},
      );
      final result = await extractMobiMetadataFromBytes(bytes);
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.author, 'Jane Author');
    });

    test('extracts publisher from EXTH record type 101', () async {
      final bytes = _buildMobiWithExth(
        name: 'Test Book',
        exthRecords: {101: 'Acme Publishing'},
      );
      final result = await extractMobiMetadataFromBytes(bytes);
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.publisher, 'Acme Publishing');
    });

    test('EXTH updated_title (503) overrides PalmDB name', () async {
      final bytes = _buildMobiWithExth(
        name: 'PalmDB Title',
        exthRecords: {503: 'Real Title'},
      );
      final result = await extractMobiMetadataFromBytes(bytes);
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, 'Real Title');
    });

    test('uses PalmDB name as title when no EXTH updated_title', () async {
      final bytes = _buildMobiWithExth(
        name: 'Fallback Title',
        exthRecords: {100: 'Author Name'},
      );
      final result = await extractMobiMetadataFromBytes(bytes);
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, 'Fallback Title');
      expect(meta.author, 'Author Name');
    });

    test('handles malformed EXTH gracefully — returns partial data', () async {
      // EXTH with bad record count causes early termination — must not throw
      final bytes = _buildMobiWithBadExth(name: 'Safe Title');
      final result = await extractMobiMetadataFromBytes(bytes);
      expect(result, isA<Success<ExtractedMeta, AppFailure>>());
    });
  });
}

// ------------- helpers -------------------------------------------------------

Uint8List _buildMinimalPalmHeader(String name) {
  final out = Uint8List(78);
  final nameBytes = name.codeUnits;
  for (var i = 0; i < nameBytes.length && i < 32; i++) {
    out[i] = nameBytes[i];
  }
  // num_records = 0 at bytes 76-77 — triggers early exit after name extraction
  return out;
}

/// Builds a synthetic MOBI file with the given name and EXTH records.
/// [exthRecords] maps EXTH type (u32) to UTF-8 string value.
Uint8List _buildMobiWithExth({
  required String name,
  required Map<int, String> exthRecords,
}) {
  // 1. Build EXTH block
  final exthData = BytesBuilder();
  int exthRecordCount = exthRecords.length;
  for (final entry in exthRecords.entries) {
    final data = entry.value.codeUnits;
    final recLen = 8 + data.length;
    exthData.add(_u32be(entry.key));
    exthData.add(_u32be(recLen));
    exthData.add(data);
  }
  final exthPayload = exthData.toBytes();
  final exthHeaderLen = 12 + exthPayload.length;

  final exthBlock = BytesBuilder();
  exthBlock.add([0x45, 0x58, 0x54, 0x48]); // "EXTH"
  exthBlock.add(_u32be(exthHeaderLen));
  exthBlock.add(_u32be(exthRecordCount));
  exthBlock.add(exthPayload);

  // 2. Build MOBI header (mobi_header_len bytes, must be ≥ 100 to cover EXTH flags)
  final mobiHeaderLen = 100; // covers up to and including EXTH flags field
  final mobiHeader = Uint8List(mobiHeaderLen);
  mobiHeader[0] = 0x4D; mobiHeader[1] = 0x4F; mobiHeader[2] = 0x42; mobiHeader[3] = 0x49; // "MOBI"
  final hlenBytes = _u32be(mobiHeaderLen);
  mobiHeader[4] = hlenBytes[0]; mobiHeader[5] = hlenBytes[1];
  mobiHeader[6] = hlenBytes[2]; mobiHeader[7] = hlenBytes[3];
  // Set EXTH flag (bit 6 = 0x40) at offset 96 (u32 big-endian)
  mobiHeader[96] = 0; mobiHeader[97] = 0; mobiHeader[98] = 0; mobiHeader[99] = 0x40;

  // 3. Build record 0: PalmDOC header (16 bytes) + MOBI header + EXTH
  final record0 = BytesBuilder();
  record0.add(Uint8List(16)); // PalmDOC header (zeroed)
  record0.add(mobiHeader);
  record0.add(exthBlock.toBytes());
  final record0Bytes = record0.toBytes();

  // 4. Build PalmDB wrapper
  // PalmDB header = 78 bytes; record list entry for record 0 = 8 bytes
  final palmHeaderSize = 78;
  final recordListSize = 8; // one record
  final record0Offset = palmHeaderSize + recordListSize;

  final out = BytesBuilder();
  // Name field (bytes 0-31)
  final nameBytes = Uint8List(32);
  final nb = name.codeUnits;
  for (var i = 0; i < nb.length && i < 32; i++) nameBytes[i] = nb[i];
  out.add(nameBytes);
  // Bytes 32-75: zeros (attributes, version, dates, etc.)
  out.add(Uint8List(44));
  // Bytes 76-77: num_records = 1
  out.add([0, 1]);
  // Record list entry: offset (4) + attrs (1) + uid (3)
  out.add(_u32be(record0Offset));
  out.add([0, 0, 0, 0]); // attrs + uid (4 bytes total)
  // Record 0
  out.add(record0Bytes);

  return out.toBytes();
}

Uint8List _buildMobiWithBadExth({required String name}) {
  // Like _buildMobiWithExth but EXTH record_count=999 with no actual records
  final mobiHeaderLen = 100;
  final mobiHeader = Uint8List(mobiHeaderLen);
  mobiHeader[0] = 0x4D; mobiHeader[1] = 0x4F; mobiHeader[2] = 0x42; mobiHeader[3] = 0x49;
  final hlen = _u32be(mobiHeaderLen);
  mobiHeader[4] = hlen[0]; mobiHeader[5] = hlen[1]; mobiHeader[6] = hlen[2]; mobiHeader[7] = hlen[3];
  mobiHeader[99] = 0x40; // EXTH flag

  // Bad EXTH: magic ok, record_count=999 but no records follow
  final badExth = BytesBuilder();
  badExth.add([0x45, 0x58, 0x54, 0x48]); // "EXTH"
  badExth.add(_u32be(12)); // header_len
  badExth.add(_u32be(999)); // bogus record count

  final record0 = BytesBuilder();
  record0.add(Uint8List(16));
  record0.add(mobiHeader);
  record0.add(badExth.toBytes());
  final record0Bytes = record0.toBytes();

  final palmHeaderSize = 78;
  final recordListSize = 8;
  final record0Offset = palmHeaderSize + recordListSize;

  final out = BytesBuilder();
  final nameBytes = Uint8List(32);
  final nb = name.codeUnits;
  for (var i = 0; i < nb.length && i < 32; i++) nameBytes[i] = nb[i];
  out.add(nameBytes);
  out.add(Uint8List(44));
  out.add([0, 1]);
  out.add(_u32be(record0Offset));
  out.add([0, 0, 0, 0]);
  out.add(record0Bytes);
  return out.toBytes();
}

List<int> _u32be(int v) => [
  (v >> 24) & 0xFF,
  (v >> 16) & 0xFF,
  (v >> 8) & 0xFF,
  v & 0xFF,
];
