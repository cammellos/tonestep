// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.5.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

// These function are ignored because they are on traits that is not defined in current crate (put an empty `#[frb]` on it to unignore): `assert_receiver_is_total_eq`, `clone`, `eq`, `fmt`, `fmt`, `hash`

Future<Set<Note>> getAllNotes() =>
    RustLib.instance.api.crateApiNotesGetAllNotes();

Future<List<Note>> allNotes() => RustLib.instance.api.crateApiNotesAllNotes();

Future<void> stop() => RustLib.instance.api.crateApiNotesStop();

Future<void> playExercise() => RustLib.instance.api.crateApiNotesPlayExercise();

enum Note {
  one,
  flatTwo,
  two,
  three,
  flatThree,
  four,
  sharpFour,
  five,
  flatSix,
  six,
  flatSeven,
  seven,
  ;

  static Future<Note> fromNumber({required int n}) =>
      RustLib.instance.api.crateApiNotesNoteFromNumber(n: n);

  Future<int> toKeyboardC1Note() =>
      RustLib.instance.api.crateApiNotesNoteToKeyboardC1Note(
        that: this,
      );

  Future<int> toKeyboardC5Note() =>
      RustLib.instance.api.crateApiNotesNoteToKeyboardC5Note(
        that: this,
      );

  Future<int> toKeyboardNote() =>
      RustLib.instance.api.crateApiNotesNoteToKeyboardNote(
        that: this,
      );
}
