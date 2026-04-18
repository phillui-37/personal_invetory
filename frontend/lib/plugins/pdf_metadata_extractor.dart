import 'dart:io';
import 'package:syncfusion_flutter_pdf/pdf.dart';
import 'metadata_extractor.dart';
import '../models/failures.dart';
import '../models/result.dart';

final class PdfMetadataExtractor implements MetadataExtractor {
  @override
  Set<String> get supportedExtensions => {'.pdf'};

  @override
  Future<Result<ExtractedMeta, AppFailure>> extract(String filePath) async {
    try {
      final bytes = await File(filePath).readAsBytes();
      final doc = PdfDocument(inputBytes: bytes);
      final info = doc.documentInformation;
      final meta = ExtractedMeta(
        title: _nullIfEmpty(info.title),
        author: _nullIfEmpty(info.author),
        fileFormat: 'pdf',
      );
      doc.dispose();
      return Success(meta);
    } catch (e) {
      return Failure(LocalFailure('pdf extraction failed: $e'));
    }
  }

  String? _nullIfEmpty(String? s) => (s == null || s.isEmpty) ? null : s;
}
