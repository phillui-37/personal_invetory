import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/ebook/ebook_bloc.dart';
import '../blocs/web_reader/web_reader_bloc.dart';
import '../models/resources.dart';
import 'resource_detail_screen.dart';

class SearchScreen extends StatefulWidget {
  const SearchScreen({super.key});

  @override
  State<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends State<SearchScreen> {
  final _controller = TextEditingController();
  Timer? _debounce;

  @override
  void dispose() {
    _controller.dispose();
    _debounce?.cancel();
    super.dispose();
  }

  void _onQueryChanged(String value) {
    setState(() {});
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 400), () {
      if (value.trim().isEmpty) {
        setState(() {});
        return;
      }
      context.read<EbookBloc>().add(SearchEbooks(value));
      context.read<WebReaderBloc>().add(SearchWebReaders(value));
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Search')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: TextField(
              key: const Key('search-input'),
              controller: _controller,
              onChanged: _onQueryChanged,
              decoration: const InputDecoration(
                labelText: 'Search resources',
                border: OutlineInputBorder(),
              ),
            ),
          ),
          if (_controller.text.trim().isEmpty)
            const Expanded(
              child: Center(child: Text('Type to search…')),
            )
          else
            Expanded(
              child: BlocBuilder<EbookBloc, EbookState>(
                builder: (context, ebookState) {
                  return BlocBuilder<WebReaderBloc, WebReaderState>(
                    builder: (context, webReaderState) {
                      final loading =
                          ebookState is EbookLoading ||
                          webReaderState is WebReaderLoading;
                      if (loading) {
                        return const LinearProgressIndicator();
                      }

                      final ebookResults = ebookState is EbookListLoaded
                          ? ebookState.ebooks
                          : const <Resource>[];
                      final webReaderResults = webReaderState is WebReaderListLoaded
                          ? webReaderState.webReaders
                          : const <Resource>[];
                      final all = [...ebookResults, ...webReaderResults];

                      if (all.isEmpty) {
                        return const Center(child: Text('No results found'));
                      }

                      return ListView.builder(
                        itemCount: all.length,
                        itemBuilder: (context, index) {
                          final resource = all[index];
                          return ListTile(
                            title: Text(resource.title),
                            trailing: Chip(label: Text(resource.resourceType.name)),
                            onTap: () {
                              Navigator.of(context).push(
                                MaterialPageRoute<void>(
                                  builder: (_) => ResourceDetailScreen(
                                    resourceId: resource.id,
                                    resourceType: resource.resourceType,
                                  ),
                                ),
                              );
                            },
                          );
                        },
                      );
                    },
                  );
                },
              ),
            ),
        ],
      ),
    );
  }
}
