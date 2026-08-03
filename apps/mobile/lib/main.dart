import 'package:flutter/material.dart';

void main() {
  runApp(const PanagangApp());
}

/// Phase 0 placeholder shell. Feature screens land in Phase 5.
class PanagangApp extends StatelessWidget {
  const PanagangApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Panagang PH',
      home: Scaffold(
        appBar: AppBar(title: const Text('Panagang PH')),
        body: const Center(
          child: Text('Scam protection MVP — scaffold ready'),
        ),
      ),
    );
  }
}
