import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/image/image_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/image_repository.dart';

void main() {
  group('ImageBloc', () {
    blocTest<ImageBloc, ImageState>(
      'emits loading then list loaded when LoadImages succeeds',
      build: () => ImageBloc(_FakeImageRepository(
        onListImages: () async => const Success([
          Resource(id: 'i1', title: 'Image 1', resourceType: ResourceType.image),
        ]),
      )),
      act: (bloc) => bloc.add(const LoadImages()),
      expect: () => const [
        ImageLoading(),
        ImageListLoaded([
          Resource(id: 'i1', title: 'Image 1', resourceType: ResourceType.image),
        ]),
      ],
    );

    blocTest<ImageBloc, ImageState>(
      'emits loading then detail loaded when LoadImageDetail succeeds',
      build: () => ImageBloc(_FakeImageRepository(
        onGetImage: (_) async => const Success(
          ImageDetail(
            resource: Resource(
              id: 'i1',
              title: 'Image 1',
              resourceType: ResourceType.image,
            ),
            meta: ImageMeta(resourceId: 'i1', width: 800, height: 600),
            locations: [],
          ),
        ),
      )),
      act: (bloc) => bloc.add(const LoadImageDetail('i1')),
      expect: () => const [
        ImageLoading(),
        ImageDetailLoaded(
          ImageDetail(
            resource: Resource(
              id: 'i1',
              title: 'Image 1',
              resourceType: ResourceType.image,
            ),
            meta: ImageMeta(resourceId: 'i1', width: 800, height: 600),
            locations: [],
          ),
        ),
      ],
    );

    blocTest<ImageBloc, ImageState>(
      'emits loading then error when LoadImages fails',
      build: () => ImageBloc(_FakeImageRepository(
        onListImages: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadImages()),
      expect: () => const [
        ImageLoading(),
        ImageError(NetworkFailure('offline')),
      ],
    );

    blocTest<ImageBloc, ImageState>(
      'emits loading then operation success when AddImage succeeds',
      build: () => ImageBloc(_FakeImageRepository(
        onAddImage: (_) async => const Success(
          Resource(id: 'i2', title: 'Image 2', resourceType: ResourceType.image),
        ),
      )),
      act: (bloc) => bloc.add(
        const AddImage(
          NewImageInput(
            resource: Resource(
              id: 'i2',
              title: 'Image 2',
              resourceType: ResourceType.image,
            ),
            meta: ImageMeta(resourceId: 'i2'),
          ),
        ),
      ),
      expect: () => const [
        ImageLoading(),
        ImageOperationSuccess(ImageOperationType.added),
      ],
    );
  });
}

final class _FakeImageRepository implements ImageRepository {
  _FakeImageRepository({
    Future<Result<List<Resource>, AppFailure>> Function()? onListImages,
    Future<Result<List<Resource>, AppFailure>> Function(String query)? onSearchImages,
    Future<Result<ImageDetail, AppFailure>> Function(String id)? onGetImage,
    Future<Result<Resource, AppFailure>> Function(NewImageInput input)? onAddImage,
    Future<Result<Resource, AppFailure>> Function(String id, UpdateImageInput input)?
        onUpdateImage,
    Future<Result<void, AppFailure>> Function(String id)? onDeleteImage,
    Future<Result<ResourceLocation, AppFailure>> Function(
      String resourceId,
      NewLocationInput input,
    )? onAddLocation,
    Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
        onRemoveLocation,
  }) : _onListImages = onListImages,
       _onSearchImages = onSearchImages,
       _onGetImage = onGetImage,
       _onAddImage = onAddImage,
       _onUpdateImage = onUpdateImage,
       _onDeleteImage = onDeleteImage,
       _onAddLocation = onAddLocation,
       _onRemoveLocation = onRemoveLocation;

  final Future<Result<List<Resource>, AppFailure>> Function()? _onListImages;
  final Future<Result<List<Resource>, AppFailure>> Function(String query)? _onSearchImages;
  final Future<Result<ImageDetail, AppFailure>> Function(String id)? _onGetImage;
  final Future<Result<Resource, AppFailure>> Function(NewImageInput input)? _onAddImage;
  final Future<Result<Resource, AppFailure>> Function(String id, UpdateImageInput input)?
      _onUpdateImage;
  final Future<Result<void, AppFailure>> Function(String id)? _onDeleteImage;
  final Future<Result<ResourceLocation, AppFailure>> Function(
    String resourceId,
    NewLocationInput input,
  )? _onAddLocation;
  final Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
      _onRemoveLocation;

  @override
  Future<Result<Resource, AppFailure>> addImage(NewImageInput input) {
    return _onAddImage?.call(input) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) {
    return _onAddLocation?.call(resourceId, input) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> deleteImage(String id) {
    return _onDeleteImage?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<ImageDetail, AppFailure>> getImage(String id) {
    return _onGetImage?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listImages({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    return _onListImages?.call() ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) {
    return _onRemoveLocation?.call(resourceId, locationId) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchImages(String query) {
    return _onSearchImages?.call(query) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<Resource, AppFailure>> updateImage(String id, UpdateImageInput input) {
    return _onUpdateImage?.call(id, input) ??
        Future.value(const Failure(ServerFailure(500)));
  }
}
