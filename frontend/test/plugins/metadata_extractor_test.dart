import 'dart:convert';
import 'package:archive/archive.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/plugins/epub_metadata_extractor.dart';
import 'package:personal_inventory_frontend/plugins/extractor_registry.dart';
import 'package:personal_inventory_frontend/plugins/metadata_extractor.dart';
import 'package:personal_inventory_frontend/plugins/mobi_metadata_extractor.dart';
import 'package:personal_inventory_frontend/plugins/pdf_metadata_extractor.dart';

void main() {
  group('ExtractorRegistry.forPath', () {
    late ExtractorRegistry registry;

    setUp(() {
      registry = ExtractorRegistry([
        PdfMetadataExtractor(),
        EpubMetadataExtractor(),
        MobiMetadataExtractor(),
      ]);
    });

    test('returns PdfMetadataExtractor for .pdf', () {
      expect(registry.forPath('book.pdf'), isA<PdfMetadataExtractor>());
    });

    test('returns EpubMetadataExtractor for .epub', () {
      expect(registry.forPath('book.epub'), isA<EpubMetadataExtractor>());
    });

    test('returns MobiMetadataExtractor for .mobi', () {
      expect(registry.forPath('book.mobi'), isA<MobiMetadataExtractor>());
    });

    test('returns MobiMetadataExtractor for .azw3', () {
      expect(registry.forPath('book.azw3'), isA<MobiMetadataExtractor>());
    });

    test('returns null for .txt', () {
      expect(registry.forPath('notes.txt'), isNull);
    });

    test('is case-insensitive for extension', () {
      expect(registry.forPath('book.PDF'), isA<PdfMetadataExtractor>());
      expect(registry.forPath('book.EPUB'), isA<EpubMetadataExtractor>());
    });
  });

  group('MobiMetadataExtractor', () {
    test('returns UnsupportedFailure for .mobi', () async {
      final extractor = MobiMetadataExtractor();
      final result = await extractor.extract('book.mobi');
      expect(result, isA<Failure<ExtractedMeta, AppFailure>>());
      final failure = (result as Failure<ExtractedMeta, AppFailure>).failure;
      expect(failure, isA<UnsupportedFailure>());
    });

    test('returns UnsupportedFailure for .azw3', () async {
      final extractor = MobiMetadataExtractor();
      final result = await extractor.extract('book.azw3');
      final failure = (result as Failure<ExtractedMeta, AppFailure>).failure;
      expect(failure, isA<UnsupportedFailure>());
    });
  });

  group('PdfMetadataExtractor', () {
    test('returns LocalFailure for invalid/nonexistent file', () async {
      final extractor = PdfMetadataExtractor();
      final result = await extractor.extract('/nonexistent/path/book.pdf');
      expect(result, isA<Failure<ExtractedMeta, AppFailure>>());
      final failure = (result as Failure<ExtractedMeta, AppFailure>).failure;
      expect(failure, isA<LocalFailure>());
    });
  });

  group('EpubMetadataExtractor', () {
    List<int> _buildMinimalEpub({
      String? title,
      String? author,
      String? isbn,
      String? publisher,
      String? language,
    }) {
      final opfContent = '''<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    ${title != null ? '<dc:title>$title</dc:title>' : ''}
    ${author != null ? '<dc:creator>$author</dc:creator>' : ''}
    ${isbn != null ? '<dc:identifier opf:scheme="ISBN">$isbn</dc:identifier>' : ''}
    ${publisher != null ? '<dc:publisher>$publisher</dc:publisher>' : ''}
    ${language != null ? '<dc:language>$language</dc:language>' : ''}
  </metadata>
</package>''';

      const containerContent = '''<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>''';

      final archive = Archive();
      archive.addFile(ArchiveFile(
        'META-INF/container.xml',
        containerContent.length,
        utf8.encode(containerContent),
      ));
      archive.addFile(ArchiveFile(
        'OEBPS/content.opf',
        opfContent.length,
        utf8.encode(opfContent),
      ));

      return ZipEncoder().encode(archive)!;
    }

    test('returns LocalFailure for nonexistent file', () async {
      final extractor = EpubMetadataExtractor();
      final result = await extractor.extract('/nonexistent/path/book.epub');
      expect(result, isA<Failure<ExtractedMeta, AppFailure>>());
      final failure = (result as Failure<ExtractedMeta, AppFailure>).failure;
      expect(failure, isA<LocalFailure>());
    });

    test('extracts title, author, isbn, publisher, language from minimal EPUB', () async {
      final epubBytes = _buildMinimalEpub(
        title: 'Test Book',
        author: 'Test Author',
        isbn: '978-3-16-148410-0',
        publisher: 'Test Publisher',
        language: 'en',
      );

      final extractor = EpubMetadataExtractor();
      final result = await extractor.extractBytes(epubBytes);

      expect(result, isA<Success<ExtractedMeta, AppFailure>>());
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, 'Test Book');
      expect(meta.author, 'Test Author');
      expect(meta.isbn, '978-3-16-148410-0');
      expect(meta.publisher, 'Test Publisher');
      expect(meta.language, 'en');
      expect(meta.fileFormat, 'epub');
    });

    test('returns null fields for missing metadata', () async {
      final epubBytes = _buildMinimalEpub();
      final extractor = EpubMetadataExtractor();
      final result = await extractor.extractBytes(epubBytes);
      expect(result, isA<Success<ExtractedMeta, AppFailure>>());
      final meta = (result as Success<ExtractedMeta, AppFailure>).value;
      expect(meta.title, isNull);
      expect(meta.author, isNull);
      expect(meta.isbn, isNull);
    });
  });
}
