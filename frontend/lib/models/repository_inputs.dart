import 'resources.dart';

final class NewEbookInput {
  const NewEbookInput({
    required this.resource,
    required this.meta,
  });

  final Resource resource;
  final EbookMeta meta;
}

final class UpdateEbookInput {
  const UpdateEbookInput({
    this.title,
    this.author,
    this.fileFormat,
    this.totalPages,
    this.currentPage,
  });

  final String? title;
  final String? author;
  final String? fileFormat;
  final int? totalPages;
  final int? currentPage;
}

final class NewWebReaderInput {
  const NewWebReaderInput({
    required this.resource,
    required this.meta,
  });

  final Resource resource;
  final WebReaderMeta meta;
}

final class UpdateWebReaderInput {
  const UpdateWebReaderInput({
    this.title,
    this.url,
    this.siteName,
    this.lastReadChapter,
    this.progress,
  });

  final String? title;
  final String? url;
  final String? siteName;
  final String? lastReadChapter;
  final double? progress;
}

final class NewLocationInput {
  const NewLocationInput({
    required this.deviceId,
    required this.pathOrUrl,
    required this.storageType,
  });

  final String deviceId;
  final String pathOrUrl;
  final StorageType storageType;
}

final class WebReaderProgressSignal {
  const WebReaderProgressSignal({
    required this.resourceId,
    required this.url,
    this.domChapter,
    this.domProgress,
  });

  final String resourceId;
  final String url;
  final String? domChapter;
  final double? domProgress;
}
