# 0001. FFmpeg livré avec Numéris

Date : 2026-10-23 · Statut : **adoptée** le 2026-10-23, avec la licence GPL-3.0-or-later du projet

## Contexte

FFmpeg est le moteur de Numéris : capture (DirectShow, AVFoundation), master FFV1, analyse (`signalstats`, `scdet`, `blackdetect`, `silencedetect`), proxys et exports H.264. Les exports doivent être de bonne qualité, ce qui demande x264, sous licence GPL.

## Décision

Reprise de la décision 0012 d'Afterdusk, pour partager les mêmes binaires vérifiés :

- **FFmpeg 9.0.2**, ffmpeg et ffprobe utilisés comme programmes séparés, appelés par le cœur Rust.
- **Windows** : construction GPL en bibliothèques partagées de BtbN/FFmpeg-Builds (`autobuild-2026-09-30-13-08`).
- **macOS** : constructions statiques de Martin Riedl, Apple Silicon et Intel, réunies en binaires universels par `lipo`.
- **Récupération** : `scripts/fetch-ffmpeg.mjs`, archives vérifiées par SHA-256, binaires extraits dans `src-tauri/resources/ffmpeg/` (non versionné) et livrés par Tauri. L'application ne télécharge rien.
- **Licence** : FFmpeg sous GPL v2 ou ultérieure dans ces constructions (x264 compris), compatible avec la GPL v3 de Numéris. `NOTICE.txt` donne l'adresse du code source correspondant.

## Conséquences

- Les filtres nécessaires (`bwdif`, `signalstats`, `scdet`, `idet`, encodeurs `ffv1`, `libx264`) sont à vérifier dans ces constructions lors du MVP.
- Une construction réduite pourra alléger l'installateur plus tard.
- dvrescue (analyse DV) fera l'objet d'une décision séparée.
