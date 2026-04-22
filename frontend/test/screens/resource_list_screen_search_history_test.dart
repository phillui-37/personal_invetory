import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/batch/batch_bloc.dart';
import 'package:personal_inventory_frontend/blocs/ebook/ebook_bloc.dart';
import 'package:personal_inventory_frontend/blocs/game/game_bloc.dart';
import 'package:personal_inventory_frontend/blocs/image/image_bloc.dart';
import 'package:personal_inventory_frontend/blocs/progress/progress_bloc.dart';
import 'package:personal_inventory_frontend/blocs/search_filter/search_filter_bloc.dart';
import 'package:personal_inventory_frontend/blocs/tag/tag_bloc.dart';
import 'package:personal_inventory_frontend/blocs/video/video_bloc.dart';
import 'package:personal_inventory_frontend/blocs/web_reader/web_reader_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/search_history.dart';
import 'package:personal_inventory_frontend/models/tag.dart';
import 'package:personal_inventory_frontend/screens/resource_list_screen.dart';
import 'package:personal_inventory_frontend/services/search_history_service.dart';
import 'package:personal_inventory_frontend/services/search_history_storage.dart';

import '../support/fake_repositories.dart';

class _MemorySearchHistoryStorage implements SearchHistoryStorage {
  _MemorySearchHistoryStorage({List<SearchHistory>? initialHistory})
      : _history = List<SearchHistory>.from(initialHistory ?? const []);

  List<SearchHistory> _history;
  final List<List<SearchHistory>> savedSnapshots = [];

  @override
  Future<List<SearchHistory>> load() async =>
      List<SearchHistory>.from(_history);

  @override
  Future<void> save(List<SearchHistory> history) async {
    _history = List<SearchHistory>.from(history);
    savedSnapshots.add(List<SearchHistory>.from(history));
  }
}

class _ListCall {
  const _ListCall({
    required this.tags,
    required this.sortBy,
    required this.sortOrder,
    required this.filterLogic,
  });

  final List<String> tags;
  final String? sortBy;
  final String? sortOrder;
  final String? filterLogic;
}

class _SpyEbookRepository extends FakeEbookRepository {
  _ListCall? lastListCall;
  int listCallCount = 0;

  @override
  Future<Result<List<Resource>, AppFailure>> listEbooks({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    listCallCount += 1;
    lastListCall = _ListCall(
      tags: List<String>.from(tags),
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
    return super.listEbooks(
      tags: tags,
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
  }
}

SearchHistory _entry({
  required String id,
  required String query,
  required List<String> tags,
  required String sortBy,
  required String filterLogic,
}) {
  return SearchHistory(
    id: id,
    query: query,
    tags: tags,
    sortBy: sortBy,
    filterLogic: filterLogic,
    timestamp: DateTime.utc(2026, 4, 22, 12, 0),
  );
}

Future<SearchHistoryService> _buildHistoryService(
    List<SearchHistory> history) async {
  final service = SearchHistoryService(
    storage: _MemorySearchHistoryStorage(initialHistory: history),
  );
  await service.load();
  return service;
}

Widget _buildScreen({
  required SearchHistoryService searchHistoryService,
  required FakeTagRepository tagRepo,
  FakeEbookRepository? ebookRepo,
  FakeWebReaderRepository? webReaderRepo,
  FakeImageRepository? imageRepo,
  FakeVideoRepository? videoRepo,
  FakeGameRepository? gameRepo,
}) {
  return RepositoryProvider<SearchHistoryService>.value(
    value: searchHistoryService,
    child: MultiBlocProvider(
      providers: [
        BlocProvider(
            create: (_) => EbookBloc(ebookRepo ?? FakeEbookRepository())),
        BlocProvider(
          create: (_) =>
              WebReaderBloc(webReaderRepo ?? FakeWebReaderRepository()),
        ),
        BlocProvider(
            create: (_) => ImageBloc(imageRepo ?? FakeImageRepository())),
        BlocProvider(
            create: (_) => VideoBloc(videoRepo ?? FakeVideoRepository())),
        BlocProvider(create: (_) => GameBloc(gameRepo ?? FakeGameRepository())),
        BlocProvider(create: (_) => TagBloc(tagRepo)),
        BlocProvider(create: (_) => ProgressBloc(FakeProgressRepository())),
        BlocProvider(create: (_) => BatchBloc(FakeBatchOperationRepository())),
        BlocProvider(create: (_) => SearchFilterBloc()),
      ],
      child: const MaterialApp(home: ResourceListScreen()),
    ),
  );
}

void main() {
  testWidgets('replay triggers one filtered reload wave', (tester) async {
    final historyService = await _buildHistoryService([
      _entry(
        id: 'saved-1',
        query: 'Alpha',
        tags: ['favorite'],
        sortBy: 'title',
        filterLogic: 'or',
      ),
    ]);
    final ebookRepo = _SpyEbookRepository()
      ..listResult = const Success<List<Resource>, AppFailure>([
        Resource(
          id: 'ebook-1',
          title: 'Alpha Manual',
          resourceType: ResourceType.ebook,
        ),
      ]);

    await tester.pumpWidget(
      _buildScreen(
        searchHistoryService: historyService,
        ebookRepo: ebookRepo,
        tagRepo: FakeTagRepository(
          listResult: Success<List<Tag>, AppFailure>([
            Tag(
              id: 'tag-1',
              name: 'favorite',
              createdAt: DateTime.utc(2026, 4, 21),
            ),
          ]),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(ebookRepo.listCallCount, 1);

    await tester.tap(find.byKey(const Key('search-history-item-saved-1')));
    await tester.pumpAndSettle();

    expect(ebookRepo.listCallCount, 2);
  });

  testWidgets('replay restores query, tags, sort, and filter logic', (
    tester,
  ) async {
    final historyService = await _buildHistoryService([
      _entry(
        id: 'saved-1',
        query: 'Alpha',
        tags: ['favorite'],
        sortBy: 'title',
        filterLogic: 'or',
      ),
    ]);
    final ebookRepo = _SpyEbookRepository()
      ..listResult = const Success<List<Resource>, AppFailure>([
        Resource(
          id: 'ebook-1',
          title: 'Alpha Manual',
          resourceType: ResourceType.ebook,
        ),
        Resource(
          id: 'ebook-2',
          title: 'Beta Manual',
          resourceType: ResourceType.ebook,
        ),
      ]);

    await tester.pumpWidget(
      _buildScreen(
        searchHistoryService: historyService,
        ebookRepo: ebookRepo,
        tagRepo: FakeTagRepository(
          listResult: Success<List<Tag>, AppFailure>([
            Tag(
              id: 'tag-1',
              name: 'favorite',
              createdAt: DateTime.utc(2026, 4, 21),
            ),
          ]),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('search-history-item-saved-1')));
    await tester.pumpAndSettle();

    final searchField = tester.widget<TextField>(
      find.byKey(const Key('search-query-input')),
    );
    expect(searchField.controller!.text, 'Alpha');
    expect(
      tester
          .widget<FilterChip>(find.byKey(const Key('filter-tag-favorite')))
          .selected,
      isTrue,
    );
    expect(
      tester
          .widget<DropdownButton<String>>(
              find.byKey(const Key('sort-dropdown')))
          .value,
      'title',
    );
    expect(
      tester
          .widget<DropdownButton<String>>(
              find.byKey(const Key('logic-dropdown')))
          .value,
      'or',
    );
    expect(ebookRepo.lastListCall, isNotNull);
    expect(ebookRepo.lastListCall!.tags, ['favorite']);
    expect(ebookRepo.lastListCall!.sortBy, 'title');
    expect(ebookRepo.lastListCall!.filterLogic, 'or');
    expect(find.text('Alpha Manual'), findsOneWidget);
    expect(find.text('Beta Manual'), findsNothing);
  });

  testWidgets('removes a single saved search from the panel', (tester) async {
    final historyService = await _buildHistoryService([
      _entry(
        id: 'saved-1',
        query: 'Alpha',
        tags: ['favorite'],
        sortBy: 'title',
        filterLogic: 'or',
      ),
      _entry(
        id: 'saved-2',
        query: 'Beta',
        tags: ['archive'],
        sortBy: 'date_added',
        filterLogic: 'and',
      ),
    ]);

    await tester.pumpWidget(
      _buildScreen(
        searchHistoryService: historyService,
        tagRepo: FakeTagRepository(
          listResult: Success<List<Tag>, AppFailure>([
            Tag(
              id: 'tag-1',
              name: 'favorite',
              createdAt: DateTime.utc(2026, 4, 21),
            ),
          ]),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('search-history-remove-saved-1')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('search-history-item-saved-1')), findsNothing);
    expect(
        find.byKey(const Key('search-history-item-saved-2')), findsOneWidget);
    expect(historyService.history.map((entry) => entry.id), ['saved-2']);
  });

  testWidgets('clears all saved searches from the panel', (tester) async {
    final historyService = await _buildHistoryService([
      _entry(
        id: 'saved-1',
        query: 'Alpha',
        tags: ['favorite'],
        sortBy: 'title',
        filterLogic: 'or',
      ),
    ]);

    await tester.pumpWidget(
      _buildScreen(
        searchHistoryService: historyService,
        tagRepo: FakeTagRepository(
          listResult: Success<List<Tag>, AppFailure>([
            Tag(
              id: 'tag-1',
              name: 'favorite',
              createdAt: DateTime.utc(2026, 4, 21),
            ),
          ]),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('search-history-clear-all')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('search-history-item-saved-1')), findsNothing);
    expect(find.text('Recent searches'), findsNothing);
    expect(historyService.history, isEmpty);
  });

  testWidgets('typing a query saves a new search history entry',
      (tester) async {
    final historyStorage = _MemorySearchHistoryStorage();
    final historyService = SearchHistoryService(storage: historyStorage);
    await historyService.load();

    await tester.pumpWidget(
      _buildScreen(
        searchHistoryService: historyService,
        tagRepo: FakeTagRepository(),
      ),
    );
    await tester.pumpAndSettle();

    await tester.enterText(
      find.byKey(const Key('search-query-input')),
      'Alpha',
    );
    await tester.pump(const Duration(milliseconds: 500));

    expect(historyService.history, hasLength(1));
    expect(historyService.history.single.query, 'Alpha');
    expect(historyStorage.savedSnapshots, hasLength(1));
    expect(find.text('Recent searches'), findsOneWidget);
    expect(find.text('Alpha'), findsWidgets);
  });

  testWidgets('tag filtering saves the latest combined search snapshot', (
    tester,
  ) async {
    final historyStorage = _MemorySearchHistoryStorage();
    final historyService = SearchHistoryService(storage: historyStorage);
    await historyService.load();

    await tester.pumpWidget(
      _buildScreen(
        searchHistoryService: historyService,
        tagRepo: FakeTagRepository(
          listResult: Success<List<Tag>, AppFailure>([
            Tag(
              id: 'tag-1',
              name: 'favorite',
              createdAt: DateTime.utc(2026, 4, 21),
            ),
          ]),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.enterText(
      find.byKey(const Key('search-query-input')),
      'Alpha',
    );
    await tester.tap(find.byKey(const Key('filter-tag-favorite')));
    await tester.pump(const Duration(milliseconds: 500));

    expect(historyService.history, hasLength(1));
    expect(historyService.history.single.query, 'Alpha');
    expect(historyService.history.single.tags, ['favorite']);
    expect(historyService.history.single.sortBy, 'date_added');
    expect(historyService.history.single.filterLogic, 'and');
    expect(historyStorage.savedSnapshots, hasLength(1));
  });
}
