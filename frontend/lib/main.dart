import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'blocs/ebook/ebook_bloc.dart';
import 'blocs/web_reader/web_reader_bloc.dart';
import 'models/batch_operations.dart';
import 'repositories/in_memory_repositories.dart';
import 'screens/batch_operations_screen.dart';
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
        home: const _AppShell(),
      ),
    );
  }
}

class _AppShell extends StatefulWidget {
  const _AppShell();

  @override
  State<_AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<_AppShell> {
  final _batchRepository = const InMemoryBatchOperationRepository();
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final pages = [
      const ResourceListScreen(),
      const SearchScreen(),
      BatchOperationsScreen(
        onImport: (request) {
          unawaited(_batchRepository.batchImport(request));
          _showBatchSnackBar(BatchOperationType.importResources);
        },
        onUpdate: (request) {
          unawaited(_batchRepository.batchUpdateMetadata(request));
          _showBatchSnackBar(BatchOperationType.updateMetadata);
        },
        onCopy: (request) {
          unawaited(_batchRepository.batchCopyMetadata(request));
          _showBatchSnackBar(BatchOperationType.copyMetadata);
        },
      ),
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

  void _showBatchSnackBar(BatchOperationType type) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text('Batch request submitted: ${type.name}')));
  }
}
