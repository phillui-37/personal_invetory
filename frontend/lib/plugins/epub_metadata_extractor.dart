import 'dart:convert';
import 'dart:io';
import 'package:archive/archive_io.dart';
import 'package:xml/xml.dart';
import 'metadata_extractor.dart';
import '../models/failures.dart';
import '../models/result.dart';

final class EpubMetadataExtractor implements MetadataExtractor {
  @override
  Set<String> get supportedExtensions => {'.epub'};

  @override
  Future<Result<ExtractedMeta, AppFailure>> extract(String filePath) async {
    try {
      final bytes = await File(filePath).readAsBytes();
      return extractBytes(bytes);
    } catch (e) {
      return Failure(LocalFailure('epub extraction failed: $e'));
    }
  }

  Future<Result<ExtractedMeta, AppFailure>> extractBytes(List<int> bytes) async {
    try {
      final archive = ZipDecoder().decodeBytes(bytes);

      final containerFile = archive.findFile('META-INF/container.xml');
      if (containerFile == null) {
        return const Failure(LocalFailure('not a valid EPUB: missing container.xml'));
      }
      final containerXml = XmlDocument.parse(utf8.decode(containerFile.content as List<int>));
      final rootfilePath = containerXml
          .findAllElements('rootfile')
          .firstOrNull
          ?.getAttribute('full-path');
      if (rootfilePath == null) {
        return const Failure(LocalFailure('container.xml has no rootfile'));
      }

      final opfFile = archive.findFile(rootfilePath);
      if (opfFile == null) {
        return Failure(LocalFailure('OPF file not found: $rootfilePath'));
      }
      final opfXml = XmlDocument.parse(utf8.decode(opfFile.content as List<int>));
      final metadata = opfXml.findAllElements('metadata').firstOrNull;
      if (metadata == null) {
        return const Failure(LocalFailure('no metadata element in OPF'));
      }

      String? getDcText(String name) {
        return metadata.findAllElements(name).firstOrNull?.innerText.trim()._nullIfEmpty;
      }

      return Success(ExtractedMeta(
        title: getDcText('dc:title') ?? getDcText('title'),
        author: getDcText('dc:creator') ?? getDcText('creator'),
        isbn: _extractIsbn(metadata),
        publisher: getDcText('dc:publisher') ?? getDcText('publisher'),
        language: getDcText('dc:language') ?? getDcText('language'),
        fileFormat: 'epub',
      ));
    } catch (e) {
      return Failure(LocalFailure('epub extraction failed: $e'));
    }
  }

  String? _extractIsbn(XmlElement metadata) {
    for (final id in metadata.findAllElements('dc:identifier')) {
      final scheme = id.getAttribute('opf:scheme')?.toLowerCase() ??
          id.getAttribute('scheme')?.toLowerCase() ??
          '';
      if (scheme.contains('isbn')) return id.innerText.trim()._nullIfEmpty;
    }
    return null;
  }
}

extension on String {
  String? get _nullIfEmpty => isEmpty ? null : this;
}
