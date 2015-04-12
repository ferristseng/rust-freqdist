# FreqDist

[![Build Status](https://travis-ci.org/ferristseng/rust-freqdist.svg)](https://travis-ci.org/ferristseng/rust-freqdist)

Provides a Frequency Distribution data structure, which can keep track of the number of times an object appears in some context. 

# Example

```rust
use freqdist::{Distribution, FrequencyDistribution};

let mut fdist = FrequencyDistribution::new();

fdist.insert("hello");
fdist.insert("hello");
fdist.insert("goodbye");

assert_eq!(fdist.get(&"hello"), 2);
```
