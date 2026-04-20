import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/repository_inputs.dart';
import '../../models/resources.dart';
import '../../models/result.dart';
import '../../repositories/image_repository.dart';

sealed class ImageEvent extends Equatable {
  const ImageEvent();

  @override
  List<Object?> get props => [];
}

final class LoadImages extends ImageEvent {
  const LoadImages({
    this.tags = const [],
    this.sortBy,
    this.sortOrder,
    this.filterLogic,
  });

  final List<String> tags;
  final String? sortBy;
  final String? sortOrder;
  final String? filterLogic;

  @override
  List<Object?> get props => [tags, sortBy, sortOrder, filterLogic];
}

final class SearchImages extends ImageEvent {
  const SearchImages(this.query);

  final String query;

  @override
  List<Object?> get props => [query];
}

final class AddImage extends ImageEvent {
  const AddImage(this.input);

  final NewImageInput input;

  @override
  List<Object?> get props => [input];
}

final class UpdateImage extends ImageEvent {
  const UpdateImage(this.id, this.input);

  final String id;
  final UpdateImageInput input;

  @override
  List<Object?> get props => [id, input];
}

final class DeleteImage extends ImageEvent {
  const DeleteImage(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class LoadImageDetail extends ImageEvent {
  const LoadImageDetail(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class AddImageLocation extends ImageEvent {
  const AddImageLocation(this.resourceId, this.input);

  final String resourceId;
  final NewLocationInput input;

  @override
  List<Object?> get props => [resourceId, input];
}

final class RemoveImageLocation extends ImageEvent {
  const RemoveImageLocation(this.resourceId, this.locationId);

  final String resourceId;
  final String locationId;

  @override
  List<Object?> get props => [resourceId, locationId];
}

sealed class ImageState extends Equatable {
  const ImageState();

  @override
  List<Object?> get props => [];
}

final class ImageInitial extends ImageState {
  const ImageInitial();
}

final class ImageLoading extends ImageState {
  const ImageLoading();
}

final class ImageListLoaded extends ImageState {
  const ImageListLoaded(this.images);

  final List<Resource> images;

  @override
  List<Object?> get props => [images];
}

final class ImageDetailLoaded extends ImageState {
  const ImageDetailLoaded(this.image);

  final ImageDetail image;

  @override
  List<Object?> get props => [image];
}

enum ImageOperationType {
  added,
  updated,
  deleted,
  locationAdded,
  locationRemoved,
}

final class ImageOperationSuccess extends ImageState {
  const ImageOperationSuccess(this.operationType);

  final ImageOperationType operationType;

  @override
  List<Object?> get props => [operationType];
}

final class ImageError extends ImageState {
  const ImageError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

final class ImageBloc extends Bloc<ImageEvent, ImageState> {
  ImageBloc(this._repository) : super(const ImageInitial()) {
    on<LoadImages>(_onLoadImages);
    on<SearchImages>(_onSearchImages);
    on<LoadImageDetail>(_onLoadImageDetail);
    on<AddImage>(_onAddImage);
    on<UpdateImage>(_onUpdateImage);
    on<DeleteImage>(_onDeleteImage);
    on<AddImageLocation>(_onAddImageLocation);
    on<RemoveImageLocation>(_onRemoveImageLocation);
  }

  final ImageRepository _repository;

  Future<void> _onLoadImages(LoadImages event, Emitter<ImageState> emit) async {
    emit(const ImageLoading());
    final result = await _repository.listImages(
      tags: event.tags,
      sortBy: event.sortBy,
      sortOrder: event.sortOrder,
      filterLogic: event.filterLogic,
    );
    result.when(
      success: (images) => emit(ImageListLoaded(images)),
      failure: (failure) => emit(ImageError(failure)),
    );
  }

  Future<void> _onSearchImages(SearchImages event, Emitter<ImageState> emit) async {
    emit(const ImageLoading());
    final result = await _repository.searchImages(event.query);
    result.when(
      success: (images) => emit(ImageListLoaded(images)),
      failure: (failure) => emit(ImageError(failure)),
    );
  }

  Future<void> _onLoadImageDetail(
    LoadImageDetail event,
    Emitter<ImageState> emit,
  ) async {
    emit(const ImageLoading());
    final result = await _repository.getImage(event.id);
    result.when(
      success: (image) => emit(ImageDetailLoaded(image)),
      failure: (failure) => emit(ImageError(failure)),
    );
  }

  Future<void> _onAddImage(AddImage event, Emitter<ImageState> emit) async {
    emit(const ImageLoading());
    final result = await _repository.addImage(event.input);
    _emitOperationResult(result, emit, ImageOperationType.added);
  }

  Future<void> _onUpdateImage(UpdateImage event, Emitter<ImageState> emit) async {
    emit(const ImageLoading());
    final result = await _repository.updateImage(event.id, event.input);
    _emitOperationResult(result, emit, ImageOperationType.updated);
  }

  Future<void> _onDeleteImage(DeleteImage event, Emitter<ImageState> emit) async {
    emit(const ImageLoading());
    final result = await _repository.deleteImage(event.id);
    _emitVoidOperationResult(result, emit, ImageOperationType.deleted);
  }

  Future<void> _onAddImageLocation(
    AddImageLocation event,
    Emitter<ImageState> emit,
  ) async {
    emit(const ImageLoading());
    final result = await _repository.addLocation(event.resourceId, event.input);
    _emitLocationOperationResult(result, emit, ImageOperationType.locationAdded);
  }

  Future<void> _onRemoveImageLocation(
    RemoveImageLocation event,
    Emitter<ImageState> emit,
  ) async {
    emit(const ImageLoading());
    final result = await _repository.removeLocation(
      event.resourceId,
      event.locationId,
    );
    _emitVoidOperationResult(result, emit, ImageOperationType.locationRemoved);
  }

  void _emitOperationResult(
    Result<Resource, AppFailure> result,
    Emitter<ImageState> emit,
    ImageOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(ImageOperationSuccess(operationType)),
      failure: (failure) => emit(ImageError(failure)),
    );
  }

  void _emitLocationOperationResult(
    Result<ResourceLocation, AppFailure> result,
    Emitter<ImageState> emit,
    ImageOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(ImageOperationSuccess(operationType)),
      failure: (failure) => emit(ImageError(failure)),
    );
  }

  void _emitVoidOperationResult(
    Result<void, AppFailure> result,
    Emitter<ImageState> emit,
    ImageOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(ImageOperationSuccess(operationType)),
      failure: (failure) => emit(ImageError(failure)),
    );
  }
}
