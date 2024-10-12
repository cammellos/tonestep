import 'package:flutter/material.dart';
import 'package:tonestep/src/rust/api/notes.dart' as notes;
import 'package:tonestep/src/rust/api/simple.dart' as api;
import 'package:tonestep/src/rust/api/note_utils.dart' as note_utils;
import 'package:tonestep/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(ToneStep());
}

class NotesProvider {
  Future<Set<notes.Note>> getAllNotes() async {
    return await notes.getAllNotes();
  }
}

class ToneStep extends StatelessWidget {
  final NotesProvider notesProvider = NotesProvider();

  ToneStep({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('Musical Notes')),
        body: const HomeScreen(),
      ),
    );
  }
}

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ElevatedButton(
        onPressed: () {
          // Correct context usage here
          Navigator.of(context).push(
            MaterialPageRoute(
              builder: (context) => const CreateNewExerciseScreen(),
            ),
          );
        },
        child: const Text('Create New Exercise'),
      ),
    );
  }
}

class CreateNewExerciseScreen extends StatefulWidget {
  const CreateNewExerciseScreen({super.key});

  @override
  State<CreateNewExerciseScreen> createState() =>
      _CreateNewExerciseScreenState();
}

class _CreateNewExerciseScreenState extends State<CreateNewExerciseScreen> {
  final NotesProvider notesProvider = NotesProvider();
  var selectedNotes = <notes.Note>{};
  bool _isInitialized = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('New Exercise')),
      body: FutureBuilder<Set<notes.Note>>(
        future: notesProvider.getAllNotes(),
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return const CircularProgressIndicator();
          } else if (snapshot.hasError) {
            return Text('Error: ${snapshot.error}');
          } else if (snapshot.hasData) {
            List<notes.Note> allNotes = (snapshot.data ?? {}).toList();
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
		  Row(
                      mainAxisAlignment: MainAxisAlignment.center,
		      children: [
		      TextButton(
		        onPressed: () => api.startPlaying(notes: selectedNotes),
			child: const Text('Play')
		      ),
		      TextButton(
			 onPressed: () =>  api.stopPlaying(),
		         child: const Text('Stop'),
		      )
		      ]
		    )
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
          color: selected ? Colors.green : Colors.grey, // Color of the cell
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
