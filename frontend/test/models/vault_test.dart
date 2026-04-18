import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/vault.dart';

void main() {
  group('VaultStatus', () {
    test('fromJson parses initialized and unlocked', () {
      final json = {'initialized': true, 'unlocked': false};
      final status = VaultStatus.fromJson(json);
      expect(status.initialized, true);
      expect(status.unlocked, false);
    });

    test('equality based on fields', () {
      const a = VaultStatus(initialized: true, unlocked: true);
      const b = VaultStatus(initialized: true, unlocked: true);
      const c = VaultStatus(initialized: false, unlocked: true);
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('StoreCredentialInput', () {
    test('toJson produces correct keys', () {
      const input = StoreCredentialInput(
        platform: 'steam',
        credentialType: 'api_key',
        plaintext: 'secret123',
      );
      expect(input.toJson(), {
        'platform': 'steam',
        'credential_type': 'api_key',
        'plaintext': 'secret123',
      });
    });
  });

  group('RetrieveCredentialInput', () {
    test('toJson produces correct keys', () {
      const input = RetrieveCredentialInput(
        platform: 'steam',
        credentialType: 'api_key',
      );
      expect(input.toJson(), {
        'platform': 'steam',
        'credential_type': 'api_key',
      });
    });
  });
}
