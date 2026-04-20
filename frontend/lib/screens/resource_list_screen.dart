import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/batch/batch_bloc.dart';
import '../blocs/ebook/ebook_bloc.dart';
import '../blocs/game/game_bloc.dart';
import '../blocs/image/image_bloc.dart';
import '../blocs/search_filter/search_filter_bloc.dart';
import '../blocs/tag/tag_bloc.dart';
import '../blocs/video/video_bloc.dart';
import '../blocs/web_reader/web_reader_bloc.dart';
import '../models/failures.dart';
import '../models/resources.dart';
import '../models/tag.dart';
import '../widgets/error_display.dart';
import '../widgets/resource_list_item.dart';
import '../widgets/search_filter_bar.dart';
import 'add_resource_screen.dart';
import 'batch_operations_screen.dart';
import 'resource_detail_screen.dart';

class ResourceListScreen extends StatefulWidget {
  const ResourceListScreen({super.key});

  @override
  State<ResourceListScreen> createState() => _ResourceListScreenState();
}

class _ResourceListScreenState extends State<ResourceListScreen> {
  void _loadAllResources(BuildContext context) {
    final f = context.read<SearchFilterBloc>().state;
    context.read<EbookBloc>().add(LoadEbooks(
      tags: f.selectedTags, sortBy: f.sortBy, sortOrder: f.sortOrder, filterLogic: f.filterLogic,
    ));
    context.read<WebReaderBloc>().add(LoadWebReaders(
      tags: f.selectedTags, sortBy: f.sortBy, sortOrder: f.sortOrder, filterLogic: f.filterLogic,
    ));
    context.read<ImageBloc>().add(LoadImages(
      tags: f.selectedTags, sortBy: f.sortBy, sortOrder: f.sortOrder, filterLogic: f.filterLogic,
    ));
    context.read<VideoBloc>().add(LoadVideos(
      tags: f.selectedTags, sortBy: f.sortBy, sortOrder: f.sortOrder, filterLogic: f.filterLogic,
    ));
    context.read<GameBloc>().add(LoadGames(
      tags: f.selectedTags, sortBy: f.sortBy, sortOrder: f.sortOrder, filterLogic: f.filterLogic,
    ));
  }

  @override
  void initState() {
    super.initState();
    _loadAllResources(context);
    context.read<TagBloc>().add(const LoadTags());
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 5,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Resources'),
          actions: [
            IconButton(
              key: const Key('batch-operations-icon'),
              icon: const Icon(Icons.library_books),
              tooltip: 'Batch operations',
              onPressed: () {
                final batchBloc = context.read<BatchBloc>();
                Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => BlocProvider.value(
                      value: batchBloc,
                      child: BatchOperationsScreen(
                        onImport: (req) => batchBloc.add(
                          BatchImportRequested(req),
                        ),
                        onUpdate: (req) => batchBloc.add(
                          BatchUpdateMetadataRequested(req),
                        ),
                        onCopy: (req) => batchBloc.add(
                          BatchCopyMetadataRequested(req),
                        ),
                      ),
                    ),
                  ),
                );
              },
            ),
          ],
          bottom: const TabBar(
            isScrollable: true,
            tabs: [
              Tab(text: 'Ebooks'),
              Tab(text: 'Web Readers'),
              Tab(text: 'Images'),
              Tab(text: 'Videos'),
              Tab(text: 'Games'),
            ],
          ),
        ),
        body: BlocListener<SearchFilterBloc, SearchFilterState>(
          listener: (context, state) {
            _loadAllResources(context);
          },
          child: Column(
          children: [
            BlocBuilder<TagBloc, TagState>(
              builder: (context, tagState) {
                if (tagState is! TagListLoaded || tagState.tags.isEmpty) {
                  return const SizedBox.shrink();
                }
                return BlocBuilder<SearchFilterBloc, SearchFilterState>(
                  builder: (context, filterState) {
                    return SearchFilterBar(
                      tags: tagState.tags,
                      selectedTags: filterState.selectedTags,
                      onTagsChanged: (tags) => context.read<SearchFilterBloc>().add(UpdateSelectedTags(tags)),
                      onSortChanged: (sort) => context.read<SearchFilterBloc>().add(UpdateSortBy(sort)),
                      onLogicChanged: (logic) => context.read<SearchFilterBloc>().add(UpdateFilterLogic(logic)),
                    );
                  },
                );
              },
            ),
            Expanded(
              child: TabBarView(
                children: [
            _ResourceTab<EbookBloc, EbookState>(
              isLoading: (state) => state is EbookLoading,
              isError: (state) => state is EbookError,
              failure: (state) => state is EbookError ? state.failure : null,
              resources: (state) =>
                  state is EbookListLoaded ? state.ebooks : const <Resource>[],
              onRetry: () => _loadAllResources(context),
              onRefresh: () async {
                _loadAllResources(context);
              },
              onTap: (resource) async {
                await Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => ResourceDetailScreen(
                      resourceId: resource.id,
                      resourceType: ResourceType.ebook,
                    ),
                  ),
                );
                if (!context.mounted) {
                  return;
                }
                _loadAllResources(context);
              },
            ),
            _ResourceTab<WebReaderBloc, WebReaderState>(
              isLoading: (state) => state is WebReaderLoading,
              isError: (state) => state is WebReaderError,
              failure: (state) => state is WebReaderError ? state.failure : null,
              resources: (state) => state is WebReaderListLoaded
                  ? state.webReaders
                  : const <Resource>[],
              onRetry: () => _loadAllResources(context),
              onRefresh: () async {
                _loadAllResources(context);
              },
              onTap: (resource) async {
                await Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => ResourceDetailScreen(
                      resourceId: resource.id,
                      resourceType: ResourceType.webReader,
                    ),
                  ),
                );
                if (!context.mounted) {
                  return;
                }
                _loadAllResources(context);
              },
            ),
            _ResourceTab<ImageBloc, ImageState>(
              isLoading: (state) => state is ImageLoading,
              isError: (state) => state is ImageError,
              failure: (state) => state is ImageError ? state.failure : null,
              resources: (state) =>
                  state is ImageListLoaded ? state.images : const <Resource>[],
              onRetry: () => _loadAllResources(context),
              onRefresh: () async {
                _loadAllResources(context);
              },
              onTap: (resource) async {
                await Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => ResourceDetailScreen(
                      resourceId: resource.id,
                      resourceType: ResourceType.image,
                    ),
                  ),
                );
                if (!context.mounted) {
                  return;
                }
                _loadAllResources(context);
              },
            ),
            _ResourceTab<VideoBloc, VideoState>(
              isLoading: (state) => state is VideoLoading,
              isError: (state) => state is VideoError,
              failure: (state) => state is VideoError ? state.failure : null,
              resources: (state) =>
                  state is VideoListLoaded ? state.videos : const <Resource>[],
              onRetry: () => _loadAllResources(context),
              onRefresh: () async {
                _loadAllResources(context);
              },
              onTap: (resource) async {
                await Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => ResourceDetailScreen(
                      resourceId: resource.id,
                      resourceType: ResourceType.video,
                    ),
                  ),
                );
                if (!context.mounted) {
                  return;
                }
                _loadAllResources(context);
              },
            ),
            _ResourceTab<GameBloc, GameState>(
              isLoading: (state) => state is GameLoading,
              isError: (state) => state is GameError,
              failure: (state) => state is GameError ? state.failure : null,
              resources: (state) =>
                  state is GameListLoaded ? state.games : const <Resource>[],
              onRetry: () => _loadAllResources(context),
              onRefresh: () async {
                _loadAllResources(context);
              },
              onTap: (resource) async {
                await Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => ResourceDetailScreen(
                      resourceId: resource.id,
                      resourceType: ResourceType.game,
                    ),
                  ),
                );
                if (!context.mounted) {
                  return;
                }
                _loadAllResources(context);
              },
            ),
                ],
              ),
            ),
          ],
        ),
        ),
        floatingActionButton: FloatingActionButton(
          key: const Key('add-resource-fab'),
          onPressed: () async {
            await Navigator.of(context).push(
              MaterialPageRoute<void>(
                builder: (_) => const AddResourceScreen(),
              ),
            );
            if (!context.mounted) {
              return;
            }
            _loadAllResources(context);
          },
          child: const Icon(Icons.add),
        ),
      ),
    );
  }
}

class _ResourceTab<B extends StateStreamable<S>, S> extends StatelessWidget {
  const _ResourceTab({
    required this.isLoading,
    required this.isError,
    required this.failure,
    required this.resources,
    required this.onRetry,
    required this.onRefresh,
    required this.onTap,
  });

  final bool Function(S state) isLoading;
  final bool Function(S state) isError;
  final AppFailure? Function(S state) failure;
  final List<Resource> Function(S state) resources;
  final VoidCallback onRetry;
  final Future<void> Function() onRefresh;
  final ValueChanged<Resource> onTap;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<B, S>(
      builder: (context, state) {
        if (isLoading(state)) {
          return const Center(child: CircularProgressIndicator());
        }

        if (isError(state)) {
          final err = failure(state);
          if (err == null) {
            return const Center(child: Text('Unknown error occurred'));
          }
          return Center(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: ErrorDisplay(
                failure: err,
                onRetry: onRetry,
              ),
            ),
          );
        }

        final items = resources(state);
        if (items.isEmpty) {
          return const Center(child: Text('No resources yet'));
        }

        return RefreshIndicator(
          onRefresh: onRefresh,
          child: ListView.builder(
            itemCount: items.length,
            itemBuilder: (context, index) {
              final resource = items[index];
              return ResourceListItem(
                resource: resource,
                onTap: () => onTap(resource),
              );
            },
          ),
        );
      },
    );
  }
}
