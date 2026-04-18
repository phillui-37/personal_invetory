import 'package:equatable/equatable.dart';
import '../models/failures.dart';
import '../models/result.dart';

abstract interface class MetadataExtractor {
  Set<String> get supportedExtensions;
  Future<Result<ExtractedMeta, AppFailure>> extract(String filePath);
}

final class ExtractedMeta extends Equatable {
  const ExtractedMeta({
    this.title,
    this.author,
    this.isbn,
    this.publisher,
    this.language,
    this.fileFormat,
  });

  final String? title;
  final String? author;
  final String? isbn;
  final String? publisher;
  final String? language;
  final String? fileFormat;

  @override
  List<Object?> get props => [title, author, isbn, publisher, language, fileFormat];
}
