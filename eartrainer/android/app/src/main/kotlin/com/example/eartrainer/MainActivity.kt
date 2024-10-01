package com.example.eartrainer

import io.flutter.embedding.android.FlutterActivity

class MainActivity: FlutterActivity() {
  init {
    System.loadLibrary("rust_lib_eartrainer")
  }
}
