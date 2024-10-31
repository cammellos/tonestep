# Tonestep

An Android and desktop application designed for ear training. This app plays a consistent drone note and then overlays additional notes, helping users develop their pitch recognition and musical ear. It is ideal for musicians looking to improve their listening skills through interval recognition, scale degree identification, and tonal awareness.

## Features

- **Drone with Overlayed Notes**: Plays a continuous drone followed by designated notes to train users' pitch recognition.
- **Cross-Platform**: Supports both Android and desktop environments.
- **Device Selection**: Flexibility to run the app on a specific device if desired.

## Usage

The Makefile includes a target to run the Flutter app. To run it:

1. **Run on Default Device**: If no device is specified, the app will run on the default device detected by Flutter.
2. **Specify a Device**: Run the app on a particular device by setting the `DEVICE` variable.

### Command

To execute the Flutter app:

```
make run
```

To specify a device:

```
make run DEVICE=<device_id>
```

### Example

```
make run DEVICE=emulator-5554
```

## Target Details

- **run**: Target to start the Flutter app after copying libraries.

If `DEVICE` is set, the app runs on the specified device. Otherwise, it runs on the default connected device.

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository.
2. Create a new feature branch (`git checkout -b feature-name`).
3. Commit your changes (`git commit -m 'Add feature name'`).
4. Push to the branch (`git push origin feature-name`).
5. Open a Pull Request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
