import 'package:equatable/equatable.dart';

enum ResourceType {
  ebook,
  webReader,
}

enum StorageType {
  localFs,
  nas,
  platform,
  portable,
}

final class Resource extends Equatable {
  const Resource({
    required this.id,
    required this.title,
    required this.resourceType,
    this.notes,
  });

  final String id;
  final String title;
  final ResourceType resourceType;
  final String? notes;

  @override
  List<Object?> get props => [id, title, resourceType, notes];
}

final class EbookMeta extends Equatable {
  const EbookMeta({
    required this.resourceId,
    this.author,
    this.fileFormat,
    this.totalPages,
    this.currentPage,
  });

  final String resourceId;
  final String? author;
  final String? fileFormat;
  final int? totalPages;
  final int? currentPage;

  @override
  List<Object?> get props => [resourceId, author, fileFormat, totalPages, currentPage];
}

final class WebReaderMeta extends Equatable {
  const WebReaderMeta({
    required this.resourceId,
    required this.url,
    this.siteName,
    this.lastReadChapter,
    this.progress,
  });

  final String resourceId;
  final String url;
  final String? siteName;
  final String? lastReadChapter;
  final double? progress;

  @override
  List<Object?> get props => [resourceId, url, siteName, lastReadChapter, progress];
}

final class ResourceLocation extends Equatable {
  const ResourceLocation({
    required this.id,
    required this.resourceId,
    required this.deviceId,
    required this.pathOrUrl,
    required this.storageType,
  });

  final String id;
  final String resourceId;
  final String deviceId;
  final String pathOrUrl;
  final StorageType storageType;

  @override
  List<Object?> get props => [id, resourceId, deviceId, pathOrUrl, storageType];
}

final class EbookDetail extends Equatable {
  const EbookDetail({
    required this.resource,
    required this.meta,
    required this.locations,
  });

  final Resource resource;
  final EbookMeta meta;
  final List<ResourceLocation> locations;

  @override
  List<Object?> get props => [resource, meta, locations];
}

final class WebReaderDetail extends Equatable {
  const WebReaderDetail({
    required this.resource,
    required this.meta,
    required this.locations,
  });

  final Resource resource;
  final WebReaderMeta meta;
  final List<ResourceLocation> locations;

  @override
  List<Object?> get props => [resource, meta, locations];
}

final class ChapterCheck extends Equatable {
  const ChapterCheck({
    required this.id,
    required this.resourceId,
    required this.hasNewChapter,
    this.latestChapter,
    required this.checkedAt,
    this.errorMessage,
  });

  final String id;
  final String resourceId;
  final bool hasNewChapter;
  final String? latestChapter;
  final DateTime checkedAt;
  final String? errorMessage;

  factory ChapterCheck.fromJson(Map<String, dynamic> json) => ChapterCheck(
    id: json['id'] as String,
    resourceId: json['resource_id'] as String,
    hasNewChapter: json['has_new_chapter'] as bool,
    latestChapter: json['latest_chapter'] as String?,
    checkedAt: DateTime.parse(json['checked_at'] as String),
    errorMessage: json['error_message'] as String?,
  );

  @override
  List<Object?> get props =>
      [id, resourceId, hasNewChapter, latestChapter, checkedAt, errorMessage];
}

final class AppNotification extends Equatable {
  const AppNotification({
    required this.id,
    required this.resourceId,
    required this.message,
    required this.createdAt,
    required this.read,
  });

  final String id;
  final String resourceId;
  final String message;
  final DateTime createdAt;
  final bool read;

  factory AppNotification.fromJson(Map<String, dynamic> json) => AppNotification(
    id: json['id'] as String,
    resourceId: json['resource_id'] as String,
    message: json['message'] as String,
    createdAt: DateTime.parse(json['created_at'] as String),
    read: json['read'] as bool,
  );

  @override
  List<Object?> get props => [id, resourceId, message, createdAt, read];
}
