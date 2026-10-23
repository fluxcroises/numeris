# Avancement

Un critère n'est coché qu'une fois testé sur Windows et macOS par GitHub Actions (CLAUDE.md). Les critères de capture sont aussi testés sur du vrai matériel (docs/materiel-teste.md).

## Étape 1 : Socle

- [ ] Projet Tauri 2 + React + TypeScript qui se compile et se lance sur Windows et macOS, avec intégration continue GitHub Actions sur les deux systèmes
- [ ] FFmpeg livré avec l'application, récupéré et vérifié par script
- [ ] Liste des périphériques de capture vidéo et audio (dshow sous Windows, avfoundation sous macOS)
- [ ] Base SQLite et migrations : cassettes, chapitres, personnes, lieux, tâches
- [ ] Choix du dossier bibliothèque ; dossier par cassette avec cassette.json
- [ ] Thème Numéris (charte graphique) et barre de titre intégrée
- [ ] Écran Bibliothèque (vide, avec « Numériser une cassette ») et écran Réglages (dossier, langue)
- [ ] Lecture d'un fichier vidéo importé (proxy généré si le format n'est pas lisible par l'interface)
- [ ] Interface en français et en anglais
- [ ] README en français et en anglais

## Étape 2 : MVP

À détailler à la fin du socle.
