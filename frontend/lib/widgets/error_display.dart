import 'package:flutter/material.dart';

import '../models/failures.dart';

/// User-friendly, actionable error messages with optional retry capability
class ErrorMessage {
  final String title;
  final String description;
  final bool isRetryable;
  final String? suggestion;

  ErrorMessage({
    required this.title,
    required this.description,
    this.isRetryable = false,
    this.suggestion,
  });
}

/// Generate user-friendly error messages from AppFailure
ErrorMessage getErrorMessage(AppFailure failure) {
  return switch (failure) {
    NetworkFailure(:final message) => ErrorMessage(
      title: 'Connection Problem',
      description: message.contains('Connection refused')
          ? 'Unable to reach the server. Check your internet connection.'
          : message.contains('Connection timeout')
          ? 'The server is taking too long to respond.'
          : 'Network error occurred.',
      isRetryable: true,
      suggestion:
          'Check your internet connection and try again.',
    ),
    NotFoundFailure(:final id) => ErrorMessage(
      title: 'Resource Not Found',
      description: 'The resource "$id" could not be found.',
      isRetryable: false,
      suggestion: 'It may have been deleted or the ID is incorrect.',
    ),
    ValidationFailure(:final field, :final message) => ErrorMessage(
      title: 'Invalid Input',
      description: '$field: $message',
      isRetryable: false,
      suggestion: 'Please check your input and try again.',
    ),
    ServerFailure(:final statusCode) => ErrorMessage(
      title: 'Server Error',
      description: switch (statusCode) {
        401 => 'Authentication failed. Check your API key in vault settings.',
        403 => 'You do not have permission to perform this action.',
        429 => 'Too many requests. Please wait a moment and try again.',
        500 => 'Server encountered an internal error.',
        503 => 'Server is temporarily unavailable.',
        _ => 'Server error (HTTP $statusCode).',
      },
      isRetryable: statusCode >= 500 || statusCode == 429,
      suggestion: switch (statusCode) {
        401 => 'Go to Settings > Vault to verify your API key.',
        429 => 'Wait a few seconds before retrying.',
        500 || 503 => 'Try again in a moment.',
        _ => 'Contact support if this persists.',
      },
    ),
    LocalFailure(:final message) => ErrorMessage(
      title: 'Local Error',
      description: message.contains('permission')
          ? 'Permission denied. Check file/folder permissions.'
          : 'An error occurred while processing your request.',
      isRetryable: false,
      suggestion: message,
    ),
    UnsupportedFailure(:final message) => ErrorMessage(
      title: 'Not Supported',
      description: message.contains('file type')
          ? 'This file type is not supported.'
          : 'This operation is not supported.',
      isRetryable: false,
      suggestion: 'Try a different file or operation.',
    ),
  };
}

/// Error display widget with retry option
class ErrorDisplay extends StatelessWidget {
  final AppFailure failure;
  final VoidCallback? onRetry;
  final bool showFullDetails;

  const ErrorDisplay({
    super.key,
    required this.failure,
    this.onRetry,
    this.showFullDetails = false,
  });

  @override
  Widget build(BuildContext context) {
    final errorMessage = getErrorMessage(failure);
    final isDarkMode = Theme.of(context).brightness == Brightness.dark;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: isDarkMode
            ? Colors.red.withOpacity(0.2)
            : Colors.red.withOpacity(0.1),
        border: Border.all(
          color: Colors.red,
          width: 1,
        ),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              const Icon(Icons.error_outline, color: Colors.red),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  errorMessage.title,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    color: Colors.red,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Text(
            errorMessage.description,
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          if (errorMessage.suggestion != null) ...[
            const SizedBox(height: 8),
            Text(
              errorMessage.suggestion!,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                fontStyle: FontStyle.italic,
                color: Theme.of(context).textTheme.bodySmall?.color?.withOpacity(0.8),
              ),
            ),
          ],
          if (showFullDetails) ...[
            const SizedBox(height: 8),
            Text(
              'Technical details: ${failure.runtimeType}',
              style: Theme.of(context).textTheme.labelSmall,
            ),
          ],
          if (errorMessage.isRetryable && onRetry != null) ...[
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton.icon(
                onPressed: onRetry,
                icon: const Icon(Icons.refresh),
                label: const Text('Retry'),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

/// Compact error display for inline/snackbar usage
class CompactErrorDisplay extends StatelessWidget {
  final AppFailure failure;
  final VoidCallback? onRetry;

  const CompactErrorDisplay({
    super.key,
    required this.failure,
    this.onRetry,
  });

  @override
  Widget build(BuildContext context) {
    final errorMessage = getErrorMessage(failure);

    return Row(
      children: [
        const Icon(Icons.error_outline, color: Colors.red, size: 20),
        const SizedBox(width: 8),
        Expanded(
          child: Text(
            errorMessage.title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Colors.red,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        if (errorMessage.isRetryable && onRetry != null)
          TextButton(
            onPressed: onRetry,
            child: const Text('Retry'),
          ),
      ],
    );
  }
}
