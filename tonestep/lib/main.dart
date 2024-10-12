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

            return Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: note_utils.alteredNotes(allNotes).map((note) {
                        return Expanded(
                            child: SquareCell(
                                selected: selectedNotes.contains(note),
                                note: note,
                                onPressed: () {
                                  setState(() {
                                    if (selectedNotes.contains(note)) {
                                      selectedNotes.remove(note);
                                    } else {
                                      selectedNotes.add(note);
                                    }
                                  });
                                }));
                      }).toList()),
                  Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: note_utils.naturalNotes(allNotes).map((note) {
                        return Expanded(
                            child: SquareCell(
                                selected: selectedNotes.contains(note),
                                note: note,
                                onPressed: () {
                                  setState(() {
                                    if (selectedNotes.contains(note)) {
                                      selectedNotes.remove(note);
                                    } else {
                                      selectedNotes.add(note);
                                    }
                                  });
                                }));
                      }).toList()),
                  Row(mainAxisAlignment: MainAxisAlignment.center, children: [
                    ElevatedButton(
                        onPressed: () => api.startPlaying(notes: selectedNotes),
                        child: const Text('Play')),
                    ElevatedButton(
                      onPressed: () => api.stopPlaying(),
                      child: const Text('Stop'),
                    )
                  ])
                ]);
          } else {
            return const Text('No notes available');
          }
        },
      ),
    );
  }
}

class SquareCell extends StatelessWidget {
  const SquareCell({
    super.key,
    required this.selected,
    required this.note,
    required this.onPressed,
  });
  final bool selected;
  final notes.Note note;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        // Calculate the width for the square cell
        double cellSize = constraints.maxWidth / 8;

        return Container(
          width: cellSize,
          margin: const EdgeInsets.all(10.0),
          height: cellSize, // Make the height equal to the width for a square
          color: selected
              ? AppColors.primary
              : AppColors.secondary, // Color of the cell
          child: TextButton(
            onPressed: onPressed,
            child: Text(
              note_utils.toString(note),
              style: const TextStyle(color: Colors.white),
            ),
          ),
        );
      },
    );
  }
}
