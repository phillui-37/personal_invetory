import '../models/failures.dart';
import '../models/resources.dart';
import '../models/result.dart';

abstract interface class NotificationRepository {
  Future<Result<List<AppNotification>, AppFailure>> listNotifications({
    bool unreadOnly = false,
  });

  Future<Result<void, AppFailure>> markRead(String id);

  /// SSE stream — yields AppNotification events as they arrive.
  Stream<AppNotification> watchNotifications();
}
