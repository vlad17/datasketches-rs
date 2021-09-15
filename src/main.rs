//! `dsrs` main executable, which provides count-distinct functionality
//! on the command line.

use std::io;
use std::iter;
use std::str;

use dsrs::counters::{Counter, HeavyHitter, KeyedCounter, KeyedMerger, Merger};
use dsrs::stream_reducer::reduce_stream;
use structopt::StructOpt;

/// `dsrs` provides both count-distinct and heavy hitter functionality
/// to the command line.
///
/// `dsrs --hh k` returns the approximate top-k most popular lines from
/// stdin. It can be viewed as essentially a separate command.
///
/// `dsrs [--key] [--raw] [--merge]` returns the count of unique lines
/// from stdin.
///
/// It has three important options (key, raw, merge), which all interact
/// and have different I/O expectations.
///
/// No matter what, all input is assumed UTF-8 and the count estimates are rounded.
///
/// Note newline terminators \r\n and \n are stripped and ignored as far
/// as unique line content is concerned. So a file with DOS vs UNIX
/// line endings should give rise to the same counts and sketches.
///
/// 0. None of {key, raw, merge} set - count distinct lines
/// 1. key only - count distinct lines, keyed by first word
/// 2. raw only - as above, but print a raw serialized sketch value, not
///    the count.
/// 3. key + raw - count distinct lines, keyed by first word, but print
///    the raw serialized sketch values rather than actual count
/// 4. merge only - merge lines of serialized sketch values and print the
///    resulting count
/// 5. key + merge - merge lines of serialized sketch values for each key
/// 6. raw + merge - merge lines of serialized sketch values and print
///    resulting single serialized sketch.
/// 7. key + raw + merge - like 5 but print the raw serialized sketch
///
/// You very likely will never need (6) or (7), which are only ever useful
/// if performing multi-level parallel aggregations (a "combiner" in
/// map reduce literature).
///
/// There then two main use cases, each of which can be keyed or not.
///
/// # Simple Single-threaded Approximate Count
///
/// This is an approximate version of `sort -u | wc -l` and relies on
/// just (0). Just count distinct lines.
///
/// ```bash
/// seq 100 | dsrs
/// # 100
/// (seq 100 && seq 100 && seq 100) | dsrs
/// # 100
/// seq 100 | xargs -L1 seq | dsrs # [1], [1, 2], [1, 2, 3], etc.
/// # 100
/// ```
///
/// # Parallel 2-level Approximate Count
///
/// Here, we parallelize with one tree level, relying on
/// multiple calls of (2) in parallel and then a final (4) for aggregation.
///
/// TODO ex
///
/// # Keyed Versions of the Above
///
/// It is often convenient to run the above computation for each
/// unique key (first word on each line). Using (1) for the first use case
/// and (3, 5) for the second would result in the same output as if we
/// took each key, re-ran the entire computation for just lines with that
/// key, and emitted all lines prefixed by key.
///
/// ```bash
/// cat <<"EOF" | dsrs --key
/// a 1
/// a 2
/// b 1
/// b 2
/// a 1
/// a 3
/// EOF
/// # b 2
/// # a 3
/// ```
#[derive(Debug, StructOpt)]
#[structopt(name = "dsrs", about = "Approximate count distinct lines.")]
struct Opt {
    /// If set, then rather than computing the count of distinct lines
    /// overall, `dsrs` will compute the count of distinct lines for each
    /// key, where a key is defined to be the first word on a line,
    /// delimited by a space.
    ///
    /// This corresponds to an approximate version of a SQL statement like
    /// `SELECT KEY, COUNT(DISTINCT *) FROM stdin-lines GROUP BY 1`
    /// where stdin-lines would be a table over all input lines where
    /// the `KEY` column is the first word and the rest of the record
    /// is the rest of the line after the first delimiter, which MUST
    /// be present in every line.
    ///
    /// If `--merge` is set, then the value of each key should be a
    /// serialized sketch value resulting from a `dsrs --raw` invocation.
    ///
    /// This means that the output of `dsrs` is a single line (containing
    /// either the approximate count or sketch, depending on --raw setting)
    /// if --key is unset or a line for each key, prefixed by each key,
    /// if --key is set.
    #[structopt(long)]
    key: bool,

    /// If set, the raw flag results in a base64 serialized printout of
    /// the sketch at the end of computation rather than the approximate
    /// distinct count. This is useful when combined with a downstream
    /// `dsrs --merge` operation later to merge multiple sketches.
    #[structopt(long)]
    raw: bool,

    /// If set, expects inputs to contain a base64 serialized printout of
    /// sketches generated by upstream `dsrs --raw` commands. Then `dsrs`
    /// will merge the deserialized sketches to compute distinct counts
    /// across all constituent values.
    #[structopt(long)]
    merge: bool,

    /// Can only be set if all other flags are disabled. Returns a
    /// upper bound estimate for the number of times a line is expected
    /// to have appeared, along with the line itself.
    #[structopt(long)]
    hh: Option<u64>,
}

fn main() {
    let opt = Opt::from_args();

    if let Some(k) = opt.hh {
        assert!(!opt.key, "--key and --hh cannot be set simultaneously");
        assert!(!opt.raw, "--raw and --hh cannot be set simultaneously");
        assert!(!opt.merge, "--merge and --hh cannot be set simultaneously");
        if k == 0 {
            return
        }
        let reduced =
            reduce_stream(io::stdin().lock(), HeavyHitter::new(k)).expect("no io error");
        for (line, count) in reduced.estimate() {
            println!("{} {}", count, str::from_utf8(line).expect("valid UTF-8"));
        }
        return
    }

    match (opt.key, opt.merge) {
        (true, false) => {
            let reduced =
                reduce_stream(io::stdin().lock(), KeyedCounter::default()).expect("no io error");
            print_dict(reduced.state(), opt.raw)
        }
        (false, false) => {
            let reduced =
                reduce_stream(io::stdin().lock(), Counter::default()).expect("no io error");
            print_single(&reduced, opt.raw);
        }
        (true, true) => {
            let reduced =
                reduce_stream(io::stdin().lock(), KeyedMerger::default()).expect("no io error");
            for (key, ctr) in reduced.state() {
                print_dict(iter::once((key, &ctr)), opt.raw)
            }
        }
        (false, true) => {
            let reduced =
                reduce_stream(io::stdin().lock(), Merger::default()).expect("no io error");
            print_single(&reduced.counter(), opt.raw)
        }
    }
}

fn print_dict<'a>(it: impl Iterator<Item = (&'a [u8], &'a Counter)>, raw: bool) {
    for (key, ctr) in it {
        let as_str = str::from_utf8(key).expect("valid UTF-8");
        print!("{} ", as_str);
        print_single(ctr, raw);
    }
}

fn print_single(c: &Counter, raw: bool) {
    if raw {
        println!("{}", c.serialize());
    } else {
        println!("{}", c.estimate().round());
    }
}

#[cfg(test)]
mod tests {

    use std::process;
    use std::str;

    use assert_cmd;
    use itertools::Itertools;

    fn sort_lines(stdout: Vec<u8>) -> Vec<u8> {
        let mut lines: Vec<_> = stdout
            .split(|c| *c == b'\n')
            .map(|s| s.to_owned())
            .collect();
        lines.sort_unstable();
        lines.rotate_left(1); // final newline to back
        lines.join(&b'\n')
    }

    fn communicate(stdin: Vec<u8>, dsrs_flags: &[&str]) -> Vec<u8> {
        let out = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("command created")
            .args(dsrs_flags)
            .write_stdin(stdin)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(out.stderr.is_empty(), "stderr {}",
                str::from_utf8(&out.stderr).expect("valid UTF-8"));
        out
            .stdout
    }

    fn eval_bash(cmd: &str) -> Vec<u8> {
        let out = process::Command::new("/bin/bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("datagen process successful");
        assert!(out.stderr.is_empty(), "{}",
                str::from_utf8(&out.stderr).unwrap());
        out.stdout
    }

    /// Asserts that the outputs of dsrs and unix tools when
    /// fed the input from datagen are equal.
    fn validate_equal(datagen: &str, keyed: bool, unix: &str) {
        let ref args = if keyed { vec!["--key"] } else { vec![] };
        validate_equal_cmd(datagen, args, unix);
        let stdin = eval_bash(datagen);
        let dsrs_stdout = communicate(stdin.clone(), args);
        let dsrs_stdout = sort_lines(dsrs_stdout);

        // to check merge, split stdin lines into thirds two different
        // ways (modulo and simply cutting it up)
        // result should still be the same

        let groups = stdin
            .split(|c| *c == b'\n')
            .enumerate()
            .into_group_map_by(|(i, _)| i % 3)
            .into_iter()
            .map(|(_, v)| {
                v.into_iter()
                    .map(|(_, vv)| vv)
                    .collect::<Vec<_>>()
                    .join(&b'\n')
            })
            .collect();
        let modulo_stdout = reduce_with_merge(groups, keyed);
        assert_eq!(
            &modulo_stdout,
            &dsrs_stdout,
            "\nmodulo:\n{}\ndsrs:\n{}",
            str::from_utf8(&modulo_stdout).expect("valid UTF-8"),
            str::from_utf8(&dsrs_stdout).expect("valid UTF-8")
        );

        let nlines = stdin.split(|c| *c == b'\n').count() - 1;
        let groups: Vec<_> = stdin
            .split(|c| *c == b'\n')
            .enumerate()
            .into_group_map_by(|(i, _)| (i * 2) / nlines)
            .into_iter()
            .map(|(_, v)| {
                v.into_iter()
                    .map(|(_, vv)| vv)
                    .collect::<Vec<_>>()
                    .join(&b'\n')
            })
            .collect();
        let chunked_stdout = reduce_with_merge(groups, keyed);
        assert_eq!(
            &chunked_stdout,
            &dsrs_stdout,
            "\nchunked:\n{}\ndsrs:\n{}",
            str::from_utf8(&chunked_stdout).expect("valid UTF-8"),
            str::from_utf8(&dsrs_stdout).expect("valid UTF-8")
        );
    }

    fn reduce_with_merge(groups: Vec<Vec<u8>>, keyed: bool) -> Vec<u8> {
        let raw: Vec<_> = groups
            .into_iter()
            .map(|stdin| {
                let flags: &[&str] = if keyed {
                    &["--key", "--raw"]
                } else {
                    &["--raw"]
                };
                communicate(stdin, flags)
            })
            .flatten()
            .collect();
        let flags: &[&str] = if keyed {
            &["--key", "--merge"]
        } else {
            &["--merge"]
        };
        let stdout = communicate(raw, flags);
        sort_lines(stdout)
    }

    const UNIX_COUNT_DISTINCT: &'static str = "sort --unique | wc -l";

    #[test]
    fn unique_lines() {
        validate_equal("seq 100", false, UNIX_COUNT_DISTINCT)
    }

    #[test]
    fn equally_dup_lines() {
        validate_equal("seq 100 && seq 100 && seq 100", false, UNIX_COUNT_DISTINCT)
    }

    #[test]
    fn unequally_dup_lines() {
        validate_equal("seq 100 | xargs -L1 seq", false, UNIX_COUNT_DISTINCT)
    }

    #[test]
    fn count_empty() {
        validate_equal("echo ; echo ; echo 1", false, UNIX_COUNT_DISTINCT)
    }

    /// Only works for single-char keys due to -w1, note col order swap.
    const UNIX_GROUPBY_COUNT_DISTINCT: &'static str =
        "sort --unique | uniq -w1 -c | awk '{print$2\" \"$1}'";

    #[test]
    fn unique_keyed_lines() {
        validate_equal(
            "(seq 100 | xargs -L1 echo 1) && \
             (seq 50 | xargs -L1 echo 2) && \
             (seq 25 | xargs -L1 echo 3)",
            true,
            UNIX_GROUPBY_COUNT_DISTINCT,
        )
    }

    #[test]
    fn keyed_dup_lines() {
        validate_equal(
            "(seq 100 | xargs -L1 echo 1) && \
             (seq 50  | xargs -L1 echo 2) && \
             (seq 100 | xargs -L1 echo 1) && \
             (seq 50  | xargs -L1 echo 2) && \
             (seq 100 | xargs -L1 echo 1) && \
             (seq 50  | xargs -L1 echo 2)",
            true,
            UNIX_GROUPBY_COUNT_DISTINCT,
        )
    }

    #[test]
    fn keyed_count_empty() {
        validate_equal(
            "echo \"1 \"; echo 1 1; echo 1 3",
            true,
            UNIX_GROUPBY_COUNT_DISTINCT,
        )
    }

    fn validate_equal_cmd(datagen: &str, args: &[&str], unix: &str) {
        let stdin = eval_bash(datagen);
        let dsrs_stdout = communicate(stdin.clone(), args);
        let unix_stdout = eval_bash(&format!("({}) | ({})", datagen, unix));
        let dsrs_stdout = sort_lines(dsrs_stdout);
        assert_eq!(
            &unix_stdout,
            &dsrs_stdout,
            "\nunix:\n{}\ndsrs:\n{}",
            str::from_utf8(&unix_stdout).expect("valid UTF-8"),
            str::from_utf8(&dsrs_stdout).expect("valid UTF-8")
        );
    }

    fn unix_hh(k: usize) -> String {
        format!("sort | uniq -c | sort -rn | head -{} | sed 's/^ *//' | sort", k)
    }

    fn validate_unix_hh(datagen: &str, k: usize) {
        let unix = unix_hh(k);
        let kstr = format!("{}", k);
        let dsrs = &["--hh", &kstr];
        validate_equal_cmd(datagen, dsrs, &unix);        
    }

    #[test]
    fn hh_unique_lines() {
        validate_unix_hh("seq 100", 100);
    }

    #[test]
    fn hh_equally_dup_lines() {
        validate_unix_hh("seq 1000 | sed 's/$/\\n1\\n2\\n3/'", 3);
    }

    #[test]
    fn hh_count_empty() {
        validate_unix_hh("echo ; echo ; echo 1", 1)
    }
}
