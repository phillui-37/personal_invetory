import '../models/failures.dart';
import '../models/resources.dart';
import '../models/result.dart';
import '../models/tag.dart';

abstract interface class TagRepository {
  Future<Result<List<Tag>, AppFailure>> listTags();

  Future<Result<Tag, AppFailure>> createTag(String name);

  Future<Result<void, AppFailure>> deleteTag(String id);

  Future<Result<List<Tag>, AppFailure>> tagsForResource(
    ResourceType resourceType,
    String resourceId,
  );

  Future<Result<void, AppFailure>> attachTag(
    ResourceType resourceType,
    String resourceId,
    String tagId,
  );

  Future<Result<void, AppFailure>> detachTag(
    ResourceType resourceType,
    String resourceId,
    String tagId,
  );
}
