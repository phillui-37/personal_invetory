import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/ebook/ebook_bloc.dart';
import '../blocs/game/game_bloc.dart';
import '../blocs/image/image_bloc.dart';
import '../blocs/tag/tag_bloc.dart';
import '../blocs/video/video_bloc.dart';
import '../blocs/web_reader/web_reader_bloc.dart';
import '../models/resources.dart';
import '../models/tag.dart';
import '../widgets/app_failure_text.dart';
import '../widgets/resource_list_item.dart';
import 'add_resource_screen.dart';
import 'resource_detail_screen.dart';

class ResourceListScreen extends StatefulWidget {
  const ResourceListScreen({super.key});

  @override
  State<ResourceListScreen> createState() => _ResourceListScreenState();
}

class _ResourceListScreenState extends State<ResourceListScreen> {
  String? _activeTagName;

  @override
  void initState() {
    super.initState();
    context.read<EbookBloc>().add(const LoadEbooks());
    context.read<WebReaderBloc>().add(const LoadWebReaders());
    context.read<ImageBloc>().add(const LoadImages());
    context.read<VideoBloc>().add(const LoadVideos());
    context.read<GameBloc>().add(const LoadGames());
    context.read<TagBloc>().add(const LoadTags());
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 5,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Resources'),
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
        body: Column(
          children: [
            BlocBuilder<TagBloc, TagState>(
              builder: (context, state) {
                if (state is! TagListLoaded || state.tags.isEmpty) {
                  return const SizedBox.shrink();
                }
                return _TagFilterBar(
                  tags: state.tags,
                  activeTagName: _activeTagName,
                  onSelect: (name) =>
                      setState(() => _activeTagName = name),
                );
              },
            ),
            Expanded(
              child: TabBarView(
                children: [
            _ResourceTab<EbookBloc, EbookState>(
              isLoading: (state) => state is EbookLoading,
              isError: (state) => state is EbookError,
              errorText: (state) => state is EbookError
                  ? appFailureMessage(state.failure)
                  : '',
              resources: (state) =>
                  state is EbookListLoaded ? state.ebooks : const <Resource>[],
              tagFilter: _activeTagName,
              onRetry: () => context.read<EbookBloc>().add(const LoadEbooks()),
              onRefresh: () async {
                context.read<EbookBloc>().add(const LoadEbooks());
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
                context.read<EbookBloc>().add(const LoadEbooks());
              },
            ),
            _ResourceTab<WebReaderBloc, WebReaderState>(
              isLoading: (state) => state is WebReaderLoading,
              isError: (state) => state is WebReaderError,
              errorText: (state) => state is WebReaderError
                  ? appFailureMessage(state.failure)
                  : '',
              resources: (state) => state is WebReaderListLoaded
                  ? state.webReaders
                  : const <Resource>[],
              tagFilter: _activeTagName,
              onRetry: () =>
                  context.read<WebReaderBloc>().add(const LoadWebReaders()),
              onRefresh: () async {
                context.read<WebReaderBloc>().add(const LoadWebReaders());
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
                context.read<WebReaderBloc>().add(const LoadWebReaders());
              },
            ),
            _ResourceTab<ImageBloc, ImageState>(
              isLoading: (state) => state is ImageLoading,
              isError: (state) => state is ImageError,
              errorText: (state) => state is ImageError
                  ? appFailureMessage(state.failure)
                  : '',
              resources: (state) =>
                  state is ImageListLoaded ? state.images : const <Resource>[],
              tagFilter: _activeTagName,
              onRetry: () => context.read<ImageBloc>().add(const LoadImages()),
              onRefresh: () async {
                context.read<ImageBloc>().add(const LoadImages());
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
                context.read<ImageBloc>().add(const LoadImages());
              },
            ),
            _ResourceTab<VideoBloc, VideoState>(
              isLoading: (state) => state is VideoLoading,
              isError: (state) => state is VideoError,
              errorText: (state) => state is VideoError
                  ? appFailureMessage(state.failure)
                  : '',
              resources: (state) =>
                  state is VideoListLoaded ? state.videos : const <Resource>[],
              tagFilter: _activeTagName,
              onRetry: () => context.read<VideoBloc>().add(const LoadVideos()),
              onRefresh: () async {
                context.read<VideoBloc>().add(const LoadVideos());
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
                context.read<VideoBloc>().add(const LoadVideos());
              },
            ),
            _ResourceTab<GameBloc, GameState>(
              isLoading: (state) => state is GameLoading,
              isError: (state) => state is GameError,
              errorText: (state) => state is GameError
                  ? appFailureMessage(state.failure)
                  : '',
              resources: (state) =>
                  state is GameListLoaded ? state.games : const <Resource>[],
              tagFilter: _activeTagName,
              onRetry: () => context.read<GameBloc>().add(const LoadGames()),
              onRefresh: () async {
                context.read<GameBloc>().add(const LoadGames());
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
                context.read<GameBloc>().add(const LoadGames());
              },
            ),
                ],
              ),
            ),
          ],
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
            context.read<EbookBloc>().add(const LoadEbooks());
            context.read<WebReaderBloc>().add(const LoadWebReaders());
            context.read<ImageBloc>().add(const LoadImages());
            context.read<VideoBloc>().add(const LoadVideos());
            context.read<GameBloc>().add(const LoadGames());
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
    required this.errorText,
    required this.resources,
    required this.onRetry,
    required this.onRefresh,
    required this.onTap,
    this.tagFilter,
  });

  final bool Function(S state) isLoading;
  final bool Function(S state) isError;
  final String Function(S state) errorText;
  final List<Resource> Function(S state) resources;
  final VoidCallback onRetry;
  final Future<void> Function() onRefresh;
  final ValueChanged<Resource> onTap;
  // When non-null, only resources matching this tag name are shown.
  // In-memory repos return all resources; HTTP repos pass ?tag= query param.
  final String? tagFilter;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<B, S>(
      builder: (context, state) {
        if (isLoading(state)) {
          return const Center(child: CircularProgressIndicator());
        }

        if (isError(state)) {
          return Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(errorText(state)),
                const SizedBox(height: 8),
                ElevatedButton(onPressed: onRetry, child: const Text('Retry')),
              ],
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

class _TagFilterBar extends StatelessWidget {
  const _TagFilterBar({
    required this.tags,
    required this.activeTagName,
    required this.onSelect,
  });

  final List<Tag> tags;
  final String? activeTagName;
  final ValueChanged<String?> onSelect;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 48,
      child: ListView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        children: [
          if (activeTagName != null)
            Padding(
              padding: const EdgeInsets.only(right: 6),
              child: InputChip(
                key: const Key('tag-filter-clear'),
                label: const Text('Clear filter'),
                onDeleted: () => onSelect(null),
                deleteIcon: const Icon(Icons.close, size: 16),
              ),
            ),
          ...tags.map(
            (tag) => Padding(
              padding: const EdgeInsets.only(right: 6),
              child: FilterChip(
                key: Key('tag-filter-${tag.name}'),
                label: Text(tag.name),
                selected: activeTagName == tag.name,
                onSelected: (_) => onSelect(
                  activeTagName == tag.name ? null : tag.name,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
