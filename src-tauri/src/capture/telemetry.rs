//! Mesures de capture lues sur la sortie d'erreur de FFmpeg : avancement
//! (`-progress`), statistiques image (`signalstats`, `idet`) et niveaux audio
//! (`astats`), imprimées par les filtres `metadata` et `ametadata`.

use serde::Serialize;

/// État du signal vidéo, pour l'indicateur de qualité et la fin de bande.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SignalState {
    /// Pas encore de mesure.
    Unknown,
    Ok,
    /// Image noire : pas de signal, ou bande vierge enregistrée au noir.
    Black,
    /// Neige : bande vierge ou fin d'enregistrement.
    Snow,
}

// Seuils à affiner sur de vraies cassettes (docs/materiel-teste.md). Valeurs
// sur 8 bits ; le noir vidéo vaut 16. Le Dazzle DVC100 sans signal donne une
// moyenne d'environ 9 en composite et 24 en S-Vidéo. Une scène sombre garde
// des points clairs : le maximum distingue le noir d'une nuit filmée.
const BLACK_MAX_YAVG: f64 = 32.0;
const BLACK_MAX_YMAX: f64 = 60.0;
// La neige change d'une image à l'autre (YDIF élevé) et n'a pas de couleur.
const SNOW_MIN_YDIF: f64 = 12.0;
const SNOW_MAX_SATAVG: f64 = 6.0;

pub fn classify(yavg: f64, ymax: f64, ydif: f64, satavg: f64) -> SignalState {
    if yavg < BLACK_MAX_YAVG && ymax < BLACK_MAX_YMAX {
        SignalState::Black
    } else if ydif > SNOW_MIN_YDIF && satavg < SNOW_MAX_SATAVG {
        SignalState::Snow
    } else {
        SignalState::Ok
    }
}

/// Ordre des trames estimé par `idet` (comptes cumulés).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldCounts {
    pub tff: u64,
    pub bff: u64,
    pub progressive: u64,
}

impl FieldCounts {
    /// « tff », « bff » ou « progressive » ; `None` sans mesure suffisante.
    pub fn guess(&self) -> Option<&'static str> {
        let best = [
            (self.tff, "tff"),
            (self.bff, "bff"),
            (self.progressive, "progressive"),
        ]
        .into_iter()
        .max_by_key(|(n, _)| *n)?;
        (best.0 >= 25).then_some(best.1)
    }
}

/// Instantané envoyé à l'interface à chaque bloc d'avancement (deux par seconde).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Telemetry {
    pub elapsed_ms: u64,
    pub frames: u64,
    pub dropped: u64,
    pub duplicated: u64,
    /// Taille écrite du master.
    pub bytes: u64,
    /// Crêtes audio par canal, en dBFS (−∞ remplacé par −120).
    pub peaks_db: Vec<f64>,
    pub signal: SignalState,
    pub yavg: Option<f64>,
    pub fields: FieldCounts,
    pub free_bytes: Option<u64>,
}

/// Accumule les lignes de FFmpeg et rend un instantané à chaque fin de bloc
/// d'avancement.
#[derive(Default)]
pub struct Parser {
    elapsed_ms: u64,
    frames: u64,
    dropped: u64,
    duplicated: u64,
    bytes: u64,
    peaks_db: Vec<f64>,
    yavg: Option<f64>,
    ymax: Option<f64>,
    ydif: Option<f64>,
    satavg: Option<f64>,
    /// Comptes cumulés d'idet, tels que FFmpeg les donne.
    raw_fields: FieldCounts,
    /// Comptes au dernier instantané, pour n'en garder que l'accroissement.
    seen_fields: FieldCounts,
    /// Comptes retenus : seulement pendant une image correcte (sur du noir ou
    /// de la neige, idet répond au hasard).
    fields: FieldCounts,
    /// Dernières lignes d'erreur de FFmpeg, pour expliquer un échec.
    pub errors: Vec<String>,
}

fn number(value: &str) -> Option<f64> {
    let v = value.trim();
    if v.contains("inf") {
        return Some(if v.starts_with('-') { -120.0 } else { 0.0 });
    }
    v.parse().ok()
}

impl Parser {
    pub fn line(&mut self, line: &str) -> Option<Telemetry> {
        if let Some(i) = line.find("lavfi.") {
            let (key, value) = line[i + "lavfi.".len()..].split_once('=')?;
            self.metadata(key, value);
            return None;
        }
        if line.contains("[error]") || line.contains("[fatal]") {
            if self.errors.len() >= 8 {
                self.errors.remove(0);
            }
            self.errors.push(line.trim().to_owned());
            return None;
        }
        let (key, value) = line.trim().split_once('=')?;
        let int = || value.trim().parse::<u64>().ok();
        match key {
            "frame" => self.frames = int()?,
            "drop_frames" => self.dropped = int()?,
            "dup_frames" => self.duplicated = int()?,
            "total_size" => self.bytes = int()?,
            "out_time_us" => self.elapsed_ms = int()? / 1000,
            "progress" => return Some(self.snapshot()),
            _ => {}
        }
        None
    }

    fn metadata(&mut self, key: &str, value: &str) {
        let Some(v) = number(value) else { return };
        match key {
            "signalstats.YAVG" => self.yavg = Some(v),
            "signalstats.YMAX" => self.ymax = Some(v),
            "signalstats.YDIF" => self.ydif = Some(v),
            "signalstats.SATAVG" => self.satavg = Some(v),
            "idet.multiple.tff" => self.raw_fields.tff = v as u64,
            "idet.multiple.bff" => self.raw_fields.bff = v as u64,
            "idet.multiple.progressive" => self.raw_fields.progressive = v as u64,
            _ => {
                // « astats.1.Peak_level », « astats.2.Peak_level »…
                if let Some(channel) = key
                    .strip_prefix("astats.")
                    .and_then(|k| k.strip_suffix(".Peak_level"))
                    .and_then(|c| c.parse::<usize>().ok())
                    .filter(|c| (1..=8).contains(c))
                {
                    if self.peaks_db.len() < channel {
                        self.peaks_db.resize(channel, -120.0);
                    }
                    self.peaks_db[channel - 1] = v.max(-120.0);
                }
            }
        }
    }

    fn snapshot(&mut self) -> Telemetry {
        let signal = match (self.yavg, self.ymax, self.ydif, self.satavg) {
            (Some(y), Some(m), Some(d), Some(s)) => classify(y, m, d, s),
            _ => SignalState::Unknown,
        };
        if signal == SignalState::Ok {
            let (raw, seen) = (self.raw_fields, self.seen_fields);
            self.fields.tff += raw.tff.saturating_sub(seen.tff);
            self.fields.bff += raw.bff.saturating_sub(seen.bff);
            self.fields.progressive += raw.progressive.saturating_sub(seen.progressive);
        }
        self.seen_fields = self.raw_fields;
        Telemetry {
            elapsed_ms: self.elapsed_ms,
            frames: self.frames,
            dropped: self.dropped,
            duplicated: self.duplicated,
            bytes: self.bytes,
            peaks_db: self.peaks_db.clone(),
            signal,
            yavg: self.yavg,
            fields: self.fields,
            free_bytes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_progress_blocks_and_filter_metadata() {
        let mut p = Parser::default();
        let lines = [
            "[Parsed_metadata_4 @ 000001dd01b8c740] [info] frame:12 pts:12 pts_time:2.4",
            "[Parsed_metadata_4 @ 000001dd01b8c740] [info] lavfi.signalstats.YAVG=9.01811",
            "[Parsed_metadata_4 @ 000001dd01b8c740] [info] lavfi.signalstats.YMAX=21",
            "[Parsed_metadata_4 @ 000001dd01b8c740] [info] lavfi.signalstats.YDIF=0.5",
            "[Parsed_metadata_4 @ 000001dd01b8c740] [info] lavfi.signalstats.SATAVG=0.2",
            "[Parsed_metadata_2 @ 0] [info] lavfi.idet.multiple.tff=40",
            "[Parsed_ametadata_8 @ 0] [info] lavfi.astats.1.Peak_level=-12.5",
            "[Parsed_ametadata_8 @ 0] [info] lavfi.astats.2.Peak_level=-inf",
            "frame=118",
            "drop_frames=2",
            "dup_frames=1",
            "total_size=2097152",
            "out_time_us=4720000",
        ];
        for line in lines {
            assert_eq!(p.line(line), None);
        }
        let t = p.line("progress=continue").unwrap();
        assert_eq!(t.frames, 118);
        assert_eq!((t.dropped, t.duplicated), (2, 1));
        assert_eq!(t.bytes, 2_097_152);
        assert_eq!(t.elapsed_ms, 4720);
        assert_eq!(t.peaks_db, vec![-12.5, -120.0]);
        assert_eq!(t.signal, SignalState::Black);
        // Sur du noir, les comptes d'idet ne sont pas retenus.
        assert_eq!(t.fields.tff, 0);

        // L'image arrive : seul l'accroissement des comptes est retenu.
        for line in [
            "[p @ 0] [info] lavfi.signalstats.YAVG=110",
            "[p @ 0] [info] lavfi.signalstats.YMAX=235",
            "[p @ 0] [info] lavfi.signalstats.SATAVG=30",
            "[p @ 0] [info] lavfi.idet.multiple.tff=70",
        ] {
            p.line(line);
        }
        let t = p.line("progress=continue").unwrap();
        assert_eq!(t.signal, SignalState::Ok);
        assert_eq!(t.fields.tff, 30);
        assert_eq!(t.fields.guess(), Some("tff"));
    }

    #[test]
    fn keeps_last_errors() {
        let mut p = Parser::default();
        p.line("[in#0 @ 0] [error] Could not run graph");
        assert_eq!(p.errors, vec!["[in#0 @ 0] [error] Could not run graph"]);
    }

    #[test]
    fn classifies_signal() {
        // Dazzle DVC100 sans signal : composite, puis S-Vidéo.
        assert_eq!(classify(9.0, 12.0, 0.3, 0.1), SignalState::Black);
        assert_eq!(classify(24.0, 26.0, 0.2, 0.1), SignalState::Black);
        // Une nuit filmée : sombre en moyenne, mais avec des lumières.
        assert_eq!(classify(28.0, 235.0, 2.0, 8.0), SignalState::Ok);
        assert_eq!(classify(120.0, 235.0, 40.0, 1.0), SignalState::Snow);
        assert_eq!(classify(110.0, 235.0, 3.0, 30.0), SignalState::Ok);
        // Une scène qui bouge beaucoup mais colorée n'est pas de la neige.
        assert_eq!(classify(60.0, 200.0, 20.0, 25.0), SignalState::Ok);
    }
}
