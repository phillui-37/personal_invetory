import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/device/device_bloc.dart';
import '../models/device.dart';

class DeviceManagementScreen extends StatelessWidget {
  const DeviceManagementScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Device Management')),
      body: BlocConsumer<DeviceBloc, DeviceState>(
        listener: (context, state) {
          if (state is DeviceOperationSuccess) {
            context.read<DeviceBloc>().add(const LoadDevices());
          }
        },
        builder: (context, state) {
          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              _RegisterDeviceForm(),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () =>
                    context.read<DeviceBloc>().add(const LoadDevices()),
                child: const Text('Refresh'),
              ),
              const SizedBox(height: 16),
              if (state is DeviceLoading)
                const Center(child: CircularProgressIndicator())
              else if (state is DeviceListLoaded)
                ...state.devices.map((d) => _DeviceTile(device: d))
              else if (state is DeviceError)
                Text(
                  'Error: ${state.failure}',
                  style: const TextStyle(color: Colors.red),
                ),
            ],
          );
        },
      ),
    );
  }
}

class _RegisterDeviceForm extends StatefulWidget {
  @override
  State<_RegisterDeviceForm> createState() => _RegisterDeviceFormState();
}

class _RegisterDeviceFormState extends State<_RegisterDeviceForm> {
  final _deviceIdCtrl = TextEditingController();
  final _deviceNameCtrl = TextEditingController();

  @override
  void dispose() {
    _deviceIdCtrl.dispose();
    _deviceNameCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Register Device',
          style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        TextField(
          controller: _deviceIdCtrl,
          decoration: const InputDecoration(
            labelText: 'Device ID',
            hintText: 'e.g. desktop-home',
          ),
        ),
        const SizedBox(height: 8),
        TextField(
          controller: _deviceNameCtrl,
          decoration: const InputDecoration(
            labelText: 'Display Name (optional)',
          ),
        ),
        const SizedBox(height: 8),
        ElevatedButton(
          onPressed: () {
            final id = _deviceIdCtrl.text.trim();
            if (id.isEmpty) return;
            final name = _deviceNameCtrl.text.trim();
            context.read<DeviceBloc>().add(
                  RegisterDevice(
                    deviceId: id,
                    deviceName: name.isEmpty ? null : name,
                  ),
                );
          },
          child: const Text('Register'),
        ),
      ],
    );
  }
}

class _DeviceTile extends StatelessWidget {
  const _DeviceTile({required this.device});
  final Device device;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: ListTile(
        leading: Icon(
          device.isActive ? Icons.computer : Icons.computer_outlined,
          color: device.isCurrent ? Colors.blue : null,
        ),
        title: Text(device.deviceName),
        subtitle: Text(
          '${device.locationCount} location(s)'
          '${device.isCurrent ? ' · current' : ''}'
          '${!device.isActive ? ' · delinked' : ''}',
        ),
        trailing: (device.isActive && !device.isCurrent)
            ? TextButton(
                onPressed: () => context
                    .read<DeviceBloc>()
                    .add(DelinkDevice(device.deviceId)),
                child: const Text('Delink'),
              )
            : null,
      ),
    );
  }
}
