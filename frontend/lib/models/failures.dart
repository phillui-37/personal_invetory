import 'package:equatable/equatable.dart';

sealed class AppFailure extends Equatable {
  const AppFailure();
}

final class NetworkFailure extends AppFailure {
  const NetworkFailure(this.message);

  final String message;

  @override
  List<Object?> get props => [message];
}

final class NotFoundFailure extends AppFailure {
  const NotFoundFailure(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class ValidationFailure extends AppFailure {
  const ValidationFailure({
    required this.field,
    required this.message,
  });

  final String field;
  final String message;

  @override
  List<Object?> get props => [field, message];
}

final class ServerFailure extends AppFailure {
  const ServerFailure(this.statusCode);

  final int statusCode;

  @override
  List<Object?> get props => [statusCode];
}

final class LocalFailure extends AppFailure {
  const LocalFailure(this.message);

  final String message;

  @override
  List<Object?> get props => [message];
}

final class UnsupportedFailure extends AppFailure {
  const UnsupportedFailure(this.message);

  final String message;

  @override
  List<Object?> get props => [message];
}
