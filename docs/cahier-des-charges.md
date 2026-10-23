# Numéris, cahier des charges

Version 2 · 23 octobre 2026 · @Lewis
Version 1 du 23 octobre 2026 revue après étude technique : les décisions prises sont listées en section 8.

## 1. Vision et principes

Numéris guide n'importe qui de la cassette posée sur la table jusqu'au film de famille rangé, retrouvable et partageable, sans connaissances techniques. C'est l'application sœur d'Afterdusk : même socle technique, mais un logiciel indépendant avec sa propre identité visuelle.

- **Gratuit et open source** (GPL-3.0-or-later), Windows et macOS, interface en français et en anglais.
- **Tout en local** : aucune vidéo ne quitte l'ordinateur, IA comprise. Ce sont des souvenirs de famille.
- **Simple d'abord** : un assistant pas à pas pour le grand public, des réglages avancés repliés pour les passionnés.
- **Archive avant tout** : on conserve toujours un master fidèle à la bande, les versions « jolies » sont des copies.
- **Supports visés** : VHS, VHS-C, S-VHS, Video8, Hi8, Digital8, MiniDV, plus l'import de fichiers déjà numérisés.

## 2. Supports et matériel pris en charge

Trois voies d'entrée, toutes aboutissant au même catalogue.

| Voie | Supports | Matériel | Windows | macOS |
| --- | --- | --- | --- | --- |
| Capture analogique | VHS, VHS-C, S-VHS, Video8, Hi8 | Magnétoscope ou caméscope + boîtier USB (composite ou S-Vidéo) | Oui (FFmpeg dshow) | Oui pour les boîtiers reconnus sans pilote (UVC), via FFmpeg avfoundation |
| Capture DV | MiniDV, Digital8 | Caméscope relié en FireWire (IEEE 1394) | Oui, via port ou carte FireWire (puce Texas Instruments conseillée) | Non prévu : FireWire absent des Mac récents et, semble-t-il, retiré de macOS 26 (à vérifier). Import de fichiers .dv capturés ailleurs |
| Import de fichiers | Tous | Fichiers .dv, .avi, .mp4, .mov, .mkv, .vob | Oui | Oui |

Numéris détecte le matériel branché, propose la bonne entrée (S-Vidéo de préférence au composite) et la bonne norme (PAL par défaut en France, NTSC possible).

Une page d'aide intégrée conseille le matériel : boîtiers USB testés, intérêt d'un magnétoscope avec correcteur de base de temps (TBC), câbles à privilégier, cartes FireWire compatibles.

### Matériel de test disponible

- PC Windows 11 avec contrôleur FireWire Texas Instruments OHCI : capture DV testable dès le MVP.
- Caméscope DV.
- Pinnacle Dazzle DVC 90, 100 ou 101 (modèle exact à confirmer). Ces modèles n'ont pas de pilote officiel pour Windows 10 et 11 en 64 bits : s'il n'est pas reconnu, il faudra un boîtier UVC récent, qui servira aussi de référence pour la liste de matériel conseillé.

## 3. Fonctionnalités

Cinq modules, du branchement au partage. La colonne Phase renvoie à la feuille de route (section 6).

### Capture

| Fonction | Phase |
| --- | --- |
| Aperçu en direct de l'image et vumètres audio | MVP |
| Enregistrement sans perte vers un master d'archive | MVP |
| Arrêt automatique en fin de bande (neige ou noir prolongé, silence) ou minuterie | MVP |
| Estimation de la place nécessaire avant capture, alerte et arrêt propre si le disque se remplit | MVP |
| Capture DV : copie numérique exacte, date et heure d'origine de chaque plan | MVP (Windows) |
| Surveillance des images perdues ou dupliquées et de la qualité du signal pendant la capture | MVP |
| Réglages image avant capture (luminosité, contraste, saturation) | Phase 2 |

### Traitement

| Fonction | Phase |
| --- | --- |
| Désentrelacement bwdif à 50 images/s | MVP |
| Recadrage auto des bords parasites (bande de bruit en bas de l'image VHS) | MVP |
| Découpe en chapitres par détection de plans, de noirs, de silences et de neige | MVP |
| Chapitres DV calés sur les changements de date d'enregistrement | MVP |
| Coupe manuelle, fusion et réordonnancement des chapitres | MVP |
| Normalisation et débruitage audio | Phase 2 |
| Correction couleur automatique et stabilisation légère | Phase 2 |
| Désentrelacement nnedi (meilleure qualité, plus lent) | Phase 2 |
| Désentrelacement QTGMC (dépend de VapourSynth) | Phase 3 |

### Catalogue

| Fonction | Phase |
| --- | --- |
| Fiche par cassette : photo de l'étiquette, titre, date, lieu, personnes, notes | MVP |
| Chapitres avec vignettes, titre et date | MVP |
| Recherche et filtres (état, année, lieu, personne, support) | MVP |
| Bibliothèque sur disque externe : consultation quand le disque est débranché, disque retrouvé s'il change de lettre | MVP |
| Étiquette imprimable avec QR code pour retrouver la cassette physique | Phase 2 |
| Frise chronologique et carte des lieux | Phase 3 |

### Export et partage

| Fonction | Phase |
| --- | --- |
| MP4 H.264 (x264) désentrelacé, prêt pour TV, téléphone et ordinateur | MVP |
| Export par chapitre ou cassette entière | MVP |
| Préréglages : archive, partage familial, réseaux sociaux ; H.265 en option | Phase 2 |
| Suppression du master après export (mode avancé, avertissement clair, désactivée par défaut) | Phase 2 |
| Clé USB ou disque « familial » avec page d'accueil HTML lisible partout | Phase 2 |
| Galerie web statique | Phase 3 |
| DVD vidéo avec menu de chapitres | Phase 3 |

### IA locale

| Fonction | Outil retenu | Phase |
| --- | --- | --- |
| Découpe automatique (changements de plan, coupures de caméra) | Filtres FFmpeg `scdet`, `blackdetect`, `silencedetect`, `signalstats` ; dvrescue pour le DV | MVP |
| Transcription audio, sous-titres et recherche dans les paroles | whisper.cpp (crate whisper-rs) | Phase 2 |
| Amélioration d'image : mise à l'échelle et réduction du bruit de bande | Real-ESRGAN (ncnn) ou équivalent, licence à vérifier par modèle | Phase 3 |
| Reconnaissance de visages pour retrouver une personne | YuNet (détection) + SFace (reconnaissance), OpenCV Zoo, Apache 2.0, via ONNX Runtime | Phase 3 |

Les modèles d'IA sont téléchargés à la demande, jamais inclus dans l'installeur, et leur licence doit permettre la redistribution avec un logiciel GPL. InsightFace est écarté : ses modèles pré-entraînés sont réservés à un usage non commercial.

La reconnaissance de visages reste désactivée par défaut et demande un accord explicite : ce sont des données biométriques, y compris sur un usage privé. Ses données se suppriment en un geste.

## 4. Parcours utilisateur et interface

Le cœur de Numéris est un assistant en cinq étapes, lancé par le bouton « Numériser une cassette ».

1. **Préparer** : choix du support (VHS, Hi8, MiniDV…), photo ou saisie de l'étiquette, conseils (rembobiner, nettoyer les têtes).
2. **Brancher** : détection du boîtier ou du caméscope, aperçu en direct, test du son, indicateur vert quand tout est prêt.
3. **Capturer** : lecture sur le magnétoscope, Numéris enregistre et s'arrête seul en fin de bande ; durée, place disque et qualité du signal affichées.
4. **Organiser** : chapitres proposés automatiquement, l'utilisateur les renomme, les date et ajoute personnes et lieux.
5. **Partager** : choix d'un préréglage d'export, Numéris garde le master et produit les copies.

### Écrans principaux

- **Bibliothèque** : barre de titre intégrée (recherche, bouton « Numériser une cassette »), panneau latéral par catégories (état, supports, personnes, lieux), grille de vignettes au format 4:3, panneau de détails de la cassette sélectionnée à droite, barre d'état (disque, tâche en cours).
- **Fiche cassette** : chapitres en liste et frise, lecteur intégré, métadonnées.
- **Capture** : aperçu plein écran, vumètres, compteur, qualité du signal, journal des incidents (images perdues).
- **File de traitement** : tâches longues (encodage, analyse, IA) en arrière-plan, avec reprise après fermeture.
- **Réglages** : dossiers de stockage, formats, langue, mode avancé.

### Identité visuelle

Thème propre à Numéris, distinct d'Afterdusk : piste « Studio » en version appli de bureau (fond violet très foncé, accent corail, Manrope). Détails dans `docs/charte-graphique.md`. L'interface est dense et pensée pour la souris dans la bibliothèque ; l'assistant reste aéré, avec de gros boutons, pour le grand public.

## 5. Architecture technique

Même socle qu'Afterdusk, pour réutiliser l'organisation du code et les choix déjà validés (FFmpeg, signature, mises à jour). Le thème et les composants visuels, eux, sont propres à Numéris.

| Brique | Choix | Rôle |
| --- | --- | --- |
| Application | Tauri 2 | Fenêtre native légère, installeurs Windows (.msi) et macOS (.dmg), mises à jour (tauri-plugin-updater) |
| Interface | React 19 + TypeScript (Vite), zustand, react-i18next | Écrans, assistant, lecteur ; textes FR/EN |
| Cœur | Rust | Pilotage de FFmpeg et dvrescue, file de tâches, accès disque, détection matériel |
| Moteur vidéo | FFmpeg 9 en version GPL (x264, x265), programmes séparés livrés avec l'appli | Capture dshow / avfoundation, encodage, filtres, analyse, découpe |
| Analyse DV | dvrescue (MediaArea, BSD-3), programme séparé | Dates d'enregistrement, timecode, erreurs de bande, découpe par date |
| Catalogue | SQLite (rusqlite, `bundled`) | Cassettes, chapitres, personnes, lieux, tâches |
| IA | whisper-rs, ONNX Runtime (`ort`), Real-ESRGAN ncnn | Modèles téléchargés à la demande |

FFmpeg est récupéré comme dans Afterdusk (décision 0012 d'Afterdusk) : versions publiées par BtbN (Windows) et Martin Riedl (macOS), archives vérifiées par SHA-256 par un script, jamais versionnées, jamais téléchargées par l'application.

### Pourquoi des programmes séparés plutôt que des bibliothèques liées

- Un plantage de FFmpeg pendant une capture de 3 heures ne fait pas tomber l'application ; le master déjà écrit reste lisible.
- Licences simples : FFmpeg reste un programme distinct, cité dans `NOTICE.txt`.
- `-progress pipe:` donne en continu les images perdues (`drop_frames`) et dupliquées (`dup_frames`).

### Capture et aperçu en direct

Sous Windows, un seul programme peut ouvrir un boîtier de capture à la fois : l'interface ne peut donc pas afficher la caméra pendant que FFmpeg enregistre. Une seule commande FFmpeg produit à la fois :

1. le master FFV1 + PCM en MKV ;
2. un flux d'aperçu léger (MJPEG, environ 640 px, 25 images/s) lu par le cœur Rust et envoyé à l'interface ;
3. les mesures audio (`astats` ou `ebur128`) pour les vumètres et les mesures image (`signalstats`) pour la qualité du signal et la détection de fin de bande.

### Lecture dans l'application

Les moteurs web de Tauri (WebView2 sous Windows, WKWebView sous macOS) ne lisent ni FFV1, ni le DV, ni la vidéo entrelacée. Chaque cassette a donc un **proxy** H.264 léger, désentrelacé, avec des images clés rapprochées pour naviguer précisément lors de la découpe des chapitres. Le master n'est jamais lu directement par l'interface.

### Formats de stockage

- **Master analogique** : vidéo FFV1 version 3 (`-level 3 -g 1 -slicecrc 1`) + audio PCM dans un MKV, entrelacé d'origine, environ 15 à 25 Go par heure en PAL (à mesurer). Chaque image est autonome et porte une somme de contrôle : une corruption est détectée et reste limitée à une image. Un MKV interrompu (coupure de courant) reste lisible jusqu'à l'arrêt.
- **Master DV** : flux .dv conservé tel quel, sans réencodage (environ 13 Go par heure), avec le rapport dvrescue.
- **Empreintes** : empreinte BLAKE3 du master et fichier `framemd5` (une empreinte par image), pour vérifier l'archive dans dix ans.
- **Proxy** : MP4 H.264 léger pour le lecteur intégré.
- **Copies de diffusion** : MP4 H.264 (ou H.265 en option), désentrelacé à 50 images/s pour garder la fluidité.
- **Métadonnées** : en base SQLite et en fichier JSON à côté de chaque cassette, pour que l'archive reste lisible sans Numéris.

### Structure du dossier bibliothèque

```
Numeris/
  1998-noel-chez-mamie/
    master.mkv              (ou master.dv pour une cassette DV)
    master.framemd5
    master.dvrescue.xml     (cassettes DV seulement)
    cassette.json           (métadonnées, chapitres, empreinte BLAKE3 du master)
    etiquette.jpg
    proxy.mp4
    chapitres/
      01-ouverture-des-cadeaux.mp4
      02-repas.mp4
    vignettes/
```

Les chemins sont enregistrés en base relativement à la racine de la bibliothèque, pour qu'elle puisse vivre sur un disque externe.

## 6. Feuille de route

Quatre étapes, chacune livrable et utilisable seule. On passe à la suivante quand la précédente fonctionne sur de vraies cassettes.

1. **Socle** : projet Tauri + React, intégration continue Windows et macOS, FFmpeg livré, base SQLite et migrations, i18n FR/EN, thème Numéris, écrans Bibliothèque et Réglages, lecteur de fichier importé. Critère : l'appli s'installe sur Windows et macOS, liste les périphériques de capture et lit une vidéo.
2. **MVP** : assistant en 5 étapes, capture USB analogique, capture DV sous Windows, import de fichiers, master FFV1 avec empreintes, proxy, désentrelacement, recadrage, chapitres automatiques, fiche cassette, recherche, export MP4. Critère : une VHS de 3 h numérisée et découpée sans intervention technique, et une cassette MiniDV capturée avec les dates de chaque plan.
3. **Phase 2** : transcription Whisper et recherche dans les paroles, préréglages d'export, suppression des masters en mode avancé, clé USB familiale, débruitage audio, correction couleur, étiquettes QR. Critère : retrouver une scène en tapant un mot prononcé dedans.
4. **Phase 3** : amélioration d'image IA, reconnaissance de visages, frise et carte des lieux, galerie web, DVD, QTGMC. Critère : retrouver toutes les apparitions d'une personne sur l'ensemble des cassettes.

Publication sur GitHub dès le MVP, sous licence GPL-3.0-or-later.

## 7. Points de vigilance

Le risque principal est le matériel : chaque boîtier de capture se comporte différemment, il faut tester sur du vrai matériel tôt.

| Risque | Impact | Parade |
| --- | --- | --- |
| Dazzle DVC 90/100/101 sans pilote 64 bits | Capture analogique impossible à tester avec le matériel actuel | Vérifier dès le socle ; sinon boîtier UVC récent |
| FireWire absent des Mac récents et des PC modernes | Capture DV limitée | DV sous Windows avec port ou carte FireWire TI ; sur Mac, import de fichiers .dv |
| Capture DV sous Windows | FFmpeg dshow ne donne pas forcément le flux DV brut avec ses dates | Prototype tôt : dvrescue seul, ou FFmpeg pour la capture puis dvrescue pour l'analyse |
| Boîtiers USB bas de gamme (clones EasyCAP) instables | Images perdues, décalage son/image | Liste de matériel testé, surveillance des pertes, resynchronisation audio |
| Bande abîmée ou signal instable | Image qui saute, capture qui coupe | Conseil d'un magnétoscope avec TBC, capture tolérante aux pertes de signal |
| Licences : FFmpeg, x264, modèles IA | Problèmes de redistribution | FFmpeg GPL compatible avec la GPL v3 du projet, `NOTICE.txt`, licence vérifiée pour chaque modèle |
| Brevets H.264 / H.265 | Question juridique selon les pays | Même position que Shotcut ou HandBrake ; à garder en tête |
| Signature des applis (SmartScreen, Gatekeeper) | Avertissements à l'installation | Proposition : même choix qu'Afterdusk (SignPath Foundation sous Windows, macOS non signé au début), à confirmer |
| Espace disque (masters de 15 à 25 Go/h, nombre de cassettes libre) | Disque saturé en cours de capture | Estimation avant capture, alerte et arrêt propre |
| Performances IA sur machines modestes | Traitements très longs | File d'attente, reprise, option GPU, résultats partiels utilisables |
| Reconnaissance de visages | Données biométriques sensibles | Désactivée par défaut, tout reste local, suppression facile |
| Droits d'auteur | Cassettes du commerce | Numéris est destiné aux enregistrements personnels ; pas de contournement de Macrovision |

## 8. Décisions prises

Le 23 octobre 2026 :

1. **Licence GPL-3.0-or-later**, comme Afterdusk. Elle permet de livrer FFmpeg en version GPL avec x264 pour des exports H.264 de qualité.
2. **Même socle technique qu'Afterdusk**, mais **thème propre** (piste « Studio » en appli de bureau).
3. **Aperçu en direct produit par FFmpeg** (second flux pendant la capture), et **proxy H.264** pour le lecteur intégré.
4. **Filtres FFmpeg** pour la détection des plans, noirs, silences et neige, au lieu de PySceneDetect : pas de Python dans l'installeur.
5. **dvrescue** pour l'analyse DV (dates, timecode, erreurs, découpe).
6. **YuNet + SFace** pour les visages au lieu d'InsightFace (licence).
7. **Masters conservés par défaut** ; suppression possible après export en mode avancé, avec avertissement.
8. **Nombre de cassettes libre** : estimation de place et bibliothèque sur disque externe prises en charge dès le MVP.
9. **QTGMC repoussé en phase 3** ; bwdif au MVP, nnedi en phase 2.

## 9. Points ouverts

- Modèle exact du Dazzle et reconnaissance par Windows 11.
- Méthode de capture DV sous Windows (dvrescue ou FFmpeg), à trancher par un prototype.
- Signature des installateurs : reprendre la décision d'Afterdusk ou non.
- Support FireWire dans macOS 26 : à confirmer.
- Taille réelle d'une heure de master FFV1 en PAL : à mesurer.
