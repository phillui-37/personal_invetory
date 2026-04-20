import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/repository_inputs.dart';
import '../../models/resources.dart';
import '../../models/result.dart';
import '../../repositories/video_repository.dart';

sealed class VideoEvent extends Equatable {
  const VideoEvent();

  @override
  List<Object?> get props => [];
}

final class LoadVideos extends VideoEvent {
  const LoadVideos({
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

final class SearchVideos extends VideoEvent {
  const SearchVideos(this.query);

  final String query;

  @override
  List<Object?> get props => [query];
}

final class AddVideo extends VideoEvent {
  const AddVideo(this.input);

  final NewVideoInput input;

  @override
  List<Object?> get props => [input];
}

final class UpdateVideo extends VideoEvent {
  const UpdateVideo(this.id, this.input);

  final String id;
  final UpdateVideoInput input;

  @override
  List<Object?> get props => [id, input];
}

final class DeleteVideo extends VideoEvent {
  const DeleteVideo(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class LoadVideoDetail extends VideoEvent {
  const LoadVideoDetail(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class AddVideoLocation extends VideoEvent {
  const AddVideoLocation(this.resourceId, this.input);

  final String resourceId;
  final NewLocationInput input;

  @override
  List<Object?> get props => [resourceId, input];
}

final class RemoveVideoLocation extends VideoEvent {
  const RemoveVideoLocation(this.resourceId, this.locationId);

  final String resourceId;
  final String locationId;

  @override
  List<Object?> get props => [resourceId, locationId];
}

sealed class VideoState extends Equatable {
  const VideoState();

  @override
  List<Object?> get props => [];
}

final class VideoInitial extends VideoState {
  const VideoInitial();
}

final class VideoLoading extends VideoState {
  const VideoLoading();
}

final class VideoListLoaded extends VideoState {
  const VideoListLoaded(this.videos);

  final List<Resource> videos;

  @override
  List<Object?> get props => [videos];
}

final class VideoDetailLoaded extends VideoState {
  const VideoDetailLoaded(this.video);

  final VideoDetail video;

  @override
  List<Object?> get props => [video];
}

enum VideoOperationType {
  added,
  updated,
  deleted,
  locationAdded,
  locationRemoved,
}

final class VideoOperationSuccess extends VideoState {
  const VideoOperationSuccess(this.operationType);

  final VideoOperationType operationType;

  @override
  List<Object?> get props => [operationType];
}

final class VideoError extends VideoState {
  const VideoError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

final class VideoBloc extends Bloc<VideoEvent, VideoState> {
  VideoBloc(this._repository) : super(const VideoInitial()) {
    on<LoadVideos>(_onLoadVideos);
    on<SearchVideos>(_onSearchVideos);
    on<LoadVideoDetail>(_onLoadVideoDetail);
    on<AddVideo>(_onAddVideo);
    on<UpdateVideo>(_onUpdateVideo);
    on<DeleteVideo>(_onDeleteVideo);
    on<AddVideoLocation>(_onAddVideoLocation);
    on<RemoveVideoLocation>(_onRemoveVideoLocation);
  }

  final VideoRepository _repository;

  Future<void> _onLoadVideos(LoadVideos event, Emitter<VideoState> emit) async {
    emit(const VideoLoading());
    final result = await _repository.listVideos();
    result.when(
      success: (videos) => emit(VideoListLoaded(videos)),
      failure: (failure) => emit(VideoError(failure)),
    );
  }

  Future<void> _onSearchVideos(SearchVideos event, Emitter<VideoState> emit) async {
    emit(const VideoLoading());
    final result = await _repository.searchVideos(event.query);
    result.when(
      success: (videos) => emit(VideoListLoaded(videos)),
      failure: (failure) => emit(VideoError(failure)),
    );
  }

  Future<void> _onLoadVideoDetail(
    LoadVideoDetail event,
    Emitter<VideoState> emit,
  ) async {
    emit(const VideoLoading());
    final result = await _repository.getVideo(event.id);
    result.when(
      success: (video) => emit(VideoDetailLoaded(video)),
      failure: (failure) => emit(VideoError(failure)),
    );
  }

  Future<void> _onAddVideo(AddVideo event, Emitter<VideoState> emit) async {
    emit(const VideoLoading());
    final result = await _repository.addVideo(event.input);
    _emitOperationResult(result, emit, VideoOperationType.added);
  }

  Future<void> _onUpdateVideo(UpdateVideo event, Emitter<VideoState> emit) async {
    emit(const VideoLoading());
    final result = await _repository.updateVideo(event.id, event.input);
    _emitOperationResult(result, emit, VideoOperationType.updated);
  }

  Future<void> _onDeleteVideo(DeleteVideo event, Emitter<VideoState> emit) async {
    emit(const VideoLoading());
    final result = await _repository.deleteVideo(event.id);
    _emitVoidOperationResult(result, emit, VideoOperationType.deleted);
  }

  Future<void> _onAddVideoLocation(
    AddVideoLocation event,
    Emitter<VideoState> emit,
  ) async {
    emit(const VideoLoading());
    final result = await _repository.addLocation(event.resourceId, event.input);
    _emitLocationOperationResult(result, emit, VideoOperationType.locationAdded);
  }

  Future<void> _onRemoveVideoLocation(
    RemoveVideoLocation event,
    Emitter<VideoState> emit,
  ) async {
    emit(const VideoLoading());
    final result = await _repository.removeLocation(
      event.resourceId,
      event.locationId,
    );
    _emitVoidOperationResult(result, emit, VideoOperationType.locationRemoved);
  }

  void _emitOperationResult(
    Result<Resource, AppFailure> result,
    Emitter<VideoState> emit,
    VideoOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(VideoOperationSuccess(operationType)),
      failure: (failure) => emit(VideoError(failure)),
    );
  }

  void _emitLocationOperationResult(
    Result<ResourceLocation, AppFailure> result,
    Emitter<VideoState> emit,
    VideoOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(VideoOperationSuccess(operationType)),
      failure: (failure) => emit(VideoError(failure)),
    );
  }

  void _emitVoidOperationResult(
    Result<void, AppFailure> result,
    Emitter<VideoState> emit,
    VideoOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(VideoOperationSuccess(operationType)),
      failure: (failure) => emit(VideoError(failure)),
    );
  }
}
