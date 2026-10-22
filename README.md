<div align="center">

<img src="src-tauri/icons/128x128.png" width="92" alt="Airo" />

# Airo

**Windows selection colors that follow your cursor between monitors.**
Give every display its own highlight color — Airo switches it automatically as your cursor moves between screens.

**v0.1.0 · Early release** — feedback and bug reports are welcome.

English · [Русский](README.ru.md)

[![Release](https://img.shields.io/github/v/release/Toveal/airo?style=flat-square&color=4a6cf7)](https://github.com/Toveal/airo/releases)
[![Downloads](https://img.shields.io/github/downloads/Toveal/airo/total?style=flat-square&color=f5a524)](https://github.com/Toveal/airo/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?style=flat-square)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB?style=flat-square)
[![License: MIT](https://img.shields.io/github/license/Toveal/airo?style=flat-square&color=97ca00)](LICENSE)

</div>

<div align="center">

<img src="docs/demo.gif" width="760" alt="Selection rectangle colors change as the cursor moves between monitors" />

[▶ Watch the demo](docs/demo.mp4)

<sub><i>Each monitor gets its own highlight color — Airo switches it as the cursor moves.</i></sub>

</div>

## What it does

Windows paints the selection rectangle — the box you drag across the desktop or in Explorer — with a single color on every display. Airo lets you set a **different color for each monitor** and applies it when the cursor enters that screen or on the first click after switching. It lives in the system tray and supports optional autostart.

Colors change **globally for your Windows session**, following the cursor or the first click, depending on your settings. Airo does not render independent colors simultaneously on different displays. You can also use it on a single monitor.

## Features

- 🎨 **A color per monitor** — outline and fill, tuned independently for each display
- 🖱️ **Automatic switching** — follows the cursor across monitors in real time
- 🌗 **Light & dark theme** — matches how you work
- 🌐 **English & Русский** — window and tray menu both localized
- 🚀 **Autostart & tray** — launches minimized, keeps out of the way
- ⬇️ **Auto-update** — built-in support for signed updates from GitHub Releases
- 🔒 **No admin required** — per-user install, no UAC prompt

## Screenshots

<table align="center">
  <tr>
    <td width="50%" valign="top">
      <img src="docs/window-en.png" alt="Airo main window" /><br/>
      <sub><i>Per-monitor outline &amp; fill colors, with a live preview.</i></sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/settings-en.png" alt="Airo settings" /><br/>
      <sub><i>Autostart, start minimized, cursor behavior and language.</i></sub>
    </td>
  </tr>
</table>

## Installation

1. Download **`Airo_0.1.0_x64-setup.exe`** from the [latest release](https://github.com/Toveal/airo/releases/latest).
2. Run it. The installer is **per-user** — it installs to `%LOCALAPPDATA%\Airo` with no administrator prompt.
3. On first launch Windows SmartScreen may warn about an unknown publisher (the app isn't code-signed with a paid certificate). Click **More info → Run anyway**.

Requires **Windows 10/11 x64** and Microsoft Edge WebView2 Runtime. Setup can download WebView2 if missing; that step needs internet access. Compatibility across Windows builds is still being verified.

## Usage

1. Pick a monitor tab and set its **outline** and **fill** colors.
2. Close the window — it tucks into the tray and keeps running.
3. Move your cursor between displays. By default, colors switch on the first click after entering another monitor; you can change this in Settings.

Everything saves as you go. Open **Settings** to toggle autostart, start-minimized, change-on-click, and the interface language.

To stop Airo and restore your original colors, choose **Quit** in the tray menu. Exit Airo before uninstalling it.

## Known limitations

- The main effect is the drag-selection rectangle on the desktop and in Explorer. Explorer’s selected-file styling is unchanged; classic Windows applications may also use the system colors Airo changes.
- Windows controls the fill opacity. The in-app preview is approximate.
- After a crash or forced termination, colors can remain changed until you relaunch Airo and quit normally.
- Avoid running multiple copies or combining Airo with another tool that changes the same system colors.

## Updating

Airo checks the GitHub Releases update feed on startup. A newer signed update can be installed from the in-app banner. Updating between two public versions has not yet been verified for this first release; you can always download the installer manually. An unavailable update feed does not prevent using the app.

## Build from source

**Requirements:** Visual Studio C++ Build Tools with Windows SDK · WebView2 Runtime · [Rust](https://www.rust-lang.org/tools/install) (stable, MSVC) · [Bun](https://bun.sh) 1.4.2 · Windows 10/11 x64

```bash
bun install --frozen-lockfile
bun run tauri dev      # run in development
bun run tauri build --config src-tauri/tauri.local.conf.json -- --locked
```

The local config builds a test installer without the maintainer’s updater signing key. For signed releases, use the default config and the private key matching the configured updater public key. See [the release guide](docs/RELEASING.md) for checks and the draft-release workflow. Updater signatures are separate from Windows publisher certificates.

**Stack:** Rust · Tauri 2 · Vue 3 · Vite

## Feedback

[Report a bug or suggest an improvement](https://github.com/Toveal/airo/issues). Include your Airo version, Windows build (`winver`), monitor setup and reproduction steps. English and Russian are welcome.

## License

Released under the [MIT License](LICENSE) © 2026 Toveal.
