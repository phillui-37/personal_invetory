import 'package:flutter/material.dart';

import '../models/resources.dart';

class ResourceListItem extends StatelessWidget {
  const ResourceListItem({
    required this.resource,
    required this.onTap,
    super.key,
  });

  final Resource resource;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: ListTile(
        title: Text(resource.title),
        subtitle: Text(resource.notes ?? ''),
        trailing: Chip(
          label: Text(
            'storage: n/a',
          ),
        ),
        onTap: onTap,
      ),
    );
  }
}
