import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/ebook/ebook_bloc.dart';
import '../blocs/game/game_bloc.dart';
import '../blocs/image/image_bloc.dart';
import '../blocs/video/video_bloc.dart';
import '../blocs/web_reader/web_reader_bloc.dart';
import '../models/resources.dart';
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
  @override
  void initState() {
    super.initState();
    context.read<EbookBloc>().add(const LoadEbooks());
    context.read<WebReaderBloc>().add(const LoadWebReaders());
    context.read<ImageBloc>().add(const LoadImages());
    context.read<VideoBloc>().add(const LoadVideos());
    context.read<GameBloc>().add(const LoadGames());
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
        body: TabBarView(
          children: [
            _ResourceTab<EbookBloc, EbookState>(
              isLoading: (state) => state is EbookLoading,
              isError: (state) => state is EbookError,
              errorText: (state) => state is EbookError
                  ? appFailureMessage(state.failure)
                  : '',
              resources: (state) =>
                  state is EbookListLoaded ? state.ebooks : const <Resource>[],
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
  });

  final bool Function(S state) isLoading;
  final bool Function(S state) isError;
  final String Function(S state) errorText;
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
