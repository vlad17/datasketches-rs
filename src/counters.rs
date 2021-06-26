//! Stateful reducers which maintain distinct count sketches,
//! aimed at servicing the `dsrs` command-line tool for deduplicating
//! byte lines of input.

use std::collections::HashMap;
use std::str;

use base64;
use memchr;

use crate::stream_reducer::LineReducer;
use crate::CpcSketch;

pub struct Counter {
    sketch: CpcSketch,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            sketch: CpcSketch::new(),
        }
    }
}

impl Counter {
    /// Serializes to base64 string with no newlines or `=` padding.
    pub fn serialize(&self) -> String {
        let bytes = self.sketch.serialize();
        base64::encode_config(bytes, base64::STANDARD_NO_PAD)
    }

    /// Deserializes from base64 string with no newlines or `=` padding.
    pub fn deserialize(s: &str) -> Result<Self, base64::DecodeError> {
        let bytes = base64::decode_config(s, base64::STANDARD_NO_PAD)?;
        let sketch = CpcSketch::deserialize(bytes.as_ref());
        Ok(Self { sketch })
    }

    /// Returns the current row estimate
    pub fn estimate(&self) -> f64 {
        self.sketch.estimate()
    }
}

impl LineReducer for Counter {
    fn read_line(&mut self, line: &[u8]) {
        self.sketch.update(line);
    }
}

#[derive(Default)]
pub struct KeyedCounter {
    sketches: HashMap<Vec<u8>, Counter>,
}

impl LineReducer for KeyedCounter {
    fn read_line(&mut self, line: &[u8]) {
        let space_ix = memchr::memchr(b' ', line).unwrap_or_else(|| {
            panic!(
                "line missing space: '{}'",
                str::from_utf8(line).unwrap_or("BAD UTF-8")
            )
        });
        let (key, value) = (&line[0..space_ix], &line[space_ix + 1..]);
        if !self.sketches.contains_key(key) {
            self.sketches.insert(key.to_owned(), Counter::default());
        }
        self.sketches
            .get_mut(key)
            .expect("key present")
            .read_line(value);
    }
}

impl KeyedCounter {
    /// Returns an iterator over all contained keys and their sketches.
    pub fn state(&self) -> impl Iterator<Item = (&[u8], &Counter)> {
        self.sketches.iter().map(|(key, ctr)| (key.as_ref(), ctr))
    }
}

// TODO: KeyedMerger -- reads the
