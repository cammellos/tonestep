import 'package:tonestep/src/rust/api/notes.dart';

bool isNaturalNote(Note note) {
  switch (note) {
    case Note.one:
    case Note.two:
    case Note.four:
    case Note.five:
    case Note.seven:
	return true;
    default:
        return false;
  }
}

List<Note> naturalNotes(List<Note> notes) {
  return notes.where((n) => isNaturalNote(n)).toList();
}

List<Note> alteredNotes(List<Note> notes) {
  return notes.where((n) => !isNaturalNote(n)).toList();
}

String toString(Note note) {
  switch (note) {
    case Note.one:
	return '1';
    case Note.flatTwo:
	return 'b2';
    case Note.two:
        return '2';
    case Note.flatThree:
        return 'b3';
    case Note.four:
	return '4';
    case Note.sharpFour:
	return '#4';
    case Note.five:
	return '5';
    case Note.flatSix:
	return 'b6';
    case Note.flatSeven:
	return 'b7';
    case Note.seven:
	return '7';
    default:
        throw Exception('invalid note ${note}');
  }
}
