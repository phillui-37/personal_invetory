import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';

void main() {
  test('Success.when calls success branch', () {
    const result = Success<int, AppFailure>(7);

    final output = result.when(
      success: (value) => value * 2,
      failure: (_) => -1,
    );

    expect(output, 14);
  });

  test('Failure.when calls failure branch', () {
    const result = Failure<int, AppFailure>(NetworkFailure('offline'));

    final output = result.when(
      success: (_) => 1,
      failure: (failure) => switch (failure) {
        NetworkFailure(message: final message) => message,
        _ => 'unexpected',
      },
    );

    expect(output, 'offline');
  });
}
