import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'blocs/ebook/ebook_bloc.dart';
import 'blocs/web_reader/web_reader_bloc.dart';
import 'repositories/in_memory_repositories.dart';
import 'screens/bulk_import_screen.dart';
import 'screens/resource_list_screen.dart';
import 'screens/search_screen.dart';

void main() {
  runApp(const PersonalInventoryApp());
}

class PersonalInventoryApp extends StatelessWidget {
  const PersonalInventoryApp({super.key});

  @override
  Widget build(BuildContext context) {
    final ebookRepository = InMemoryEbookRepository();
    final webReaderRepository = InMemoryWebReaderRepository();

    return MultiBlocProvider(
      providers: [
        BlocProvider<EbookBloc>(create: (_) => EbookBloc(ebookRepository)),
        BlocProvider<WebReaderBloc>(create: (_) => WebReaderBloc(webReaderRepository)),
      ],
      child: MaterialApp(
        home: _AppShell(ebookRepository: ebookRepository),
      ),
    );
  }
}

class _AppShell extends StatefulWidget {
  const _AppShell({
    required this.ebookRepository,
  });

  final InMemoryEbookRepository ebookRepository;

  @override
  State<_AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<_AppShell> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final pages = [
      const ResourceListScreen(),
      const SearchScreen(),
      BulkImportScreen(ebookRepository: widget.ebookRepository),
    ];

    return Scaffold(
      body: pages[_index],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        onDestinationSelected: (value) => setState(() => _index = value),
        destinations: const [
          NavigationDestination(icon: Icon(Icons.inventory_2), label: 'Inventory'),
          NavigationDestination(icon: Icon(Icons.search), label: 'Search'),
          NavigationDestination(icon: Icon(Icons.batch_prediction), label: 'Batch'),
        ],
      ),
    );
  }
}
