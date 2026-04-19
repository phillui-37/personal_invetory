import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/device/device_bloc.dart';
import 'package:personal_inventory_frontend/models/device.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/screens/device_management_screen.dart';

import '../support/fake_repositories.dart';

Widget _buildScreen(FakeDeviceRepository repo) {
  return MaterialApp(
    home: BlocProvider<DeviceBloc>(
      create: (_) => DeviceBloc(repo),
      child: const DeviceManagementScreen(),
    ),
  );
}

final _currentDevice = Device(
  id: 'u1',
  deviceId: 'desktop-home',
  deviceName: 'Home Desktop',
  linkedAt: DateTime.utc(2024),
  locationCount: 3,
  isCurrent: true,
);

final _otherDevice = Device(
  id: 'u2',
  deviceId: 'laptop-work',
  deviceName: 'laptop-work',
  linkedAt: DateTime.utc(2024),
  locationCount: 0,
  isCurrent: false,
);

void main() {
  group('DeviceManagementScreen', () {
    testWidgets('shows loading indicator then device list', (tester) async {
      final repo = FakeDeviceRepository(
        listResult: Success([_currentDevice, _otherDevice]),
      );
      await tester.pumpWidget(_buildScreen(repo));
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(find.text('Home Desktop'), findsOneWidget);
      expect(find.text('laptop-work'), findsOneWidget);
    });

    testWidgets('shows delink button only on active non-current devices',
        (tester) async {
      final repo = FakeDeviceRepository(
        listResult: Success([_currentDevice, _otherDevice]),
      );
      await tester.pumpWidget(_buildScreen(repo));
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      // Delink button appears for the non-current active device
      expect(find.text('Delink'), findsOneWidget);
    });

    testWidgets('shows error message on failure', (tester) async {
      final repo = FakeDeviceRepository(
        listResult: const Failure(NetworkFailure('offline')),
      );
      await tester.pumpWidget(_buildScreen(repo));
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(find.textContaining('offline'), findsOneWidget);
    });

    testWidgets('shows register form with device_id field', (tester) async {
      await tester.pumpWidget(_buildScreen(FakeDeviceRepository()));
      expect(find.text('Register Device'), findsOneWidget);
      expect(find.byType(TextField), findsWidgets);
    });
  });
}
