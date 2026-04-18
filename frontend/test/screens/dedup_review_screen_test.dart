import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/dedup/dedup_bloc.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/dedup_repository.dart';
import 'package:personal_inventory_frontend/screens/dedup_review_screen.dart';

import '../support/fake_repositories.dart';

Widget _buildTestWidget(DedupRepository repo) {
  return MaterialApp(
    home: BlocProvider<DedupBloc>(
      create: (_) => DedupBloc(repo),
      child: const DedupReviewScreen(),
    ),
  );
}

void main() {
  group('DedupReviewScreen', () {
    testWidgets('shows scan button initially', (tester) async {
      await tester.pumpWidget(_buildTestWidget(FakeDedupRepository()));
      expect(find.text('Scan for Duplicates'), findsOneWidget);
    });

    testWidgets('shows warnings list after scan', (tester) async {
      const warning = DedupWarning(
        id: 'w1',
        resourceIdA: 'a1',
        resourceIdB: 'b1',
        similarityScore: 0.92,
        status: DedupWarningStatus.pending,
      );
      final repo = FakeDedupRepository(
        scanResult: const Success([warning]),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Scan for Duplicates'));
      await tester.pumpAndSettle();

      expect(find.text('92%'), findsOneWidget);
      expect(find.text('Dismiss'), findsOneWidget);
      expect(find.text('Merge'), findsOneWidget);
    });

    testWidgets('shows empty state when no warnings', (tester) async {
      final repo = FakeDedupRepository(
        scanResult: const Success([]),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Scan for Duplicates'));
      await tester.pumpAndSettle();

      expect(find.text('No duplicate warnings found'), findsOneWidget);
    });

    testWidgets('shows error message on failure', (tester) async {
      final repo = FakeDedupRepository(
        scanResult: const Failure(NetworkFailure('offline')),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Scan for Duplicates'));
      await tester.pumpAndSettle();

      expect(find.textContaining('Error'), findsOneWidget);
    });
  });
}
