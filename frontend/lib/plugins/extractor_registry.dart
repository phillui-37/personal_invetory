import 'metadata_extractor.dart';

class ExtractorRegistry {
  ExtractorRegistry(this._extractors);

  final List<MetadataExtractor> _extractors;

  MetadataExtractor? forPath(String filePath) {
    final ext = '.${filePath.toLowerCase().split('.').last}';
    return _extractors.where((e) => e.supportedExtensions.contains(ext)).firstOrNull;
  }
}
