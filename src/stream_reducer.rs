//! A small abstraction for reducing over byte lines from a stream,
//! used for the command line tool `dsrs`.
//!
//! There may be opportunities for improvement here, e.g., see the
//! fancy regex searcher for ripgrep [1].
//!
//! [1]: https://docs.rs/grep-searcher/0.1.8/grep_searcher/index.html

use std::io::{BufRead, Error};

use bstr::io::BufReadExt;

pub trait LineReducer {
    fn read_line(&mut self, line: &[u8]);
}

pub fn reduce_stream<R: BufRead, T: LineReducer>(
    stream: R,
    mut line_reader: T,
) -> Result<T, Error> {
    // TODO: consider 2-threaded approach, building up a
    // contiguous vec buffer with offsets, by creating a "LineBuffer" struct out here
    // which mutably fills up inside the below, then get sent over on completion.
    stream.for_byte_line(|line| {
        line_reader.read_line(line);
        Ok(true)
    })?;
    Ok(line_reader)
}

#[cfg(test)]
mod tests {

    use std::u8;

    use proptest::{collection, prop_assert_eq, proptest, sample};

    use super::*;

    #[derive(Default)]
    struct DumbReducer {
        all: Vec<u8>,
    }

    impl LineReducer for DumbReducer {
        fn read_line(&mut self, line: &[u8]) {
            self.all.extend_from_slice(line);
            self.all.push(b'\n');
        }
    }

    fn non_newlines() -> Vec<u8> {
        (0..u8::MAX).filter(|x| *x != b'\n').collect()
    }

    proptest! {
        #[test]
        fn reduces_stream(
            mut s in collection::vec(collection::vec(sample::select(non_newlines()), 0..81), 0..10)) {
            for line in s.iter_mut() {
                while line.last().filter(|c| **c == b'\r').is_some() {
                    line.pop();
                }
            }
            let mut file = s.join(&b'\n');
            file.push(b'\n');

            let reducer = DumbReducer::default();
            let reducer = reduce_stream(&file[..], reducer).unwrap();

            prop_assert_eq!(reducer.all, file);
        }
    }
}
