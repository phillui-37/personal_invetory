import 'package:equatable/equatable.dart';

final class VaultStatus extends Equatable {
  const VaultStatus({required this.initialized, required this.unlocked});

  factory VaultStatus.fromJson(Map<String, dynamic> json) => VaultStatus(
    initialized: json['initialized'] as bool,
    unlocked: json['unlocked'] as bool,
  );

  final bool initialized;
  final bool unlocked;

  @override
  List<Object?> get props => [initialized, unlocked];
}

final class StoreCredentialInput {
  const StoreCredentialInput({
    required this.platform,
    required this.credentialType,
    required this.plaintext,
  });

  final String platform;
  final String credentialType;
  final String plaintext;

  Map<String, dynamic> toJson() => {
    'platform': platform,
    'credential_type': credentialType,
    'plaintext': plaintext,
  };
}

final class RetrieveCredentialInput {
  const RetrieveCredentialInput({
    required this.platform,
    required this.credentialType,
  });

  final String platform;
  final String credentialType;

  Map<String, dynamic> toJson() => {
    'platform': platform,
    'credential_type': credentialType,
  };
}
