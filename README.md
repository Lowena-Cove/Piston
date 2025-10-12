# Piston

An Android application that provides shortcuts for various utilities.

## Features

- **Launch SteamOS**: Launch SteamOS from a Deck.iso file
- **Termux Org Page**: Quick shortcut to open the Termux GitHub organization page (https://github.com/termux)

## Usage

1. Install the app on your Android device
2. Open the app to see the available shortcuts
3. Tap "Launch SteamOS" to launch SteamOS functionality
4. Tap "Termux Org Page" to open the Termux GitHub organization page in your default browser

## Requirements

- Android API level 28 or higher
- Internet permission for opening web links

## Build

This is a standard Android project that can be built using Android Studio or Gradle:

```bash
./gradlew assembleDebug
```

## Permissions

- `INTERNET`: Required to open web links in the browser