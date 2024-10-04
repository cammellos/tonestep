import 'package:flutter/material.dart';
import 'package:tonestep/src/rust/api/simple.dart';
import 'package:tonestep/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('flutter_rust_bridge quickstart')),
        body: Center(
	  child: ElevatedButton(
	    style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
	    child: Text('Action: Call Rust `greet("Tom")`\nResult: `${greet(name: "Tom")}`'),
            onPressed: () {playSound();},
	  ),
      ),
      )
    );
  }
}
