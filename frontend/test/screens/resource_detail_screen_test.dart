import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/ebook/ebook_bloc.dart';
import 'package:personal_inventory_frontend/blocs/web_reader/web_reader_bloc.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/screens/resource_detail_screen.dart';

import '../support/fake_repositories.dart';

void main() {
  testWidgets('ResourceDetailScreen renders ebook detail and locations', (tester) async {
    final ebookRepo = FakeEbookRepository(
      detailResult: const Success(
        EbookDetail(
          resource: Resource(
            id: 'e1',
            title: 'Deep Book',
            resourceType: ResourceType.ebook,
          ),
          meta: EbookMeta(resourceId: 'e1', author: 'Author A', fileFormat: 'epub'),
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
    await tester.enterText(find.byKey(const Key('location-device-id')), 'new-device');
    await tester.enterText(find.byKey(const Key('location-path-or-url')), '/tmp/new.epub');
    await tester.tap(find.byKey(const Key('location-submit')));
    await tester.pump();
  });

  testWidgets('ResourceDetailScreen dispatches progress signal hook for web reader', (
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

    await tester.pumpWidget(
      MaterialApp(
        home: MultiBlocProvider(
          providers: [
            BlocProvider(create: (_) => EbookBloc(ebookRepo)),
            BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
          ],
          child: const ResourceDetailScreen(
            resourceId: 'w1',
            resourceType: ResourceType.webReader,
          ),
        ),
      ),
    );

    await tester.pump();

    await tester.enterText(
      find.byKey(const Key('progress-url')),
      'https://example.com/ch2',
    );
    await tester.enterText(find.byKey(const Key('progress-chapter')), 'Chapter 2');
    await tester.enterText(find.byKey(const Key('progress-value')), '0.42');
    await tester.tap(find.byKey(const Key('progress-dispatch')));
    await tester.pump();

    expect(webReaderRepo.trackProgressCalls, 1);
  });

  testWidgets('ResourceDetailScreen delete confirmation dispatches ebook delete', (tester) async {
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
}
