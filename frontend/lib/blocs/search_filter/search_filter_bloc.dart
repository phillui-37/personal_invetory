import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

// Events
sealed class SearchFilterEvent extends Equatable {
  const SearchFilterEvent();
  @override
  List<Object?> get props => [];
}

final class UpdateSelectedTags extends SearchFilterEvent {
  const UpdateSelectedTags(this.tags);
  final List<String> tags;
  @override
  List<Object?> get props => [tags];
}

final class UpdateSortBy extends SearchFilterEvent {
  const UpdateSortBy(this.sortBy);
  final String sortBy;
  @override
  List<Object?> get props => [sortBy];
}

final class UpdateSortOrder extends SearchFilterEvent {
  const UpdateSortOrder(this.sortOrder);
  final String sortOrder;
  @override
  List<Object?> get props => [sortOrder];
}

final class UpdateFilterLogic extends SearchFilterEvent {
  const UpdateFilterLogic(this.logic);
  final String logic;
  @override
  List<Object?> get props => [logic];
}

final class ApplyFilterSnapshot extends SearchFilterEvent {
  const ApplyFilterSnapshot({
    required this.selectedTags,
    required this.sortBy,
    required this.filterLogic,
  });

  final List<String> selectedTags;
  final String sortBy;
  final String filterLogic;

  @override
  List<Object?> get props => [selectedTags, sortBy, filterLogic];
}

final class ClearFilters extends SearchFilterEvent {
  const ClearFilters();
}

// State
class SearchFilterState extends Equatable {
  const SearchFilterState({
    this.selectedTags = const [],
    this.sortBy = 'date_added',
    this.sortOrder = 'desc',
    this.filterLogic = 'and',
  });

  final List<String> selectedTags;
  final String sortBy;
  final String sortOrder;
  final String filterLogic;

  SearchFilterState copyWith({
    List<String>? selectedTags,
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    return SearchFilterState(
      selectedTags: selectedTags ?? this.selectedTags,
      sortBy: sortBy ?? this.sortBy,
      sortOrder: sortOrder ?? this.sortOrder,
      filterLogic: filterLogic ?? this.filterLogic,
    );
  }

  @override
  List<Object?> get props => [selectedTags, sortBy, sortOrder, filterLogic];
}

// BLoC
class SearchFilterBloc extends Bloc<SearchFilterEvent, SearchFilterState> {
  SearchFilterBloc() : super(const SearchFilterState()) {
    on<UpdateSelectedTags>(_onUpdateSelectedTags);
    on<UpdateSortBy>(_onUpdateSortBy);
    on<UpdateSortOrder>(_onUpdateSortOrder);
    on<UpdateFilterLogic>(_onUpdateFilterLogic);
    on<ApplyFilterSnapshot>(_onApplyFilterSnapshot);
    on<ClearFilters>(_onClearFilters);
  }

  void _onUpdateSelectedTags(
    UpdateSelectedTags event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(state.copyWith(selectedTags: event.tags));
  }

  void _onUpdateSortBy(UpdateSortBy event, Emitter<SearchFilterState> emit) {
    emit(state.copyWith(sortBy: event.sortBy));
  }

  void _onUpdateSortOrder(
    UpdateSortOrder event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(state.copyWith(sortOrder: event.sortOrder));
  }

  void _onUpdateFilterLogic(
    UpdateFilterLogic event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(state.copyWith(filterLogic: event.logic));
  }

  void _onApplyFilterSnapshot(
    ApplyFilterSnapshot event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(
      state.copyWith(
        selectedTags: event.selectedTags,
        sortBy: event.sortBy,
        filterLogic: event.filterLogic,
      ),
    );
  }

  void _onClearFilters(ClearFilters event, Emitter<SearchFilterState> emit) {
    emit(const SearchFilterState());
  }
}
