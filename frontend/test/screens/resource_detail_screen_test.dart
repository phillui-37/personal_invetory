import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/ebook/ebook_bloc.dart';
import 'package:personal_inventory_frontend/blocs/game/game_bloc.dart';
import 'package:personal_inventory_frontend/blocs/image/image_bloc.dart';
import 'package:personal_inventory_frontend/blocs/progress/progress_bloc.dart';
import 'package:personal_inventory_frontend/blocs/tag/tag_bloc.dart';
import 'package:personal_inventory_frontend/blocs/video/video_bloc.dart';
import 'package:personal_inventory_frontend/blocs/web_reader/web_reader_bloc.dart';
import 'package:personal_inventory_frontend/models/progress.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/screens/resource_detail_screen.dart';
import 'package:personal_inventory_frontend/widgets/web_reader_progress_tracker.dart';

import '../support/fake_repositories.dart';

void main() {
  testWidgets('ResourceDetailScreen renders ebook detail and locations',
      (tester) async {
    final ebookRepo = FakeEbookRepository(
      detailResult: const Success(
        EbookDetail(
          resource: Resource(
            id: 'e1',
            title: 'Deep Book',
            resourceType: ResourceType.ebook,
          ),
          meta: EbookMeta(
              resourceId: 'e1', author: 'Author A', fileFormat: 'epub'),
          locations: [
            ResourceLocation(
              id: 'loc1',
              resourceId: 'e1',
              deviceId: 'usb-A',
              pathOrUrl: '/mnt/books/deep.epub',
              storageType: StorageType.portable,
            ),
          ],
        ),
      ),
      removeLocationResult: const Success(null),
      addLocationResult: const Success(
        ResourceLocation(
          id: 'loc2',
          resourceId: 'e1',
          deviceId: 'macbook',
          pathOrUrl: '/books/deep.epub',
          storageType: StorageType.localFs,
        ),
      ),
    );
    final webReaderRepo = FakeWebReaderRepository();

    await tester.pumpWidget(
      MaterialApp(
        home: MultiBlocProvider(
          providers: [
            BlocProvider(create: (_) => EbookBloc(ebookRepo)),
            BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
            BlocProvider(create: (_) => ProgressBloc(FakeProgressRepository())),
            BlocProvider(create: (_) => TagBloc(FakeTagRepository())),
            BlocProvider(create: (_) => ImageBloc(FakeImageRepository())),
            BlocProvider(create: (_) => VideoBloc(FakeVideoRepository())),
            BlocProvider(create: (_) => GameBloc(FakeGameRepository())),
          ],
          child: const ResourceDetailScreen(
            resourceId: 'e1',
            resourceType: ResourceType.ebook,
          ),
        ),
      ),
    );

    await tester.pump();
    expect(find.text('Deep Book'), findsOneWidget);
    expect(find.text('/mnt/books/deep.epub'), findsOneWidget);

    await tester.tap(find.byKey(const Key('remove-location-loc1')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Remove'));
    await tester.pump();

    await tester.tap(find.byKey(const Key('add-location-button')));
    await tester.pumpAndSettle();
    await tester.enterText(
        find.byKey(const Key('location-device-id')), 'new-device');
    await tester.enterText(
        find.byKey(const Key('location-path-or-url')), '/tmp/new.epub');
    await tester.tap(find.byKey(const Key('location-submit')));
    await tester.pump();
  });

  testWidgets(
      'ResourceDetailScreen dispatches progress signal hook for web reader', (
    tester,
  ) async {
    final ebookRepo = FakeEbookRepository();
    final webReaderRepo = FakeWebReaderRepository(
      detailResult: const Success(
        WebReaderDetail(
          resource: Resource(
            id: 'w1',
            title: 'Reader',
            resourceType: ResourceType.webReader,
          ),
          meta: WebReaderMeta(resourceId: 'w1', url: 'https://example.com/ch1'),
          locations: [],
        ),
      ),
      trackProgressResult: const Success(null),
    );
    final progressRepo = FakeProgressRepository(
      upsertResult: Success(
        ResourceProgress(
          resourceId: 'w1',
          progress: 0.42,
          notes: 'Chapter 2',
          updatedAt: DateTime.utc(2025, 1, 1),
        ),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MultiBlocProvider(
          providers: [
            BlocProvider(create: (_) => EbookBloc(ebookRepo)),
            BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
            BlocProvider(create: (_) => ProgressBloc(progressRepo)),
            BlocProvider(create: (_) => TagBloc(FakeTagRepository())),
            BlocProvider(create: (_) => ImageBloc(FakeImageRepository())),
            BlocProvider(create: (_) => VideoBloc(FakeVideoRepository())),
            BlocProvider(create: (_) => GameBloc(FakeGameRepository())),
          ],
          child: const ResourceDetailScreen(
            resourceId: 'w1',
            resourceType: ResourceType.webReader,
          ),
        ),
      ),
    );

    await tester.pump();

    final trackerState = tester.state<WebReaderProgressTrackerState>(
      find.byType(WebReaderProgressTracker),
    );
    trackerState
        .invokeProgressUpdate('{"chapter":"Chapter 2","progress":0.42}');
    await tester.pump();

    expect(progressRepo.upsertCalls, 1);
    expect(progressRepo.lastUpsertResourceType, ResourceType.webReader);
    expect(progressRepo.lastUpsertResourceId, 'w1');
    expect(progressRepo.lastUpsertProgress, 0.42);
    expect(progressRepo.lastUpsertNotes, 'Chapter 2');
    expect(webReaderRepo.trackProgressCalls, 0);
  });

  testWidgets(
      'ResourceDetailScreen delete confirmation dispatches ebook delete',
      (tester) async {
    final ebookRepo = FakeEbookRepository(
      detailResult: const Success(
        EbookDetail(
          resource: Resource(
            id: 'e1',
            title: 'Delete Me',
            resourceType: ResourceType.ebook,
          ),
          meta: EbookMeta(resourceId: 'e1'),
          locations: [],
        ),
      ),
      deleteResult: const Success(null),
    );
    final webReaderRepo = FakeWebReaderRepository();

    await tester.pumpWidget(
      MaterialApp(
        home: MultiBlocProvider(
          providers: [
            BlocProvider(create: (_) => EbookBloc(ebookRepo)),
            BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
            BlocProvider(create: (_) => ProgressBloc(FakeProgressRepository())),
            BlocProvider(create: (_) => TagBloc(FakeTagRepository())),
            BlocProvider(create: (_) => ImageBloc(FakeImageRepository())),
            BlocProvider(create: (_) => VideoBloc(FakeVideoRepository())),
            BlocProvider(create: (_) => GameBloc(FakeGameRepository())),
          ],
          child: const ResourceDetailScreen(
            resourceId: 'e1',
            resourceType: ResourceType.ebook,
          ),
        ),
      ),
    );

    await tester.pump();
    await tester.tap(find.byIcon(Icons.delete));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete'));
    await tester.pump();

    expect(ebookRepo.deleteCalls, 1);
  });

  testWidgets(
      'ResourceDetailScreen shows chapter checks and dispatches check now', (
    tester,
  ) async {
    final checkedAt = DateTime.utc(2025, 1, 1, 12);
    final ebookRepo = FakeEbookRepository();
    final webReaderRepo = FakeWebReaderRepository(
      detailResult: const Success(
        WebReaderDetail(
          resource: Resource(
            id: 'w1',
            title: 'Reader',
            resourceType: ResourceType.webReader,
          ),
          meta: WebReaderMeta(resourceId: 'w1', url: 'https://example.com/ch1'),
          locations: [],
        ),
      ),
      listCheckHistoryResult: Success([
        ChapterCheck(
          id: 'chk1',
          resourceId: 'w1',
          hasNewChapter: true,
          latestChapter: 'Chapter 2',
          checkedAt: checkedAt,
        ),
      ]),
      triggerCheckResult: Success(
        ChapterCheck(
          id: 'chk2',
          resourceId: 'w1',
          hasNewChapter: false,
          checkedAt: checkedAt,
        ),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MultiBlocProvider(
          providers: [
            BlocProvider(create: (_) => EbookBloc(ebookRepo)),
            BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
            BlocProvider(create: (_) => ProgressBloc(FakeProgressRepository())),
            BlocProvider(create: (_) => TagBloc(FakeTagRepository())),
            BlocProvider(create: (_) => ImageBloc(FakeImageRepository())),
            BlocProvider(create: (_) => VideoBloc(FakeVideoRepository())),
            BlocProvider(create: (_) => GameBloc(FakeGameRepository())),
          ],
          child: const ResourceDetailScreen(
            resourceId: 'w1',
            resourceType: ResourceType.webReader,
          ),
        ),
      ),
    );

    await tester.pump();
    await tester.pump();

    expect(find.text('Chapter Checks'), findsOneWidget);
    await tester.tap(find.byType(ExpansionTile));
    await tester.pumpAndSettle();
    expect(find.textContaining('Latest: Chapter 2'), findsOneWidget);

    final button =
        tester.widget<TextButton>(find.byKey(const Key('check-now-button')));
    button.onPressed!.call();
    await tester.pump();

    expect(webReaderRepo.triggerCheckCalls, 1);
    expect(webReaderRepo.lastTriggerCheckResourceId, 'w1');
  });
}
