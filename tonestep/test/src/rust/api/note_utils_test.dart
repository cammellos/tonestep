import 'package:flutter_test/flutter_test.dart'; // Use flutter_test for Flutter-specific tests
import 'package:tonestep/src/rust/api/note_utils.dart'; // Adjust based on the actual import path
import 'package:tonestep/src/rust/api/notes.dart'; // Import Note definitions

void main() {
  group('isNaturalNote tests', () {
    test('returns true for natural notes', () {
      expect(isNaturalNote(Note.one), isTrue);
      expect(isNaturalNote(Note.two), isTrue);
      expect(isNaturalNote(Note.three), isTrue);
      expect(isNaturalNote(Note.four), isTrue);
      expect(isNaturalNote(Note.five), isTrue);
      expect(isNaturalNote(Note.six), isTrue);
      expect(isNaturalNote(Note.seven), isTrue);
    });

    test('returns false for augmented notes', () {
      expect(isNaturalNote(Note.flatTwo), isFalse);
      expect(isNaturalNote(Note.flatThree), isFalse);
      expect(isNaturalNote(Note.sharpFour), isFalse);
      expect(isNaturalNote(Note.flatSix), isFalse);
      expect(isNaturalNote(Note.flatSeven), isFalse);
    });
  });

  group('naturalNotes tests', () {
    test('filters natural notes from a mixed list', () {
      final notes = [
        Note.one,
        Note.flatTwo,
        Note.two,
        Note.flatThree,
	Note.three,
        Note.four,
        Note.sharpFour,
        Note.five,
        Note.flatSix,
	Note.six,
        Note.flatSeven,
        Note.seven,
      ];

      final expectedNaturalNotes = [
        Note.one,
        Note.two,
	Note.three,
        Note.four,
        Note.five,
	Note.six,
        Note.seven,
      ];

      expect(naturalNotes(notes), equals(expectedNaturalNotes));
    });

    test('returns empty list when there are no natural notes', () {
      final notes = [Note.flatTwo, Note.flatThree, Note.sharpFour, Note.flatSix, Note.flatSeven];

      expect(naturalNotes(notes), isEmpty);
    });
  });

  group('alteredNotes tests', () {
    test('filters altered notes from a mixed list', () {
      final notes = [
        Note.one,
        Note.flatTwo,
        Note.two,
        Note.flatThree,
	Note.three,
        Note.four,
        Note.sharpFour,
        Note.five,
        Note.flatSix,
	Note.six,
        Note.flatSeven,
        Note.seven,
      ];

      final expectedAlteredNotes = [
        Note.flatTwo,
        Note.flatThree,
        Note.sharpFour,
        Note.flatSix,
        Note.flatSeven,
      ];

      expect(alteredNotes(notes), equals(expectedAlteredNotes));
    });

    test('returns empty list when there are no altered notes', () {
      final notes = [Note.one, Note.two, Note.four, Note.five, Note.seven];

      expect(alteredNotes(notes), isEmpty);
    });
  });

  group('toString tests', () {
    test('correctly converts notes to strings', () {
      expect(toString(Note.one), equals('1'));
      expect(toString(Note.flatTwo), equals('b2'));
      expect(toString(Note.two), equals('2'));
      expect(toString(Note.flatThree), equals('b3'));
      expect(toString(Note.three), equals('3'));
      expect(toString(Note.four), equals('4'));
      expect(toString(Note.sharpFour), equals('#4'));
      expect(toString(Note.five), equals('5'));
      expect(toString(Note.six), equals('6'));
      expect(toString(Note.flatSix), equals('b6'));
      expect(toString(Note.flatSeven), equals('b7'));
      expect(toString(Note.seven), equals('7'));
    });
  });
}
