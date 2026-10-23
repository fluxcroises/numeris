# Avancement

Un critère n'est coché qu'une fois testé sur Windows et macOS par GitHub Actions (CLAUDE.md). Les critères de capture sont aussi testés sur du vrai matériel (docs/materiel-teste.md).

## Étape 1 : Socle

- [x] Projet Tauri 2 + React + TypeScript qui se compile et se lance sur Windows et macOS, avec intégration continue GitHub Actions sur les deux systèmes
- [x] FFmpeg livré avec l'application, récupéré et vérifié par script
- [ ] Liste des périphériques de capture vidéo et audio (dshow sous Windows, avfoundation sous macOS) — lecture des listes testée sur les deux systèmes, détection réelle vérifiée sous Windows seulement ; reste à essayer sur un Mac avec un boîtier
- [x] Base SQLite et migrations : cassettes, chapitres, personnes, lieux, tâches
- [x] Choix du dossier bibliothèque ; dossier par cassette avec cassette.json
- [ ] Thème Numéris (charte graphique) et barre de titre intégrée — l'application se lance sur les deux systèmes ; rendu à vérifier à l'écran, surtout la barre de titre sous macOS
- [x] Écran Bibliothèque (vide, avec « Numériser une cassette ») et écran Réglages (dossier, langue)
- [ ] Lecture d'un fichier vidéo importé (proxy généré si le format n'est pas lisible par l'interface) — copie de lecture testée sur les deux systèmes ; lecture dans la fenêtre à vérifier à la main
- [x] Interface en français et en anglais
- [x] README en français et en anglais

## Étape 2 : MVP

Ordre de travail : capture analogique d'abord (risque principal, Dazzle disponible), puis import de fichiers et chapitres automatiques, puis DV (caméscope pas encore disponible), enfin l'assistant complet et l'export.

### Capture analogique

- [x] Moteur de capture : master FFV1 + PCM en MKV (`-level 3 -g 1 -slicecrc 1`), aperçu MJPEG et mesures dans une seule commande FFmpeg
- [ ] Choix de l'entrée (S-Vidéo de préférence au composite) et de la norme (PAL par défaut)
- [ ] Aperçu en direct, vumètres, compteur, images perdues et dupliquées affichés pendant la capture
- [x] Arrêt propre à la demande : master lisible, empreinte BLAKE3 et framemd5 enregistrés
- [ ] Capture interrompue (plantage, coupure) : master lisible jusqu'à l'arrêt, base intacte
- [x] Estimation de la place avant capture ; alerte puis arrêt propre si le disque se remplit
- [ ] Arrêt automatique en fin de bande (noir, neige ou silence prolongés) ou minuterie
- [ ] Indicateur de qualité du signal pendant la capture
- [ ] Essai réel : une VHS capturée avec le Dazzle DVC100, notée dans docs/materiel-teste.md

### Import de fichiers

- [ ] Import d'un fichier (.mp4, .mov, .mkv, .avi, .vob, .dv) comme cassette : copie vérifiée par empreinte, original jamais modifié
- [ ] Copie de lecture et vignettes générées en arrière-plan, file de tâches reprise après fermeture

### Traitement et chapitres

- [ ] Désentrelacement bwdif et recadrage automatique des bords parasites
- [ ] Chapitres proposés automatiquement (plans, noirs, silences, neige)
- [ ] Fiche cassette : lecteur, chapitres avec vignettes, renommer, dater, couper, fusionner, réordonner
- [ ] Personnes et lieux sur la cassette et les chapitres ; recherche et filtres

### DV

- [ ] Analyse des fichiers .dv par dvrescue : dates d'enregistrement, timecode, erreurs ; chapitres par date
- [ ] Capture DV sous Windows en FireWire (essai réel avec le caméscope)

### Assistant et export

- [ ] Assistant en 5 étapes : Préparer, Brancher, Capturer, Organiser, Partager
- [ ] Export MP4 H.264 désentrelacé à 50 images/s, par chapitre ou cassette entière
- [ ] Bibliothèque sur disque externe : consultation quand le disque est débranché, disque retrouvé s'il change de lettre

Critère de fin du MVP : une VHS de 3 h numérisée et découpée sans intervention technique, et une cassette MiniDV capturée avec les dates de chaque plan.
