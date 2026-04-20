import 'package:equatable/equatable.dart';

class SearchHistory extends Equatable {
  const SearchHistory({
    required this.id,
    required this.query,
    required this.tags,
    required this.sortBy,
    required this.filterLogic,
    required this.timestamp,
  });

  final String id;
  final String query;
  final List<String> tags;
  final String sortBy;
  final String filterLogic;
  final DateTime timestamp;

  @override
  List<Object?> get props =>
      [id, query, tags, sortBy, filterLogic, timestamp];

  Map<String, dynamic> toJson() => {
        'id': id,
        'query': query,
        'tags': tags,
        'sortBy': sortBy,
        'filterLogic': filterLogic,
        'timestamp': timestamp.toIso8601String(),
      };

  factory SearchHistory.fromJson(Map<String, dynamic> json) => SearchHistory(
        id: json['id'] as String,
        query: json['query'] as String,
        tags: List<String>.from(json['tags'] as List),
        sortBy: json['sortBy'] as String,
        filterLogic: json['filterLogic'] as String,
        timestamp: DateTime.parse(json['timestamp'] as String),
      );
}
