import '../models/failures.dart';

String appFailureMessage(AppFailure failure) {
  return switch (failure) {
    NetworkFailure(:final message) => 'Network error: $message',
    NotFoundFailure(:final id) => 'Not found: $id',
    ValidationFailure(:final field, :final message) => '$field: $message',
    ServerFailure(:final statusCode) => 'Server error: $statusCode',
    LocalFailure(:final message) => 'Local error: $message',
    UnsupportedFailure(:final message) => 'Unsupported: $message',
  };
}
