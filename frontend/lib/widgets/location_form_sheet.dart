import 'package:flutter/material.dart';

import '../models/repository_inputs.dart';
import '../models/resources.dart';

class LocationFormSheet extends StatefulWidget {
  const LocationFormSheet({
    required this.onSubmit,
    super.key,
  });

  final ValueChanged<NewLocationInput> onSubmit;

  @override
  State<LocationFormSheet> createState() => _LocationFormSheetState();
}

class _LocationFormSheetState extends State<LocationFormSheet> {
  final _deviceIdController = TextEditingController();
  final _pathController = TextEditingController();
  StorageType _storageType = StorageType.localFs;

  @override
  void dispose() {
    _deviceIdController.dispose();
    _pathController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            key: const Key('location-device-id'),
            controller: _deviceIdController,
            decoration: const InputDecoration(labelText: 'Device ID'),
          ),
          TextField(
            key: const Key('location-path-or-url'),
            controller: _pathController,
            decoration: const InputDecoration(labelText: 'Path / URL'),
          ),
          DropdownButtonFormField<StorageType>(
            key: const Key('location-storage-type'),
            initialValue: _storageType,
            decoration: const InputDecoration(labelText: 'Storage Type'),
            items: StorageType.values
                .map(
                  (type) => DropdownMenuItem<StorageType>(
                    value: type,
                    child: Text(type.name),
                  ),
                )
                .toList(),
            onChanged: (value) {
              if (value != null) {
                setState(() => _storageType = value);
              }
            },
          ),
          const SizedBox(height: 12),
          ElevatedButton(
            key: const Key('location-submit'),
            onPressed: () {
              widget.onSubmit(
                NewLocationInput(
                  deviceId: _deviceIdController.text,
                  pathOrUrl: _pathController.text,
                  storageType: _storageType,
                ),
              );
              Navigator.of(context).pop();
            },
            child: const Text('Add Location'),
          ),
        ],
      ),
    );
  }
}
