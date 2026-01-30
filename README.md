# Rusic

Rusic is a modern, lightweight music player application built with Rust and the Dioxus framework. It provides a clean and responsive interface for managing and enjoying your local music collection.

![Rusic Interface](https://github.com/user-attachments/assets/8366b1ea-021f-4631-a97b-5ed6e5bf1562)

## Overview

Rusic allows you to scan your local directories for audio files, automatically organizing them into a browsable library. You can navigate by artists, albums, or explore your custom playlists. The application is built for performance and desktop integration, utilizing the power of Rust (well most of it gets demolished by webview .d).

## Features

- **Library Management**: Automatically scans your music folder to populate your library with artist and album metadata.
- **Playback Control**: Full suite of media controls including play, pause, skip, volume, and seeking.
- **Fullscreen Player**: An immersive mode that focuses on album artwork and playback details.
- **Theming**: Includes dynamic theming support to customize the visual appearance.
- **Native Integration**: Integrates with system media controls and "Now Playing" displays (only macOS for now).

## Getting Started

Do a git clone to this project to get the current code.
`git clone https://github.com/temidaradev/rusic`

Ensure you have Rust and Cargo installed on your system.

### Prerequisites

* **Rust**: Ensure you have the latest stable Rust installed. Install Rust
* **Node.js**: Required for Tailwind CSS processing. Install Node.js
* **dioxus-cli**: You can install dioxus-cli with this command `cargo install --locked dioxus-cli` (this will take a bit)
* **openssl, xdotool, glib, libsoup3**: (i dont know about what they are equal in another distros sorry, check flake.nix for all packages or just listen rust compiler for missing stuff .d)

### Quick Start

Install dependencies:
`npm i`

Run the application (dev):
Use the provided Makefile to handle CSS generation and run the app `make run`
or for better experience you can use `dx serve` for development

For building the app use `dx build --release` command and you will probably be able to run the executable in the printed location

### Cache

Rusic stores its local database, configuration files, and cached album artwork in your system's cache directory (typically `~/.cache/rusic` on macOS and Linux).

## What did i use to build this?

- **Dioxus**: For the cross-platform user interface.
- **Rodio**: For audio playback.
- **Lofty**: For reading audio metadata tags.
- **TailwindCSS**: For styling the application.
