import 'package:flutter/material.dart';

class AppColors {
  static const Color primary = Color(0xFF76B8A6); // Teal
  static const Color primaryContainer = Color(0xFF609B8A); // Darker teal
  static const Color secondary = Color(0xFFF2A679); // Orange
  static const Color secondaryContainer = Color(0xFFDB8F63); // Darker orange
  static const Color surface = Color(0xFFD6D3D2); // Deep dark for surfaces
  static const Color error = Color(0xFFD97E64); // Warm red-pink
  static const Color onPrimary = Colors.white; // Text color on primary
  static const Color onSecondary = Colors.black; // Text on accent
  static const Color onSurface = Colors.white; // Text on dark surfaces
  static const Color onBackground = Colors.black; // Text on background
  static const Color onError = Colors.white; // Text on error background
}

class AppTheme {
  static ThemeData get themeData {
    return ThemeData(
      colorScheme: const ColorScheme(
        primary: AppColors.primary,
        primaryContainer:
            AppColors.primaryContainer, // Darker version of the primary color
        secondary: AppColors.secondary, // Accent orange
        secondaryContainer:
            AppColors.secondaryContainer, // Slightly darker orange
        surface: AppColors.surface, // Light neutral background
        error: AppColors.error, // Warm red-pink for error messages
        onPrimary: AppColors.onPrimary, // Text color on primary
        onSecondary: AppColors.onSecondary, // Text on accent
        onSurface: AppColors.onSurface, // Text on dark surfaces
        onError: AppColors.onError, // Text on error background
        brightness: Brightness.light, // Set to light theme
      ),
      buttonTheme: const ButtonThemeData(
        buttonColor: AppColors.primary, // Primary button color
        textTheme: ButtonTextTheme.primary, // Text color on buttons
      ),
      textTheme: const TextTheme(
        displayLarge: TextStyle(
            color: AppColors.primary,
            fontSize: 32,
            fontWeight: FontWeight.bold),
        bodyLarge: TextStyle(color: AppColors.primary, fontSize: 16),
      ),
    );
  }
}
