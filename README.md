# DataSketches in Rust

A Rust binding for the [Apache DataSketches](https://datasketches.apache.org/) library.

At this point, this package only wraps the count-distinct CPC sketch and provides a command-line tool, `dsrs`, for approximate `sort | uniq -c` functionality.

## Installation

Assumes a modern Rust `cargo` is installed.

```
cargo install dsrs
```

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
```

This is all only possible thanks to the excellent [dtolnay/cxx](https://github.com/dtolnay/cxx) library!
