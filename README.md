# Frequency Distribution

[![Build Status](https://travis-ci.org/ferristseng/rust-freqdist.svg)](https://travis-ci.org/ferristseng/rust-freqdist)
[![](http://meritbadge.herokuapp.com/rust-freqdist)](https://crates.io/crates/rust-freqdist)

Implementation of a Frequency Distribution in Rust. Keeps track of how many 
times an object appears in a larger context (for example, how many times a 
word appears in a piece of text). The underlying data structure of the 
Frequency Distribution is a HashMap, so the object that is being counted
must be hashable.

# Example

```rust
use freqdist::FrequencyDistribution;

let mut fdist = FrequencyDistribution::new();

fdist.insert("hello");
fdist.insert("hello");
fdist.insert("goodbye");

assert_eq!(fdist.get(&"hello"), 2);
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
