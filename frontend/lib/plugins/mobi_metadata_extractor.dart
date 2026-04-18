import 'metadata_extractor.dart';
import '../models/failures.dart';
import '../models/result.dart';

/// MOBI and AZW3 extraction is not supported in Phase 2.
/// Phase 3 will add flutter_rust_bridge integration.
final class MobiMetadataExtractor implements MetadataExtractor {
  @override
  Set<String> get supportedExtensions => {'.mobi', '.azw3'};

  @override
  Future<Result<ExtractedMeta, AppFailure>> extract(String filePath) async {
    return const Failure(UnsupportedFailure('MOBI/AZW3 metadata extraction not yet supported'));
  }
}
