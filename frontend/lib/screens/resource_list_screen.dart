import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/ebook/ebook_bloc.dart';
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
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 2,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Resources'),
          bottom: const TabBar(
            tabs: [
              Tab(text: 'Ebooks'),
              Tab(text: 'Web Readers'),
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
