import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/search_filter/search_filter_bloc.dart';

void main() {
  group('SearchFilterBloc', () {
    late SearchFilterBloc bloc;

    setUp(() {
      bloc = SearchFilterBloc();
    });

    tearDown(() => bloc.close());

    test('initial state has empty tags and default sort', () {
      expect(bloc.state.selectedTags, isEmpty);
      expect(bloc.state.sortBy, equals('date_added'));
      expect(bloc.state.sortOrder, equals('desc'));
      expect(bloc.state.filterLogic, equals('and'));
    });

    test('UpdateSelectedTags updates tags', () async {
      bloc.add(const UpdateSelectedTags(['fiction', 'sci-fi']));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.selectedTags, 'selectedTags', ['fiction', 'sci-fi'],
        )),
      );
    });

    test('UpdateSortBy updates sort field', () async {
      bloc.add(const UpdateSortBy('title'));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.sortBy, 'sortBy', 'title',
        )),
      );
    });

    test('UpdateSortOrder updates sort direction', () async {
      bloc.add(const UpdateSortOrder('asc'));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.sortOrder, 'sortOrder', 'asc',
        )),
      );
    });

    test('UpdateFilterLogic updates logic', () async {
      bloc.add(const UpdateFilterLogic('or'));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.filterLogic, 'filterLogic', 'or',
        )),
      );
    });

    test('ApplyFilterSnapshot updates replayed filters in one emission', () async {
      bloc.add(
        const ApplyFilterSnapshot(
          selectedTags: ['fiction'],
          sortBy: 'title',
          filterLogic: 'or',
        ),
      );
      await expectLater(
        bloc.stream,
        emits(
          isA<SearchFilterState>()
              .having((s) => s.selectedTags, 'selectedTags', ['fiction'])
              .having((s) => s.sortBy, 'sortBy', 'title')
              .having((s) => s.filterLogic, 'filterLogic', 'or'),
        ),
      );
    });

    test('ClearFilters resets to defaults', () async {
      bloc.add(const UpdateSelectedTags(['fiction']));
      bloc.add(const UpdateSortBy('title'));
      bloc.add(const ClearFilters());
      await expectLater(
        bloc.stream,
        emitsThrough(isA<SearchFilterState>().having(
          (s) => s.selectedTags, 'selectedTags', isEmpty,
        )),
      );
    });
  });
}
