import 'package:flutter/material.dart';
import 'package:tonestep/src/rust/api/notes.dart' as notes;
import 'package:tonestep/src/rust/api/note_utils.dart' as noteUtils;
import 'package:tonestep/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(ToneStep());
}

class NotesProvider {
  Future<List<notes.Note>> getAllNotes() async {
    print('called');
    return await notes.getAllNotes();
  }
}

class ToneStep extends StatelessWidget {
  final NotesProvider notesProvider = NotesProvider();

  @override
  Widget build(BuildContext context) {
      return MaterialApp(
	home: Scaffold(
	  appBar: AppBar(title: Text('Musical Notes')),
	  body: FutureBuilder<List<notes.Note>>(
	    future: notesProvider.getAllNotes(),
	    builder: (context, snapshot) {
	      if (snapshot.connectionState == ConnectionState.waiting) {
		return CircularProgressIndicator();
	      } else if (snapshot.hasError) {
		return Text('Error: ${snapshot.error}');
	      } else if (snapshot.hasData) {
		return ListView.builder(
		  itemCount: snapshot.data!.length,
		  itemBuilder: (context, index) {
		    return ListTile(
		      title: Text(noteUtils.toString(snapshot.data![index])),
		    );
		  },
		);
	      } else {
		return Text('No notes available');
	      }
	    },
	  ),
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

class CreateNewExerciseScreen extends StatelessWidget {
  const CreateNewExerciseScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('New Exercise')),
      body: const Center(
        child: Text('hello'),
      ),
    );
  }
}

