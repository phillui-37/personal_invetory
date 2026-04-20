import 'dart:io' as io;
import 'dart:typed_data';

import 'metadata_extractor.dart';
import '../models/failures.dart';
import '../models/result.dart';

/// Extracts MOBI/AZW3 metadata from raw bytes.
/// Pure Dart — no native bridge. Mirrors Rust mobi.rs logic.
Future<Result<ExtractedMeta, AppFailure>> extractMobiMetadataFromBytes(
    Uint8List bytes) async {
  try {
    final meta = _parseMobiBytes(bytes);
    return Success(meta);
  } catch (_) {
    return const Success(ExtractedMeta());
  }
}

ExtractedMeta _parseMobiBytes(Uint8List bytes) {
  if (bytes.length < 78) {
    return ExtractedMeta(title: _palmDbName(bytes));
  }

  final palmName = _palmDbName(bytes);
  final numRecords = (bytes[76] << 8) | bytes[77];

  if (numRecords == 0) return ExtractedMeta(title: palmName);

  // Record 0 offset from first record list entry (bytes 78-81)
  final r0off = _u32be(bytes, 78);
  final r1off = numRecords > 1 && bytes.length >= 78 + 16
      ? _u32be(bytes, 86)
      : bytes.length;

  if (r0off >= bytes.length || r0off >= r1off) {
    return ExtractedMeta(title: palmName);
  }

  final record0 = bytes.sublist(r0off, r1off.clamp(0, bytes.length));
  return _parseRecord0(palmName, record0);
}

ExtractedMeta _parseRecord0(String? palmName, Uint8List record0) {
  const mobiStart = 16;
  if (record0.length < mobiStart + 8) return ExtractedMeta(title: palmName);

  if (record0[mobiStart] != 0x4D ||
      record0[mobiStart + 1] != 0x4F ||
      record0[mobiStart + 2] != 0x42 ||
      record0[mobiStart + 3] != 0x49) {
    return ExtractedMeta(title: palmName);
  }

  final mobiHeaderLen = _u32be(record0, mobiStart + 4);
  final exthFlagsOffset = mobiStart + 96;

  if (record0.length < exthFlagsOffset + 4) return ExtractedMeta(title: palmName);

  final exthFlags = _u32be(record0, exthFlagsOffset);
  final hasExth = (exthFlags & 0x40) != 0;

  if (!hasExth) return ExtractedMeta(title: palmName);

  final exthOffset = mobiStart + mobiHeaderLen;
  if (record0.length < exthOffset + 12) return ExtractedMeta(title: palmName);

  return _parseExth(palmName, record0.sublist(exthOffset));
}

ExtractedMeta _parseExth(String? palmName, Uint8List exth) {
  if (exth.length < 12) return ExtractedMeta(title: palmName);
  if (exth[0] != 0x45 || exth[1] != 0x58 || exth[2] != 0x54 || exth[3] != 0x48) {
    return ExtractedMeta(title: palmName);
  }

  final numRecords = _u32be(exth, 8);

  String? author;
  String? publisher;
  String? updatedTitle;

  var pos = 12;
  for (var i = 0; i < numRecords; i++) {
    if (pos + 8 > exth.length) break;
    final recType = _u32be(exth, pos);
    final recLen = _u32be(exth, pos + 4);
    if (recLen < 8 || pos + recLen > exth.length) break;

    final data = exth.sublist(pos + 8, pos + recLen);
    final text = _decodeUtf8Lossy(data).trimRight();
    final value = text.isEmpty ? null : text;

    switch (recType) {
      case 100:
        author = value;
      case 101:
        publisher = value;
      case 503:
        updatedTitle = value;
    }

    pos += recLen;
  }

  return ExtractedMeta(
    title: updatedTitle ?? palmName,
    author: author,
    publisher: publisher,
  );
}

String? _palmDbName(Uint8List bytes) {
  final end32 = bytes.length < 32 ? bytes.length : 32;
  var end = 0;
  while (end < end32 && bytes[end] != 0) end++;
  if (end == 0) return null;
  return String.fromCharCodes(bytes.sublist(0, end));
}

int _u32be(Uint8List bytes, int offset) =>
    (bytes[offset] << 24) |
    (bytes[offset + 1] << 16) |
    (bytes[offset + 2] << 8) |
    bytes[offset + 3];

String _decodeUtf8Lossy(Uint8List bytes) {
  var end = bytes.length;
  while (end > 0 && bytes[end - 1] == 0) end--;
  if (end == 0) return '';
  try {
    return String.fromCharCodes(bytes.sublist(0, end));
  } catch (_) {
    return '';
  }
}

/// MOBI and AZW3 metadata extractor — parses PalmDoc/MOBI headers in pure Dart.
final class MobiMetadataExtractor implements MetadataExtractor {
  @override
  Set<String> get supportedExtensions => {'.mobi', '.azw3'};

  @override
  Future<Result<ExtractedMeta, AppFailure>> extract(String filePath) async {
    try {
      final bytes = await io.File(filePath).readAsBytes();
      return extractMobiMetadataFromBytes(bytes);
    } catch (e) {
      return Failure(UnsupportedFailure('MOBI extraction failed: $e'));
    }
  }
}
