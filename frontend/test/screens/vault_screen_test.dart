import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/vault/vault_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/vault_repository.dart';
import 'package:personal_inventory_frontend/screens/vault_screen.dart';

import '../support/fake_repositories.dart';

Widget _buildTestWidget(VaultRepository repo) {
  return MaterialApp(
    home: BlocProvider<VaultBloc>(
      create: (_) => VaultBloc(repo),
      child: const VaultScreen(),
    ),
  );
}

void main() {
  group('VaultScreen', () {
    testWidgets('shows check status button initially', (tester) async {
      await tester.pumpWidget(_buildTestWidget(FakeVaultRepository()));
      expect(find.text('Check Vault Status'), findsOneWidget);
    });

    testWidgets('shows initialize button when vault not initialized',
        (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Success(
          VaultStatus(initialized: false, unlocked: false),
        ),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.text('Initialize Vault'), findsOneWidget);
    });

    testWidgets('shows unlock button when vault initialized but locked',
        (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Success(
          VaultStatus(initialized: true, unlocked: false),
        ),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.text('Unlock'), findsOneWidget);
    });

    testWidgets('shows lock button when vault unlocked', (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Success(
          VaultStatus(initialized: true, unlocked: true),
        ),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.text('Lock'), findsOneWidget);
    });

    testWidgets('shows error message on failure', (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Failure(NetworkFailure('offline')),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.textContaining('Error'), findsOneWidget);
    });
  });
}
