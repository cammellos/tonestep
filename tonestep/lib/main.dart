import 'package:flutter/material.dart';
import 'package:tonestep/src/rust/api/notes.dart' as notes;
import 'package:tonestep/src/rust/api/simple.dart' as api;
import 'package:tonestep/src/rust/api/note_utils.dart' as note_utils;
import 'package:tonestep/src/rust/frb_generated.dart';
import 'package:tonestep/components/theme.dart';
import 'package:flutter/services.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(ToneStep());

  List<Uint8List> wavDataList = [];

  for (int i = 1; i <= 12; i++) {
    ByteData data = await rootBundle.load('rust/resources/$i.wav');
    Uint8List bytes = data.buffer.asUint8List();
    wavDataList.add(bytes);
  }

  api.initWavFilesFromBytes(wavData: wavDataList);
}

class NotesProvider {
  Future<List<notes.Note>> allNotes() async {
    return await notes.allNotes();
  }
}

class ToneStep extends StatelessWidget {
  final NotesProvider notesProvider = NotesProvider();

  ToneStep({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      theme: AppTheme.themeData,
      home: const Scaffold(
        body: HomeScreenScreen(),
      ),
    );
  }
}

class HomeScreenScreen extends StatefulWidget {
  const HomeScreenScreen({super.key});

  @override
  State<HomeScreenScreen> createState() => _HomeScreenScreenState();
}

class _HomeScreenScreenState extends State<HomeScreenScreen> {
  final NotesProvider notesProvider = NotesProvider();
  var selectedNotes = <notes.Note>{};
  bool _isInitialized = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: FutureBuilder<List<notes.Note>>(
        future: notesProvider.allNotes(),
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return const CircularProgressIndicator();
          } else if (snapshot.hasError) {
            return Text('Error: ${snapshot.error}');
          } else if (snapshot.hasData) {
            List<notes.Note> allNotes = (snapshot.data ?? []);
            if (!_isInitialized) {
              selectedNotes.addAll(allNotes);
              _isInitialized = true;
            }

            return Container(
                padding: const EdgeInsets.all(10),
                child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Row(
                          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                          children: allNotes.take(4).map((note) {
                            return SquareButton(
                                selected: selectedNotes.contains(note),
                                label: note_utils.toString(note),
                                onPressed: () {
                                  setState(() {
                                    if (selectedNotes.contains(note)) {
                                      selectedNotes.remove(note);
                                    } else {
                                      selectedNotes.add(note);
                                    }
                                  });
                                });
                          }).toList()),
                      const SizedBox(height: 10),
                      Row(
                          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                          children: allNotes.skip(4).take(4).map((note) {
                            return SquareButton(
                                selected: selectedNotes.contains(note),
                                label: note_utils.toString(note),
                                onPressed: () {
                                  setState(() {
                                    if (selectedNotes.contains(note)) {
                                      selectedNotes.remove(note);
                                    } else {
                                      selectedNotes.add(note);
                                    }
                                  });
                                });
                          }).toList()),
                      const SizedBox(height: 10),
                      Row(
                          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                          children: allNotes.skip(8).take(4).map((note) {
                            return SquareButton(
                                selected: selectedNotes.contains(note),
                                label: note_utils.toString(note),
                                onPressed: () {
                                  setState(() {
                                    if (selectedNotes.contains(note)) {
                                      selectedNotes.remove(note);
                                    } else {
                                      selectedNotes.add(note);
                                    }
                                  });
                                });
                          }).toList()),
                      const SizedBox(height: 10),
                      Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            ElevatedButton(
                                style: ElevatedButton.styleFrom(
                                  backgroundColor: AppColors.primary,
                                ),
                                onPressed: () =>
                                    api.startPlaying(notes: selectedNotes),
                                child: const Text('Play',
                                    style: TextStyle(
                                        fontSize: 30, color: Colors.white))),
                            const SizedBox(width: 20),
                            ElevatedButton(
                                style: ElevatedButton.styleFrom(
                                  backgroundColor: AppColors.error,
                                ),
                                onPressed: () => api.stopPlaying(),
                                child: const Text('Stop',
                                    style: TextStyle(
                                        fontSize: 30, color: Colors.white))),
                          ])
                    ]));
          } else {
            return const Text('No notes available');
          }
        },
      ),
    );
  }
}

class SquareButton extends StatelessWidget {
  final String label;
  final VoidCallback onPressed;
  final bool selected;

  SquareButton(
      {required this.label, required this.onPressed, required this.selected});

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 5.0),
        child: AspectRatio(
          aspectRatio: 1, // Makes the button square
          child: ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor:
                  selected ? AppColors.primary : AppColors.secondary,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(12), // Rounded corners
              ),
            ),
            onPressed: onPressed,
            child: Text(label,
                style: TextStyle(
                    fontSize: 30,
                    fontWeight: FontWeight.bold,
                    color: selected
                        ? AppColors.onPrimary
                        : AppColors.onSecondary)),
          ),
        ),
      ),
    );
  }
}
