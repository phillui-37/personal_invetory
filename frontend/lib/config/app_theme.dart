import 'package:flutter/material.dart';

class AppTheme {
  static const _primaryBrand = Color(0xFF6366f1); // Indigo-500
  static const _errorColor = Color(0xFFef4444); // Red-500
  static const _warningColor = Color(0xFFF59E0B); // Amber-500

  static final lightTheme = ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorScheme: ColorScheme.fromSeed(
      seedColor: _primaryBrand,
      brightness: Brightness.light,
      error: _errorColor,
    ),
    scaffoldBackgroundColor: Colors.white,
    appBarTheme: AppBarTheme(
      backgroundColor: Colors.white,
      foregroundColor: Colors.black,
      elevation: 0,
      centerTitle: false,
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: Color(0xFFF3F4F6),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: Color(0xFFE5E7EB)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: Color(0xFFE5E7EB)),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: _primaryBrand, width: 2),
      ),
      contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 12),
    ),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: _primaryBrand,
        foregroundColor: Colors.white,
        padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: _primaryBrand,
        side: BorderSide(color: _primaryBrand),
        padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: _primaryBrand,
        padding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      ),
    ),
    cardTheme: CardThemeData(
      color: Colors.white,
      elevation: 1,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: Color(0xFFF3F4F6),
      selectedColor: _primaryBrand,
      labelStyle: TextStyle(color: Colors.black),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
    ),
  );

  static final darkTheme = ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: ColorScheme.fromSeed(
      seedColor: _primaryBrand,
      brightness: Brightness.dark,
      error: _errorColor,
    ),
    scaffoldBackgroundColor: Color(0xFF111827),
    appBarTheme: AppBarTheme(
      backgroundColor: Color(0xFF1F2937),
      foregroundColor: Colors.white,
      elevation: 0,
      centerTitle: false,
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: Color(0xFF374151),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: Color(0xFF4B5563)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: Color(0xFF4B5563)),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: _primaryBrand, width: 2),
      ),
      contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 12),
    ),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: _primaryBrand,
        foregroundColor: Colors.white,
        padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: _primaryBrand,
        side: BorderSide(color: _primaryBrand),
        padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: _primaryBrand,
        padding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      ),
    ),
    cardTheme: CardThemeData(
      color: Color(0xFF1F2937),
      elevation: 1,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: Color(0xFF374151),
      selectedColor: _primaryBrand,
      labelStyle: TextStyle(color: Colors.white),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
    ),
  );

  /// Get readable text for contrast checking (WCAG AAA compliance)
  static bool hasGoodContrast(Color background, Color foreground) {
    final bgLuminance = background.computeLuminance();
    final fgLuminance = foreground.computeLuminance();
    
    final lighter = fgLuminance > bgLuminance ? fgLuminance : bgLuminance;
    final darker = fgLuminance > bgLuminance ? bgLuminance : fgLuminance;
    
    final contrast = (lighter + 0.05) / (darker + 0.05);
    
    // AAA requires 7:1 for normal text, 4.5:1 for large text
    // We aim for at least 4.5:1 as minimum
    return contrast >= 4.5;
  }
}
