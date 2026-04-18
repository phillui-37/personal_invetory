import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/ebook/ebook_bloc.dart';
import 'package:personal_inventory_frontend/blocs/game/game_bloc.dart';
import 'package:personal_inventory_frontend/blocs/image/image_bloc.dart';
import 'package:personal_inventory_frontend/blocs/video/video_bloc.dart';
import 'package:personal_inventory_frontend/blocs/web_reader/web_reader_bloc.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/screens/resource_list_screen.dart';

import '../support/fake_repositories.dart';

void main() {
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
      MultiBlocProvider(
        providers: [
          BlocProvider(create: (_) => EbookBloc(ebookRepo)),
          BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
          BlocProvider(create: (_) => ImageBloc(FakeImageRepository())),
          BlocProvider(create: (_) => VideoBloc(FakeVideoRepository())),
          BlocProvider(create: (_) => GameBloc(FakeGameRepository())),
        ],
        child: const MaterialApp(
          home: ResourceListScreen(),
        ),
      ),
    );

    await tester.pump();
    expect(find.text('Book One'), findsOneWidget);
    expect(find.byKey(const Key('add-resource-fab')), findsOneWidget);
  });

  testWidgets('ResourceListScreen reloads list after returning from detail', (tester) async {
    final ebookRepo = FakeEbookRepository(
      listResult: const Success([
        Resource(id: 'e1', title: 'Book One', resourceType: ResourceType.ebook),
      ]),
      detailResult: const Success(
        EbookDetail(
          resource: Resource(id: 'e1', title: 'Book One', resourceType: ResourceType.ebook),
          meta: EbookMeta(resourceId: 'e1'),
          locations: [],
        ),
      ),
    );
    final webReaderRepo = FakeWebReaderRepository(
      listResult: const Success([]),
    );

    await tester.pumpWidget(
      MultiBlocProvider(
        providers: [
          BlocProvider(create: (_) => EbookBloc(ebookRepo)),
          BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
          BlocProvider(create: (_) => ImageBloc(FakeImageRepository())),
          BlocProvider(create: (_) => VideoBloc(FakeVideoRepository())),
          BlocProvider(create: (_) => GameBloc(FakeGameRepository())),
        ],
        child: const MaterialApp(
          home: ResourceListScreen(),
        ),
      ),
    );

    await tester.pump();
    expect(ebookRepo.listCalls, 1);

    await tester.tap(find.text('Book One'));
    await tester.pumpAndSettle();

    await tester.pageBack();
    await tester.pumpAndSettle();

    expect(ebookRepo.listCalls, greaterThanOrEqualTo(2));
  });
}
