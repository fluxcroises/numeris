# Matériel testé

Chaque essai sur du vrai matériel est noté ici : il alimente la page d'aide « Quel matériel choisir ? » de l'application.

| Date | Matériel | Liaison | Système | Résultat | Remarques |
| --- | --- | --- | --- | --- | --- |
| 2026-10-23 | Pinnacle Dazzle DVC100 (USB 2304:021A) | USB, composite / S-Vidéo | Windows 11 64 bits | Détecté | Pilote installé et fonctionnel. FFmpeg (dshow) voit deux périphériques : « Dazzle DVC100 Video Device » (vidéo) et « Ligne (Dazzle DVC100 Audio Device) » (audio). Capture à tester au MVP |
| 2026-10-23 | Pinnacle Dazzle DVC100 | USB, S-Vidéo (broche 3), sans cassette | Windows 11 64 bits | Capture OK | Moteur de capture de Numéris : 720×576 à 25 i/s en 4:2:2, son stéréo 48 kHz, 0 image perdue ou dupliquée, master FFV1 lisible. Démarrage du boîtier : environ 1 s. Sans signal, luminosité moyenne 9 en composite et 24 en S-Vidéo (seuil du noir ajusté). Une capture arrêtée brutalement reste lisible jusqu'à la coupure. Reste à faire : une vraie VHS (qualité d'image, ordre des trames, fin de bande) |
| à venir | Caméscope DV (modèle à noter) | FireWire, contrôleur Texas Instruments OHCI | Windows 11 | à tester | |
