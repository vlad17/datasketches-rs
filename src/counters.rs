//! Stateful reducers which maintain distinct count and heavy
//! hitters sketches, aimed at servicing the `dsrs` command-line tool
//! for deduplicating byte lines of input.

use std::collections::HashMap;
use std::convert::TryInto;
use std::str;

use base64;
use memchr;

use crate::stream_reducer::LineReducer;
use crate::{CpcSketch, CpcUnion, DataSketchesError, HhSketch};

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
    pub fn deserialize(s: &str) -> Result<Self, DataSketchesError> {
        let bytes = base64::decode_config(s, base64::STANDARD_NO_PAD)?;
        let sketch = CpcSketch::deserialize(bytes.as_ref())?;
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

pub struct Merger {
    sketch: CpcUnion,
}

impl Default for Merger {
    fn default() -> Self {
        Self {
            sketch: CpcUnion::new(),
        }
    }
}

impl Merger {
    pub fn counter(&self) -> Counter {
        let sketch = self.sketch.sketch();
        Counter { sketch }
    }
}

impl LineReducer for Merger {
    fn read_line(&mut self, line: &[u8]) {
        let line = str::from_utf8(line).unwrap_or_else(|e| {
            panic!(
                "invalid UTF-8: {}\n{}\n{:?}",
                e,
                String::from_utf8_lossy(line),
                line
            )
        });
        let counter = Counter::deserialize(line).expect("properly deserialized counter");
        self.sketch.merge(counter.sketch);
    }
}

#[derive(Default)]
pub struct KeyedMerger {
    sketches: HashMap<Vec<u8>, Merger>,
}

impl LineReducer for KeyedMerger {
    fn read_line(&mut self, line: &[u8]) {
        let space_ix = memchr::memchr(b' ', line).unwrap_or_else(|| {
            panic!(
                "line missing space: '{}'",
                str::from_utf8(line).unwrap_or("BAD UTF-8")
            )
        });
        let (key, value) = (&line[0..space_ix], &line[space_ix + 1..]);
        if !self.sketches.contains_key(key) {
            self.sketches.insert(key.to_owned(), Merger::default());
        }
        self.sketches
            .get_mut(key)
            .expect("key present")
            .read_line(value);
    }
}

impl KeyedMerger {
    /// Returns an iterator over all contained keys and their sketches.
    pub fn state(&self) -> impl Iterator<Item = (&[u8], Counter)> {
        self.sketches
            .iter()
            .map(|(key, mrgr)| (key.as_ref(), mrgr.counter()))
    }
}

pub struct HeavyHitter {
    sketch: HhSketch,
    k: u64
}

// https://users.rust-lang.org/t/logarithm-of-integers/8506/5

fn log2_floor(x: u64) -> usize {
    const fn num_bits<T>() -> usize { std::mem::size_of::<T>() * 8 }
    assert!(x > 0);
    num_bits::<u64>() - x.leading_zeros() as usize - 1
}

impl HeavyHitter {

    /// Creates a new heavy hitter sketch targeting elements in the top-k
    /// by reserving O(k) space.
    pub fn new( k: u64) -> Self {
        let lg2_k_with_room = log2_floor(k as u64).max(1) + 2;
        Self {
            sketch: HhSketch::new(lg2_k_with_room.try_into().unwrap()),
            k
        }
    }
    
    /// Serializes to base64 string with no newlines or `=` padding.
    pub fn serialize(&self) -> String {
        unimplemented!()
    }

    /// Deserializes from base64 string with no newlines or `=` padding.
    pub fn deserialize(_s: &str) -> Result<Self, base64::DecodeError> {
        unimplemented!()
    }

    /// Returns pairs (heavy hitter slice, estimate of count size)
    pub fn estimate(&self) -> impl Iterator<Item = (&[u8], u64)> {
        let mut v = self.sketch.estimate_no_fn();
        v.sort_by_key(|row| row.ub);
        v
            .into_iter()
            .rev()
            .take(self.k as usize)
            .map(|row| (row.key, row.ub))
    }
}

impl LineReducer for HeavyHitter {
    fn read_line(&mut self, line: &[u8]) {
        self.sketch.update(line, 1);
    }
}
