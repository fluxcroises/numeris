# Numéris

_[English version](README.en.md)_

Numéris guide n'importe qui de la cassette posée sur la table jusqu'au film de famille rangé, retrouvable et partageable, sans connaissances techniques.

- VHS, VHS-C, S-VHS, Video8, Hi8, Digital8, MiniDV, et fichiers déjà numérisés
- Windows et macOS, en français et en anglais
- Tout reste sur l'ordinateur : aucune vidéo n'est envoyée nulle part
- Un master d'archive fidèle à la bande est toujours conservé
- Gratuit et open source (GPL v3)

> **État : en développement.** Le socle est en place (bibliothèque, réglages, détection du matériel de capture, lecteur). La numérisation elle-même arrive à l'étape suivante. Voir [docs/avancement.md](docs/avancement.md).

## Compiler Numéris

Prérequis :

- [Node.js](https://nodejs.org) 24
- [Rust](https://rustup.rs) stable
- Les [prérequis de Tauri 2](https://tauri.app/start/prerequisites/) pour votre système (WebView2 et outils de compilation C++ sous Windows, Xcode Command Line Tools sous macOS)

```bash
npm ci
node scripts/fetch-ffmpeg.mjs
npm run tauri dev
```

`scripts/fetch-ffmpeg.mjs` télécharge FFmpeg 9.0.2 (version GPL), vérifie les archives par SHA-256 et place les programmes dans `src-tauri/resources/ffmpeg/`. L'application elle-même ne télécharge rien.

Vérifications (les mêmes que l'intégration continue) :

```bash
npm run format:check && npm run lint && npm run typecheck && npm test
cd src-tauri && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Installateur :

```bash
npm run tauri build
```

## Documentation

- [Cahier des charges](docs/cahier-des-charges.md)
- [Charte graphique](docs/charte-graphique.md)
- [Décisions](docs/decisions/)
- [Matériel testé](docs/materiel-teste.md)

## Licence

Numéris est distribué sous licence [GNU GPL version 3 ou ultérieure](LICENSE). Les composants tiers sont listés dans [NOTICE.txt](NOTICE.txt).

Numéris est destiné à vos enregistrements personnels ; il ne contourne aucune protection contre la copie.
