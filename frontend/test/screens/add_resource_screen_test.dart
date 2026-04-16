import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/ebook/ebook_bloc.dart';
import 'package:personal_inventory_frontend/blocs/web_reader/web_reader_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/screens/add_resource_screen.dart';

import '../support/fake_repositories.dart';

void main() {
  testWidgets('AddResourceScreen submits ebook payload from form', (tester) async {
    final ebookRepo = FakeEbookRepository(
      addResult: const Success(
        Resource(id: 'e1', title: 'Book One', resourceType: ResourceType.ebook),
      ),
    );
    final webReaderRepo = FakeWebReaderRepository();

    await tester.pumpWidget(
      MultiBlocProvider(
        providers: [
          BlocProvider(create: (_) => EbookBloc(ebookRepo)),
          BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
        ],
        child: const MaterialApp(
          home: AddResourceScreen(),
        ),
      ),
    );

    await tester.enterText(find.byKey(const Key('resource-title')), 'Book One');
    await tester.enterText(find.byKey(const Key('ebook-author')), 'Author One');
    await tester.enterText(find.byKey(const Key('location-device-id')), 'macbook');
    await tester.enterText(find.byKey(const Key('location-path')), '/books/book.epub');
    await tester.tap(find.byKey(const Key('resource-submit')));
    await tester.pump();

    expect(ebookRepo.lastAddInput?.resource.title, 'Book One');
    expect(ebookRepo.lastAddInput?.meta.author, 'Author One');
    expect(ebookRepo.lastAddInput?.resource.id, isNot('new-resource'));
    expect(ebookRepo.addLocationCalls, 1);
  });

  testWidgets('AddResourceScreen shows validation feedback from bloc failure', (
    tester,
  ) async {
    final ebookRepo = FakeEbookRepository(
      addResult: const Failure(ValidationFailure(field: 'title', message: 'required')),
    );
    final webReaderRepo = FakeWebReaderRepository();

    await tester.pumpWidget(
      MultiBlocProvider(
        providers: [
          BlocProvider(create: (_) => EbookBloc(ebookRepo)),
          BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
        ],
        child: const MaterialApp(
          home: AddResourceScreen(),
        ),
      ),
    );

    await tester.tap(find.byKey(const Key('resource-submit')));
    await tester.pump();

    expect(find.textContaining('title: required'), findsWidgets);
    expect(ebookRepo.addLocationCalls, 0);
  });

  testWidgets('AddResourceScreen edit mode dispatches update event', (tester) async {
    final ebookRepo = FakeEbookRepository(
      updateResult: const Success(
        Resource(id: 'e1', title: 'Updated', resourceType: ResourceType.ebook),
      ),
    );
    final webReaderRepo = FakeWebReaderRepository();

    await tester.pumpWidget(
      MultiBlocProvider(
        providers: [
          BlocProvider(create: (_) => EbookBloc(ebookRepo)),
          BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
        ],
        child: const MaterialApp(
          home: AddResourceScreen(resourceId: 'e1'),
        ),
      ),
    );

    await tester.enterText(find.byKey(const Key('resource-title')), 'Updated');
    await tester.enterText(find.byKey(const Key('ebook-author')), 'Author Updated');
    await tester.tap(find.byKey(const Key('resource-submit')));
    await tester.pump();

    expect(ebookRepo.addCalls, 0);
    expect(ebookRepo.updateCalls, 1);
    expect(ebookRepo.lastUpdateId, 'e1');
  });

  testWidgets('AddResourceScreen maps site name into web reader siteName', (tester) async {
    final ebookRepo = FakeEbookRepository();
    final webReaderRepo = FakeWebReaderRepository(
      addResult: const Success(
        Resource(id: 'w1', title: 'Reader', resourceType: ResourceType.webReader),
      ),
    );

    await tester.pumpWidget(
      MultiBlocProvider(
        providers: [
          BlocProvider(create: (_) => EbookBloc(ebookRepo)),
          BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
        ],
        child: const MaterialApp(
          home: AddResourceScreen(initialResourceType: ResourceType.webReader),
        ),
      ),
    );

    await tester.enterText(find.byKey(const Key('resource-title')), 'Reader');
    await tester.enterText(find.byKey(const Key('web-reader-url')), 'https://example.com/read');
    await tester.enterText(find.byKey(const Key('web-reader-site-name')), 'Example Site');
    await tester.tap(find.byKey(const Key('resource-submit')));
    await tester.pump();

    expect(webReaderRepo.lastAddInput?.meta.siteName, 'Example Site');
    expect(webReaderRepo.lastAddInput?.meta.lastReadChapter, isNull);
  });

  testWidgets('AddResourceScreen edit mode pre-fills ebook fields from detail', (tester) async {
    final ebookRepo = FakeEbookRepository(
      detailResult: const Success(
        EbookDetail(
          resource: Resource(
            id: 'e1',
            title: 'Stored Ebook',
            resourceType: ResourceType.ebook,
            notes: 'Saved notes',
          ),
          meta: EbookMeta(resourceId: 'e1', author: 'Stored Author', fileFormat: 'epub'),
          locations: [
            ResourceLocation(
              id: 'loc-1',
              resourceId: 'e1',
              deviceId: 'stored-device',
              pathOrUrl: '/stored/path.epub',
              storageType: StorageType.localFs,
            ),
          ],
        ),
      ),
      updateResult: const Success(
        Resource(id: 'e1', title: 'Stored Ebook', resourceType: ResourceType.ebook),
      ),
    );
    final webReaderRepo = FakeWebReaderRepository();

    await tester.pumpWidget(
      MultiBlocProvider(
        providers: [
          BlocProvider(create: (_) => EbookBloc(ebookRepo)),
          BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
        ],
        child: const MaterialApp(
          home: AddResourceScreen(resourceId: 'e1'),
        ),
      ),
    );

    await tester.pump();

    expect(find.text('Stored Ebook'), findsOneWidget);
    expect(find.text('Stored Author'), findsOneWidget);
    expect(find.text('/stored/path.epub'), findsOneWidget);
  });
}
