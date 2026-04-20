import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'blocs/batch/batch_bloc.dart';
import 'blocs/dedup/dedup_bloc.dart';
import 'blocs/device/device_bloc.dart';
import 'blocs/ebook/ebook_bloc.dart';
import 'blocs/game/game_bloc.dart';
import 'blocs/image/image_bloc.dart';
import 'blocs/progress/progress_bloc.dart';
import 'blocs/sync/sync_bloc.dart';
import 'blocs/tag/tag_bloc.dart';
import 'blocs/vault/vault_bloc.dart';
import 'blocs/video/video_bloc.dart';
import 'blocs/web_reader/web_reader_bloc.dart';
import 'config/app_config.dart';
import 'config/app_theme.dart';
import 'models/resources.dart';
import 'repositories/http_batch_operation_repository.dart';
import 'repositories/http_dedup_repository.dart';
import 'repositories/http_device_repository.dart';
import 'repositories/http_sync_repository.dart';
import 'repositories/http_vault_repository.dart';
import 'repositories/in_memory_repositories.dart';
import 'screens/bulk_import_screen.dart';
import 'screens/ecosystem_screen.dart';
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
    final imageRepository = InMemoryImageRepository();
    final videoRepository = InMemoryVideoRepository();
    final gameRepository = InMemoryGameRepository();
    final progressRepository = InMemoryProgressRepository();
    final tagRepository = InMemoryTagRepository();
    final config = AppConfig.fromEnvironment();
    final vaultRepository = HttpVaultRepository(config: config);
    final dedupRepository = HttpDedupRepository(config: config);
    final syncRepository = HttpSyncRepository(config: config);
    final deviceRepository = HttpDeviceRepository(config: config);
    final batchRepository = HttpBatchOperationRepository(
      config: config,
      resourceType: ResourceType.ebook,
    );

    return MultiBlocProvider(
      providers: [
        BlocProvider<EbookBloc>(create: (_) => EbookBloc(ebookRepository)),
        BlocProvider<WebReaderBloc>(
            create: (_) => WebReaderBloc(webReaderRepository)),
        BlocProvider<ImageBloc>(create: (_) => ImageBloc(imageRepository)),
        BlocProvider<VideoBloc>(create: (_) => VideoBloc(videoRepository)),
        BlocProvider<GameBloc>(create: (_) => GameBloc(gameRepository)),
        BlocProvider<ProgressBloc>(
            create: (_) => ProgressBloc(progressRepository)),
        BlocProvider<TagBloc>(create: (_) => TagBloc(tagRepository)),
        BlocProvider<VaultBloc>(create: (_) => VaultBloc(vaultRepository)),
        BlocProvider<DedupBloc>(create: (_) => DedupBloc(dedupRepository)),
        BlocProvider<SyncBloc>(create: (_) => SyncBloc(syncRepository)),
        BlocProvider<DeviceBloc>(create: (_) => DeviceBloc(deviceRepository)),
        BlocProvider<BatchBloc>(create: (_) => BatchBloc(batchRepository)),
      ],
      child: MaterialApp(
        theme: AppTheme.lightTheme,
        darkTheme: AppTheme.darkTheme,
        themeMode: ThemeMode.system,
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
      const EcosystemScreen(),
    ];

    return Scaffold(
      body: pages[_index],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        onDestinationSelected: (value) => setState(() => _index = value),
        destinations: const [
          NavigationDestination(
              icon: Icon(Icons.inventory_2), label: 'Inventory'),
          NavigationDestination(icon: Icon(Icons.search), label: 'Search'),
          NavigationDestination(
              icon: Icon(Icons.batch_prediction), label: 'Batch'),
          NavigationDestination(
              icon: Icon(Icons.cloud_sync), label: 'Ecosystem'),
        ],
      ),
    );
  }
}
