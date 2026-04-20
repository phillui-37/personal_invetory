import 'dart:io';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('iOS Build Infrastructure Configuration', () {
    test('iOS Podfile exists and is properly configured', () {
      final baseDir = Directory.current;
      final podfile = File('${baseDir.path}/ios/Podfile');
      
      expect(podfile.existsSync(), isTrue, 
        reason: 'ios/Podfile must exist (run: flutter create --platforms=ios .)');
      
      final content = podfile.readAsStringSync();
      expect(content.contains('platform :ios'), isTrue, 
        reason: 'Podfile must define iOS platform');
      expect(content.contains('flutter_ios_podfile_setup'), isTrue, 
        reason: 'Podfile must call flutter_ios_podfile_setup');
      expect(content.contains('flutter_install_all_ios_pods'), isTrue, 
        reason: 'Podfile must install iOS pods');
      expect(content.contains("platform :ios, '12.0'"), isTrue, 
        reason: 'iOS minimum deployment target must be 12.0');
    });

    test('Xcode project structure exists', () {
      expect(
        Directory('${Directory.current.path}/ios/Runner.xcodeproj').existsSync(),
        isTrue,
        reason: 'ios/Runner.xcodeproj must exist',
      );
      expect(
        Directory('${Directory.current.path}/ios/Runner.xcworkspace').existsSync(),
        isTrue,
        reason: 'ios/Runner.xcworkspace must exist',
      );
    });

    test('iOS Info.plist includes network permissions', () {
      final infoPlist = File('${Directory.current.path}/ios/Runner/Info.plist');
      expect(infoPlist.existsSync(), isTrue, 
        reason: 'Info.plist must exist');
      
      final content = infoPlist.readAsStringSync();
      expect(content.contains('NSLocalNetworkUsageDescription'), isTrue, 
        reason: 'Info.plist must include NSLocalNetworkUsageDescription for network access');
    });

    test('iOS Podfile uses modular headers', () {
      final podfile = File('${Directory.current.path}/ios/Podfile');
      final content = podfile.readAsStringSync();
      expect(content.contains('use_modular_headers!'), isTrue, 
        reason: 'Podfile must use modular headers for framework compatibility');
    });
  });
}
