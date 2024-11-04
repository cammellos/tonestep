package com.example.tonestep

import io.flutter.embedding.android.FlutterActivity

class MainActivity: FlutterActivity() {
  init {
    System.loadLibrary("rust_lib_tonestep")
  }
}
