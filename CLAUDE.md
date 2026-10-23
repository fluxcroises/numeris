# Numéris

Logiciel open source (GPL v3) pour numériser des cassettes vidéo (VHS, VHS-C, S-VHS, Video8, Hi8, Digital8, MiniDV) et organiser les films de famille.
Tauri 2 + Rust (src-tauri/) et React + TypeScript (src/).
Cahier des charges : docs/cahier-des-charges.md. Charte graphique : docs/charte-graphique.md. Avancement : docs/avancement.md. Décisions : docs/decisions/.

## Règles absolues
- Ne jamais modifier, déplacer, réencoder ou supprimer un master sans action explicite de l'utilisateur.
- Un master n'est considéré comme écrit qu'une fois son empreinte BLAKE3 et son framemd5 enregistrés.
- Toute suppression passe par la corbeille interne ; la suppression d'un master demande une confirmation explicite.
- Une capture interrompue (plantage, coupure, disque plein) laisse un master lisible jusqu'à l'arrêt et ne corrompt jamais la base.
- Aucune vidéo, image, transcription ni donnée personnelle ne quitte l'ordinateur. Aucun appel réseau hors des exceptions listées ici, chacune désactivable dans les Réglages. Exceptions : vérification des mises à jour, téléchargement d'un modèle d'IA à la demande de l'utilisateur. Toute nouvelle exception est ajoutée ici après accord de l'utilisateur.
- La reconnaissance de visages reste désactivée tant que l'utilisateur ne l'a pas activée explicitement.

## Conventions
- Rust : cargo fmt et cargo clippy sans avertissement ; erreurs avec thiserror, jamais de unwrap hors tests.
- TypeScript strict, ESLint et Prettier ; composants fonctionnels React.
- Aucun texte d'interface en dur : tout passe par i18next (fr.json et en.json à jour ensemble).
- Aucune couleur en dur dans les composants : uniquement les variables du thème (src/theme.css, docs/charte-graphique.md).
- FFmpeg et dvrescue sont des programmes séparés, appelés par le cœur Rust ; jamais de liaison directe à leurs bibliothèques.
- Traitements longs (capture, encodage, analyse, IA) dans des tâches Rust en arrière-plan, progression envoyée par événements Tauri, reprise après fermeture.
- Chemins de médias stockés relativement à la racine de la bibliothèque.
- Chaque cassette a son dossier avec un cassette.json lisible sans l'application ; toute évolution de son format est versionnée et prévoit une migration.
- Migrations SQLite numérotées dans src-tauri/src/db/migrations/.
- Dépendances et modèles d'IA compatibles GPL v3 uniquement ; vérifier la licence avant tout ajout.
- Code commenté en français, noms de variables en anglais.

## Tests
- cargo test pour le cœur, Vitest pour l'interface.
- Échantillons vidéo générés par FFmpeg lui-même (mires, bruit, noir, silence), versionnés avec les tests ; les captures réelles restent hors du dépôt (test-assets/captures/).
- Un test par règle absolue (master jamais modifié, capture interrompue lisible, aucun appel réseau non prévu).
- Tester sur Windows et macOS via GitHub Actions avant de cocher un critère.
- La capture se teste aussi sur du vrai matériel : noter dans docs/materiel-teste.md le boîtier, le système et le résultat.

## Façon de travailler
- Une étape à la fois, dans l'ordre de docs/cahier-des-charges.md section 6.
- Plan court avant chaque fonctionnalité, commit par fonctionnalité, messages en français.
- Demander avant une décision difficile à défaire, et la consigner dans docs/decisions/.
