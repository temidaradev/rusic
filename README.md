# Rusic

Rusic is a modern, lightweight music player application built with Rust and the Dioxus framework. It provides a clean and responsive interface for managing and enjoying your local music collection.

![Rusic Interface](https://github.com/user-attachments/assets/d5ece9e3-ca7a-4986-929f-c01198ab6ba1)

## Overview

Rusic allows you to scan your local directories for audio files, automatically organizing them into a browsable library. You can navigate by artists, albums, or explore your custom playlists. The application is built for performance and desktop integration, utilizing the power of Rust.

## Features

- **Library Management**: Automatically scans your music folder to populate your library with artist and album metadata.
- **Playback Control**: Full suite of media controls including play, pause, skip, volume, and seeking.
- **Fullscreen Player**: An immersive mode that focuses on album artwork and playback details.
- **Theming**: Includes dynamic theming support to customize the visual appearance.
- **Native Integration**: Integrates with system media controls (MPRIS) and "Now Playing" displays.

## Installation

### Flatpak (Recommended)

Rusic is available on Flathub (coming soon). To install from source manifest:

```bash
git clone https://github.com/temidaradev/rusic
cd rusic
flatpak-builder --user --install --force-clean build-dir com.temidaradev.rusic.json
flatpak run com.temidaradev.rusic
```

### Build from Source


If you want to hack on it:

```bash
# Install dependencies
npm install

# Run in dev mode
dx serve
```

## Tech Stack

- **Dioxus**: UI Framework
- **Rodio**: Audio playback
- **Lofty**: Metadata parsing
- **TailwindCSS**: All the styling
