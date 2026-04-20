import 'package:equatable/equatable.dart';

final class Tag extends Equatable {
  const Tag({
    required this.id,
    required this.name,
    required this.createdAt,
  });

  final String id;
  final String name;
  final DateTime createdAt;

  factory Tag.fromJson(Map<String, dynamic> json) => Tag(
        id: json['id'] as String,
        name: json['name'] as String,
        createdAt: DateTime.parse(json['created_at'] as String),
      );

  Map<String, dynamic> toJson() => <String, dynamic>{
        'id': id,
        'name': name,
        'created_at': _toApiTimestamp(createdAt),
      };

  @override
  List<Object?> get props => [id, name, createdAt];

  static String _toApiTimestamp(DateTime value) =>
      value.toUtc().toIso8601String().replaceFirst('.000Z', 'Z');
}
