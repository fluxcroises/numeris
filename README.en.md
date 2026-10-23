# Numéris

_[Version française](README.md)_

Numéris guides anyone from the tape on the table to a family film that is stored, easy to find and easy to share, with no technical knowledge needed.

- VHS, VHS-C, S-VHS, Video8, Hi8, Digital8, MiniDV, and files that are already digitized
- Windows and macOS, in English and French
- Everything stays on the computer: no video is ever sent anywhere
- An archival master, faithful to the tape, is always kept
- Free and open source (GPL v3)

> **Status: in development.** The foundation is in place (library, settings, capture hardware detection, player). Digitizing itself comes in the next stage. See [docs/avancement.md](docs/avancement.md) (in French).

## Building Numéris

Requirements:

- [Node.js](https://nodejs.org) 24
- [Rust](https://rustup.rs) stable
- The [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/) for your system (WebView2 and C++ build tools on Windows, Xcode Command Line Tools on macOS)

```bash
npm ci
node scripts/fetch-ffmpeg.mjs
npm run tauri dev
```

`scripts/fetch-ffmpeg.mjs` downloads FFmpeg 9.0.2 (GPL build), checks the archives against their SHA-256 and puts the programs in `src-tauri/resources/ffmpeg/`. The app itself downloads nothing.

Checks (the same as continuous integration):

```bash
npm run format:check && npm run lint && npm run typecheck && npm test
cd src-tauri && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Installer:

```bash
npm run tauri build
```

## Documentation

The project documents are written in French:

- [Specification](docs/cahier-des-charges.md)
- [Visual identity](docs/charte-graphique.md)
- [Decisions](docs/decisions/)
- [Tested hardware](docs/materiel-teste.md)

## License

Numéris is released under the [GNU GPL version 3 or later](LICENSE). Third-party components are listed in [NOTICE.txt](NOTICE.txt).

Numéris is meant for your personal recordings; it does not bypass any copy protection.
