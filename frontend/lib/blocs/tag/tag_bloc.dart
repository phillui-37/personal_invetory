import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/resources.dart';
import '../../models/result.dart';
import '../../models/tag.dart';
import '../../repositories/tag_repository.dart';

sealed class TagEvent extends Equatable {
  const TagEvent();

  @override
  List<Object?> get props => [];
}

final class LoadTags extends TagEvent {
  const LoadTags();
}

final class CreateTag extends TagEvent {
  const CreateTag(this.name);

  final String name;

  @override
  List<Object?> get props => [name];
}

final class DeleteTag extends TagEvent {
  const DeleteTag(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class LoadResourceTags extends TagEvent {
  const LoadResourceTags(this.resourceType, this.resourceId);

  final ResourceType resourceType;
  final String resourceId;

  @override
  List<Object?> get props => [resourceType, resourceId];
}

final class AttachTag extends TagEvent {
  const AttachTag(this.resourceType, this.resourceId, this.tagId);

  final ResourceType resourceType;
  final String resourceId;
  final String tagId;

  @override
  List<Object?> get props => [resourceType, resourceId, tagId];
}

final class DetachTag extends TagEvent {
  const DetachTag(this.resourceType, this.resourceId, this.tagId);

  final ResourceType resourceType;
  final String resourceId;
  final String tagId;

  @override
  List<Object?> get props => [resourceType, resourceId, tagId];
}

sealed class TagState extends Equatable {
  const TagState();

  @override
  List<Object?> get props => [];
}

final class TagInitial extends TagState {
  const TagInitial();
}

final class TagLoading extends TagState {
  const TagLoading();
}

final class TagListLoaded extends TagState {
  const TagListLoaded(this.tags);

  final List<Tag> tags;

  @override
  List<Object?> get props => [tags];
}

final class ResourceTagsLoaded extends TagState {
  const ResourceTagsLoaded(this.tags);

  final List<Tag> tags;

  @override
  List<Object?> get props => [tags];
}

enum TagOperationType { created, deleted, attached, detached }

final class TagOperationSuccess extends TagState {
  const TagOperationSuccess(this.operationType);

  final TagOperationType operationType;

  @override
  List<Object?> get props => [operationType];
}

final class TagError extends TagState {
  const TagError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

final class TagBloc extends Bloc<TagEvent, TagState> {
  TagBloc(this._repository) : super(const TagInitial()) {
    on<LoadTags>(_onLoadTags);
    on<CreateTag>(_onCreateTag);
    on<DeleteTag>(_onDeleteTag);
    on<LoadResourceTags>(_onLoadResourceTags);
    on<AttachTag>(_onAttachTag);
    on<DetachTag>(_onDetachTag);
  }

  final TagRepository _repository;

  Future<void> _onLoadTags(LoadTags event, Emitter<TagState> emit) async {
    emit(const TagLoading());
    final result = await _repository.listTags();
    result.when(
      success: (tags) => emit(TagListLoaded(tags)),
      failure: (failure) => emit(TagError(failure)),
    );
  }

  Future<void> _onCreateTag(CreateTag event, Emitter<TagState> emit) async {
    emit(const TagLoading());
    final result = await _repository.createTag(event.name);
    _emitOperationResult(result, emit, TagOperationType.created);
  }

  Future<void> _onDeleteTag(DeleteTag event, Emitter<TagState> emit) async {
    emit(const TagLoading());
    final result = await _repository.deleteTag(event.id);
    _emitOperationResult(result, emit, TagOperationType.deleted);
  }

  Future<void> _onLoadResourceTags(
    LoadResourceTags event,
    Emitter<TagState> emit,
  ) async {
    emit(const TagLoading());
    final result = await _repository.tagsForResource(
      event.resourceType,
      event.resourceId,
    );
    result.when(
      success: (tags) => emit(ResourceTagsLoaded(tags)),
      failure: (failure) => emit(TagError(failure)),
    );
  }

  Future<void> _onAttachTag(AttachTag event, Emitter<TagState> emit) async {
    emit(const TagLoading());
    final result = await _repository.attachTag(
      event.resourceType,
      event.resourceId,
      event.tagId,
    );
    _emitOperationResult(result, emit, TagOperationType.attached);
  }

  Future<void> _onDetachTag(DetachTag event, Emitter<TagState> emit) async {
    emit(const TagLoading());
    final result = await _repository.detachTag(
      event.resourceType,
      event.resourceId,
      event.tagId,
    );
    _emitOperationResult(result, emit, TagOperationType.detached);
  }

  void _emitOperationResult<T>(
    Result<T, AppFailure> result,
    Emitter<TagState> emit,
    TagOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(TagOperationSuccess(operationType)),
      failure: (failure) => emit(TagError(failure)),
    );
  }
}
