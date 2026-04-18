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
import 'package:personal_inventory_frontend/screens/search_screen.dart';

import '../support/fake_repositories.dart';

void main() {
  testWidgets('SearchScreen debounces and renders merged results', (tester) async {
    final ebookRepo = FakeEbookRepository(
      searchResult: const Success([
        Resource(id: 'e1', title: 'Ebook Hit', resourceType: ResourceType.ebook),
      ]),
    );
    final webReaderRepo = FakeWebReaderRepository(
      searchResult: const Success([
        Resource(
          id: 'w1',
          title: 'Web Reader Hit',
          resourceType: ResourceType.webReader,
        ),
      ]),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: MultiBlocProvider(
          providers: [
            BlocProvider(create: (_) => EbookBloc(ebookRepo)),
            BlocProvider(create: (_) => WebReaderBloc(webReaderRepo)),
            BlocProvider(create: (_) => ImageBloc(FakeImageRepository())),
            BlocProvider(create: (_) => VideoBloc(FakeVideoRepository())),
            BlocProvider(create: (_) => GameBloc(FakeGameRepository())),
          ],
          child: const SearchScreen(),
        ),
      ),
    );

    expect(find.text('Type to search…'), findsOneWidget);

    await tester.enterText(find.byKey(const Key('search-input')), 'hit');
    await tester.pump(const Duration(milliseconds: 450));
    await tester.pump();

    expect(ebookRepo.searchCalls, 1);
    expect(webReaderRepo.searchCalls, 1);
    expect(find.text('Ebook Hit'), findsOneWidget);
    expect(find.text('Web Reader Hit'), findsOneWidget);
  });
}
