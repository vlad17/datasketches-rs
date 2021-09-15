# DataSketches in Rust

A Rust binding for the [Apache DataSketches](https://datasketches.apache.org/) library and command-line tool.

On the command-line, we provide

  - `dsrs [--key] [--raw] [--merge]` for approximate distinct line-counting, and
  - `dsrs --hh k` for heavy hitters (approximate most frequent lines).

For instance, the following experiment checks how many unique lines exist when you print all numbers up to 100M twice.

```bash
m100=$((100 * 1000 * 1000))
(seq $m100 && seq $m100) | \
  /usr/bin/time -f "%e sec %M KB" dsrs
102055590
5.22 sec 4288 KB

(seq $m100 && seq $m100) | \
  /usr/bin/time -f "%e sec %M KB" sort -u | wc -l
438.66 sec 12880 KB
100000000

(seq $m100 && seq $m100) | \
  /usr/bin/time -f "%e sec %M KB" awk '{a[$0]=1}END{print length(a)}'
100000000
39.28 sec 898240 KB
```

Next, we can ask for the most popular lines from a stream (there is a [topfew](https://github.com/djc/topfew-rs) Rust package, but it does not support streams).

```bash
m10=$((10 * 1000 * 1000))
seq $m10 | sed 's/$/\n1\n2\n3/' | \
  /usr/bin/time -f "%e sec %M KB" sort | \
  uniq -c | sort -rn | head -3
54.88 sec 8968 KB
10000001 3
10000001 2
10000001 1
  
# exact hashmap solution, requires go
pushd /tmp && \
  (test -d topfew || git clone git@github.com:timbray/topfew.git topfew) && \
  pushd topfew && make && popd && popd
seq $m10 | sed 's/$/\n1\n2\n3/' | \
  /usr/bin/time -f "%e sec %M KB" /tmp/topfew/bin/tf -f 1 -n 3
10000001 2
10000001 3
10000001 1
10.67 sec 1060332 KB
  
seq $m10 | sed 's/$/\n1\n2\n3/' | \
  /usr/bin/time -f "%e sec %M KB" target/release/dsrs --hh 3
10000001 2
10000001 1
10000001 3
4.48 sec 4560 KB
```

Here's a sophisticated example of the tool [in action](https://vladfeinberg.com/2021/06/29/amazon-reviewers-with-sketches.html), used to compute rolling average active reviewers for Amazon over a couple decades. The equivalent non-sketch based solution OOMs.

## Installation

Assumes a modern Rust `cargo` is installed. The command line tool `dsrs` can be installed with:

```
cargo install dsrs
```

The library may be used as a regular Rust dependency by adding it to your `Cargo.toml` file.

## Embedded C++ Library

This Rust library contains manually-copied header files from the header-only `datasketches-cpp` library at commit [043b947f](https://github.com/apache/datasketches-cpp/tree/043b947fe5b1f9b82527deb0eea4da32f5764f6c).

This was done by extracting all headers. Assuming you're in the `datasketches-rs` directory, which has a sibling `datasketches-cpp`:

```
# make all required directories
find ../datasketches-cpp/ -name "*.h" -or -name "*.hpp" | \
  xargs dirname | \
  sort -u |
  cut -d/ -f2- | \
  xargs mkdir -p
# copy over the actual headers
find ../datasketches-cpp/ -name "*.h" -or -name "*.hpp" | \
  cut -d/ -f2- | \
  xargs -I {} cp ../{} {}
# and the license info too
cp ../datasketches-cpp/{NOTICE,LICENSE} datasketches-cpp/

# some manual interventions were required for the heavy hitters
# implementation, which requires the C++ side to temporarily own
# keys from Rust, so additional management code needs to be injected.
git apply fi.patch
```

This is all only possible thanks to the excellent [dtolnay/cxx](https://github.com/dtolnay/cxx) library!

## Why DataSketches in Rust?

There are quite a few crates containing HyperLogLog sketches. However, when I attempted to use them (as of 2021-06-20), I found that their APIs panicked on certain inputs (e.g., try `amadeus_streaming::HyperLogLog::<u64>::new(0.0001);`), or did not have merge operations. A very rudimentary `cargo criterion` on 1M unique keys finds that CPC has better accuracy anyway (for all of the below, the same nominal accuracy configuration was set, so these should be using roughly the same amount of memory):

```
repeat-ten/dsrs::CpcSketch/1000000
                        time:   [144.95 ms 149.27 ms 155.42 ms]
repeat-ten/amadeus_streaming::HyperLogLog/1000000
                        time:   [132.89 ms 134.01 ms 135.49 ms]
repeat-ten/probabilistic_collections::HyperLogLog/1000000
                        time:   [159.99 ms 165.94 ms 172.32 ms]
repeat-ten/probably::HyperLogLog/1000000
                        time:   [119.47 ms 123.95 ms 127.84 ms]
repeat-ten/hyperloglogplus::HyperLogLogPlus/1000000
                        time:   [120.74 ms 121.32 ms 122.10 ms]

relative errors
size: 1000000
  relerr:   1.1% name: dsrs::CpcSketch
  relerr:   3.3% name: amadeus_streaming::HyperLogLog
  relerr:   4.3% name: hyperloglogplus::HyperLogLogPlus
  relerr:  50.7% name: probably::HyperLogLog
  relerr:   inf% name: probabilistic_collections::HyperLogLog
```

while overall update speed doesn't change too much between implementations.
