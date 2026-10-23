# Charte graphique

Piste « Studio » en appli de bureau, retenue le 23 octobre 2026. Maquette de référence : planche « C2 · Studio en appli de bureau » du canevas https://claude.ai/artifact/W5ErhwWtYVzqfVy4DVDJ6g (privé).

Numéris a sa propre identité, distincte d'Afterdusk (gris neutre, accent bleu).

## Principes

- **Logiciel de bureau, pas site web** : barre de titre intégrée à la fenêtre, panneau latéral par catégories, panneau de détails à droite, barre d'état en bas. Pas de grand titre de page ni de gros boutons dans la bibliothèque.
- **Dense dans la bibliothèque, aéré dans l'assistant** : texte de 13 px et lignes d'environ 30 px dans les vues de travail ; l'assistant « Numériser une cassette » garde de gros boutons et des textes plus grands pour le grand public.
- **L'image d'abord** : vignettes au format 4:3 des cassettes, coins peu arrondis, séparateurs fins plutôt que cartes et ombres.
- **Les durées et les timecodes** s'affichent en police monospace.

## Couleurs (thème sombre)

| Rôle | Variable | Valeur |
| --- | --- | --- |
| Fond de la fenêtre | `--nm-bg` | `#15121D` |
| Barres (titre, latérale, état) | `--nm-chrome` | `#100E17` |
| Panneau de détails | `--nm-panel` | `#1A1724` |
| Champs, surfaces | `--nm-surface` | `#1F1B2A` |
| Élément sélectionné | `--nm-selected` | `#251F33` |
| Survol, contrôle actif | `--nm-hover` | `#332D47` |
| Bordures | `--nm-border` | `#2A2539` |
| Bordures de champs | `--nm-field-border` | `#2E2940` |
| Texte | `--nm-text` | `#EFEAF7` |
| Texte secondaire | `--nm-text-secondary` | `#ABA3BF` |
| Titres de sections | `--nm-text-muted` | `#8F87A6` |
| Accent (action principale, sélection) | `--nm-accent` | `#FF8266` |
| Texte sur l'accent | `--nm-on-accent` | `#1A1016` |
| État « Prête », réussite, espace disque | `--nm-ready` | `#7FDCC4` |
| État « À traiter » | `--nm-todo` | `#FF9D86` |
| État « Exportée » | `--nm-exported` | `#C2BADF` |

Les contrastes texte/fond respectent 4,5:1. Les états se distinguent aussi par leur libellé, jamais par la seule couleur. Un thème clair pourra venir plus tard ; il redéfinira les mêmes variables.

## Typographie

- **Manrope** (OFL) pour toute l'interface : 13 px pour le texte courant, 11 px en capitales espacées pour les titres de sections, 16 px en gras pour le titre du panneau de détails.
- **JetBrains Mono** (OFL) pour les durées, timecodes et raccourcis clavier.
- Les polices sont livrées avec l'application (aucun appel réseau).

## Formes

- Rayons : 5-6 px pour les contrôles et les vignettes, 3 px pour les étiquettes.
- Icônes au trait (Lucide), 14-16 px dans les vues denses.
- Boutons de fenêtre dessinés par l'application (barre de titre personnalisée de Tauri) : réduire, agrandir, fermer à droite sous Windows ; boutons natifs à gauche sous macOS.
