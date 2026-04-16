class AppConfig {
  const AppConfig({
    required this.baseUrl,
    required this.apiKey,
  });

  final String baseUrl;
  final String apiKey;

  static AppConfig fromEnvironment() {
    return const AppConfig(
      baseUrl: String.fromEnvironment('BASE_URL', defaultValue: ''),
      apiKey: String.fromEnvironment('API_KEY', defaultValue: ''),
    );
  }

  String get authorizationHeader => 'Bearer $apiKey';

  bool get isConfigured => baseUrl.isNotEmpty && apiKey.isNotEmpty;
}
