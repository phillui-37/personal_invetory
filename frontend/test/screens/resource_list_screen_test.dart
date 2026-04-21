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
import 'package:personal_inventory_frontend/models/tag.dart';
import 'package:personal_inventory_frontend/screens/resource_list_screen.dart';

import '../support/fake_repositories.dart';

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

  @override
  Future<Result<List<Resource>, AppFailure>> listEbooks({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
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

class _SpyWebReaderRepository extends FakeWebReaderRepository {
  _ListCall? lastListCall;

  @override
  Future<Result<List<Resource>, AppFailure>> listWebReaders({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    lastListCall = _ListCall(
      tags: List<String>.from(tags),
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
    return super.listWebReaders(
      tags: tags,
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
  }
}

class _SpyImageRepository extends FakeImageRepository {
  _ListCall? lastListCall;

  @override
  Future<Result<List<Resource>, AppFailure>> listImages({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    lastListCall = _ListCall(
      tags: List<String>.from(tags),
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
    return super.listImages(
      tags: tags,
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
  }
}

class _SpyVideoRepository extends FakeVideoRepository {
  _ListCall? lastListCall;

  @override
  Future<Result<List<Resource>, AppFailure>> listVideos({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    lastListCall = _ListCall(
      tags: List<String>.from(tags),
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
    return super.listVideos(
      tags: tags,
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
  }
}

class _SpyGameRepository extends FakeGameRepository {
  _ListCall? lastListCall;

  @override
  Future<Result<List<Resource>, AppFailure>> listGames({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    lastListCall = _ListCall(
      tags: List<String>.from(tags),
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
    return super.listGames(
      tags: tags,
      sortBy: sortBy,
      sortOrder: sortOrder,
      filterLogic: filterLogic,
    );
  }
}

void main() {
  Widget buildScreen({
    FakeEbookRepository? ebookRepo,
    FakeWebReaderRepository? webReaderRepo,
    FakeImageRepository? imageRepo,
    FakeVideoRepository? videoRepo,
    FakeGameRepository? gameRepo,
    FakeTagRepository? tagRepo,
    SearchFilterBloc? searchFilterBloc,
  }) {
    return MultiBlocProvider(
      providers: [
        BlocProvider(
            create: (_) => EbookBloc(ebookRepo ?? FakeEbookRepository())),
        BlocProvider(
            create: (_) =>
                WebReaderBloc(webReaderRepo ?? FakeWebReaderRepository())),
        BlocProvider(
            create: (_) => ImageBloc(imageRepo ?? FakeImageRepository())),
        BlocProvider(
            create: (_) => VideoBloc(videoRepo ?? FakeVideoRepository())),
        BlocProvider(create: (_) => GameBloc(gameRepo ?? FakeGameRepository())),
        BlocProvider(create: (_) => TagBloc(tagRepo ?? FakeTagRepository())),
        BlocProvider(create: (_) => ProgressBloc(FakeProgressRepository())),
        BlocProvider(create: (_) => BatchBloc(FakeBatchOperationRepository())),
        if (searchFilterBloc != null)
          BlocProvider<SearchFilterBloc>.value(value: searchFilterBloc)
        else
          BlocProvider(create: (_) => SearchFilterBloc()),
      ],
      child: const MaterialApp(
        home: ResourceListScreen(),
      ),
    );
  }

  testWidgets('ResourceListScreen renders ebook list item', (tester) async {
    final ebookRepo = FakeEbookRepository(
      listResult: const Success([
        Resource(id: 'e1', title: 'Book One', resourceType: ResourceType.ebook),
      ]),
    );
    final webReaderRepo = FakeWebReaderRepository(
      listResult: const Success([]),
    );

    await tester.pumpWidget(
      buildScreen(ebookRepo: ebookRepo, webReaderRepo: webReaderRepo),
    );

    await tester.pump();
    expect(find.text('Book One'), findsOneWidget);
    expect(find.byKey(const Key('add-resource-fab')), findsOneWidget);
  });

  testWidgets('ResourceListScreen shows batch operations icon button',
      (tester) async {
    await tester.pumpWidget(buildScreen());

    await tester.pump();
    expect(find.byKey(const Key('batch-operations-icon')), findsOneWidget);
  });

  testWidgets('ResourceListScreen reloads list after returning from detail',
      (tester) async {
    final ebookRepo = FakeEbookRepository(
      listResult: const Success([
        Resource(id: 'e1', title: 'Book One', resourceType: ResourceType.ebook),
      ]),
      detailResult: const Success(
        EbookDetail(
          resource: Resource(
              id: 'e1', title: 'Book One', resourceType: ResourceType.ebook),
          meta: EbookMeta(resourceId: 'e1'),
          locations: [],
        ),
      ),
    );
    final webReaderRepo = FakeWebReaderRepository(
      listResult: const Success([]),
    );

    await tester.pumpWidget(
      buildScreen(ebookRepo: ebookRepo, webReaderRepo: webReaderRepo),
    );

    await tester.pump();
    expect(ebookRepo.listCalls, 1);

    await tester.tap(find.text('Book One'));
    await tester.pumpAndSettle();

    await tester.pageBack();
    await tester.pumpAndSettle();

    expect(ebookRepo.listCalls, greaterThanOrEqualTo(2));
  });

  testWidgets('ResourceListScreen reloads all tabs with updated filter params',
      (tester) async {
    final ebookRepo = _SpyEbookRepository();
    final webReaderRepo = _SpyWebReaderRepository();
    final imageRepo = _SpyImageRepository();
    final videoRepo = _SpyVideoRepository();
    final gameRepo = _SpyGameRepository();
    final tagRepo = FakeTagRepository(
      listResult: Success<List<Tag>, AppFailure>([
        Tag(
            id: 'tag-1',
            name: 'favorite',
            createdAt: DateTime.utc(2026, 4, 21)),
      ]),
    );

    await tester.pumpWidget(
      buildScreen(
        ebookRepo: ebookRepo,
        webReaderRepo: webReaderRepo,
        imageRepo: imageRepo,
        videoRepo: videoRepo,
        gameRepo: gameRepo,
        tagRepo: tagRepo,
      ),
    );
    await tester.pump();

    await tester.tap(find.byKey(const Key('filter-tag-favorite')));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('logic-dropdown')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('OR').last);
    await tester.pumpAndSettle();

    for (final call in [
      ebookRepo.lastListCall,
      webReaderRepo.lastListCall,
      imageRepo.lastListCall,
      videoRepo.lastListCall,
      gameRepo.lastListCall,
    ]) {
      expect(call, isNotNull);
      expect(call!.tags, ['favorite']);
      expect(call.sortBy, 'date_added');
      expect(call.sortOrder, 'desc');
      expect(call.filterLogic, 'or');
    }

    expect(find.byKey(const Key('clear-filters-chip')), findsOneWidget);
  });

  testWidgets('ResourceListScreen skips filter bar when no tags are available',
      (tester) async {
    await tester.pumpWidget(
      buildScreen(
        tagRepo: FakeTagRepository(
          listResult: const Success<List<Tag>, AppFailure>([]),
        ),
      ),
    );
    await tester.pump();

    expect(find.byKey(const Key('tag-chips-list')), findsNothing);
    expect(find.byKey(const Key('clear-filters-chip')), findsNothing);
  });
}
