//! Découpe du flux d'aperçu MJPEG (images JPEG mises bout à bout par FFmpeg)
//! en images séparées. Dans les données compressées d'un JPEG, un octet 0xFF
//! est toujours suivi de 0x00 : la marque de fin 0xFF 0xD9 ne peut donc pas
//! apparaître ailleurs qu'à la fin de l'image.

const SOI: [u8; 2] = [0xFF, 0xD8];
const EOI: [u8; 2] = [0xFF, 0xD9];

#[derive(Default)]
pub struct FrameSplitter {
    buffer: Vec<u8>,
}

impl FrameSplitter {
    /// Ajoute des octets lus et renvoie les images complètes qu'ils terminent.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Vec<u8>> {
        self.buffer.extend_from_slice(bytes);
        let mut frames = Vec::new();
        loop {
            let Some(start) = find(&self.buffer, &SOI, 0) else {
                // Rien d'utile avant un début d'image : on ne garde que le
                // dernier octet, qui peut être la première moitié de la marque.
                let keep = self.buffer.len().min(1);
                self.buffer.drain(..self.buffer.len() - keep);
                break;
            };
            let Some(end) = find(&self.buffer, &EOI, start + 2) else {
                self.buffer.drain(..start);
                break;
            };
            frames.push(self.buffer[start..end + 2].to_vec());
            self.buffer.drain(..end + 2);
        }
        frames
    }
}

fn find(haystack: &[u8], needle: &[u8; 2], from: usize) -> Option<usize> {
    haystack
        .get(from..)?
        .windows(2)
        .position(|w| w == needle)
        .map(|i| i + from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg(body: &[u8]) -> Vec<u8> {
        [&SOI[..], body, &EOI[..]].concat()
    }

    #[test]
    fn splits_frames_across_reads() {
        let a = jpeg(&[1, 2, 0xFF, 0x00, 3]);
        let b = jpeg(&[4, 5]);
        let stream = [a.clone(), b.clone()].concat();
        let mut splitter = FrameSplitter::default();
        let mut frames = Vec::new();
        // Lectures de 3 octets : les marques sont coupées en deux.
        for chunk in stream.chunks(3) {
            frames.extend(splitter.push(chunk));
        }
        assert_eq!(frames, vec![a, b]);
    }

    #[test]
    fn ignores_garbage_before_a_frame() {
        let mut splitter = FrameSplitter::default();
        let frame = jpeg(&[9]);
        let frames = splitter.push(&[[7, 7, 7].as_slice(), &frame].concat());
        assert_eq!(frames, vec![frame]);
    }
}
