#![feature(default_type_params, associated_types)]

#[cfg(test)] extern crate test;
extern crate xxhash;

use std::default::Default;
use std::hash::{Hasher, Hash};
use std::iter::{FromIterator, Iterator};
use std::borrow::BorrowFrom;
use std::collections::HashMap;
use std::collections::hash_map::{Iter, Keys};

use xxhash::{XXHasher, XXState};

/// A distribution of K's. V stores the distribution of 
/// K's (most likely will be numeric). Implementors will most 
/// likely define what K is. 
/// Distribution doesn't require a cryptographically secure hash, and by 
/// default will not use one.
pub trait Distribution<K: Eq + Hash<S>, V, S = XXState, H: Hasher<S> = XXHasher> 
for Sized? {
  fn len(&self) -> uint;
  fn get<Sized? Q>(&self, k: &Q) -> Option<&V> where Q: Hash<S> + Eq + BorrowFrom<K>;
  fn clear(&mut self);
  fn insert(&mut self, k: K);
  fn remove<Sized? Q>(&mut self, k: &Q) where Q: Hash<S> + Eq + BorrowFrom<K>;
}

/// Implementation of a Frequency Distribution in Rust. Keeps track of how many 
/// times an object appears in a larger context (for example, how many times a 
/// token appears in a piece of text). The underlying data structure of the 
/// Frequency Distribution is a HashMap, so the object that is being counted
/// must be hashable.
///
/// # Example
///
/// ```rust
/// use freqdist::{Distribution, FrequencyDistribution};
///
/// let mut fdist = FrequencyDistribution::new();
///
/// fdist.insert("hello");
/// fdist.insert("hello");
/// fdist.insert("goodbyte");
///
/// assert_eq!(*fdist.get("hello").unwrap(), 2);
/// ```
pub struct FrequencyDistribution<K, H = XXHasher> {
  hashmap: HashMap<K, uint, H>,
  sum_counts: uint
}

impl<K: Eq + Hash<XXState>> FrequencyDistribution<K, XXHasher> {
  /// Creates a new FrequencyDistribution where the size of the
  /// HashMap is unknown.
  pub fn new() -> FrequencyDistribution<K> {
    FrequencyDistribution::with_hasher(XXHasher::new())
  }
  
  /// Creates a new FrequencyDistribution where the size of the HashMap
  /// is known, or a estimate can be made.
  pub fn with_capacity(size: uint) -> FrequencyDistribution<K> {
    FrequencyDistribution::with_capacity_and_hasher(size, XXHasher::new())
  }
}

impl<K: Eq + Hash<S>, S, H: Hasher<S>> FrequencyDistribution<K, H> {
  /// Creates a new FrequencyDistrbution with a hasher and size, where 
  /// the size is known or can be estimated.
  pub fn with_capacity_and_hasher(
    size: uint, 
    hasher: H
  ) -> FrequencyDistribution<K, H> {
    FrequencyDistribution {
      hashmap: HashMap::with_capacity_and_hasher(size, hasher),
      sum_counts: 0    
    }
  }

  /// Creates a new FrequencyDistribution with a hasher and default size.
  pub fn with_hasher(hasher: H) -> FrequencyDistribution<K, H> {
    FrequencyDistribution {
      hashmap: HashMap::with_hasher(hasher),
      sum_counts: 0
    }
  }
}

impl<K: Eq + Hash<S>, S, H: Hasher<S>> FrequencyDistribution<K, H> {
  /// Iterator over the key, frequency pairs.
  #[inline]
  pub fn iter(&self) -> Iter<K, uint> {
    self.hashmap.iter()
  }

  /// Iterator over the non-zero frequency keys.
  #[inline]
  pub fn iter_non_zero(&self) -> NonZeroKeysIter<K> {
    NonZeroKeysIter { iter: self.iter() }
  }

  /// Iterator over just the keys.
  #[inline]
  pub fn keys(&self) -> Keys<K, uint> {
    self.hashmap.keys()
  }

  /// Returns the total number of values tallied.
  #[inline(always)]
  pub fn sum_counts(&self) -> uint {
    self.sum_counts
  }

  /// Inserts a value into the hashmap if it does not exist with a new quantity
  /// specified by the increment. If the value already exists, increments by 
  /// the specified amount.
  fn insert_or_incr_by(&mut self, k: K, incr: uint) {
    if !self.hashmap.contains_key(&k) {
      self.hashmap.insert(k, incr);
    } else {
      *self.hashmap.get_mut(&k).unwrap() += incr;
    }

    self.sum_counts += incr;
  }
}

impl<K: Eq + Hash<XXState>> Default for FrequencyDistribution<K> {
  /// Creates a default FrequencyDistribution with an XXHasher.
  fn default() -> FrequencyDistribution<K> {
    FrequencyDistribution::new()
  }
}

impl<K: Eq + Hash<S>, S, H: Hasher<S> + Default> FromIterator<(K, uint)> 
for FrequencyDistribution<K, H> {
  /// Iterates through an iterator, and creates a new FrequencyDistribution from 
  /// it. The iterator should be an iterator over keys and frequencies. If a 
  /// upper bounded `size_hint` is available, then it is used, otherwise the lower 
  /// bounded `size_hint` is used.
  ///
  /// # Example
  ///
  /// ```rust
  /// use std::iter::FromIterator;
  /// use freqdist::{Distribution, FrequencyDistribution};
  ///
  /// let existing = vec![
  ///   ("apples", 3),
  ///   ("oranges", 4),
  ///   ("bannana", 7)
  /// ];
  ///
  /// let fdist: FrequencyDistribution<&str> = FromIterator::from_iter(existing.into_iter());
  ///
  /// assert_eq!(*fdist.get("apples").unwrap(), 3);
  /// assert_eq!(*fdist.get("oranges").unwrap(), 4);
  /// assert_eq!(*fdist.get("bannana").unwrap(), 7);
  /// ```
  fn from_iter<T: Iterator<Item = (K, uint)>>(iter: T) -> FrequencyDistribution<K, H> {
    let mut fdist = if iter.size_hint().1.is_some() {
      FrequencyDistribution::with_capacity_and_hasher(
        iter.size_hint().1.unwrap(),
        Default::default())
    } else {
      FrequencyDistribution::with_capacity_and_hasher(
        iter.size_hint().0, 
        Default::default())
    };

    fdist.extend(iter);
    fdist
  }
}

impl<K: Eq + Hash<S>, S, H: Hasher<S>> Extend<(K, uint)> 
for FrequencyDistribution<K, H> {
  /// Extends the hashmap by adding the keys or updating the frequencies of the keys.
  fn extend<T: Iterator<Item = (K, uint)>>(&mut self, mut iter: T) {
    for (k, freq) in iter {
      self.insert_or_incr_by(k, freq);
    }
  }
}

impl<K: Eq + Hash<S>, S, H: Hasher<S>> Distribution<K, uint, S, H> 
for FrequencyDistribution<K, H> {
  /// Returns the number of entries in the distribution
  #[inline]
  fn len(&self) -> uint {
    self.hashmap.len()
  }

  /// Gets the frequency in which the key occurs.
  #[inline]
  fn get<Sized? Q>(&self, k: &Q) -> Option<&uint> 
    where Q: Hash<S> + Eq + BorrowFrom<K>
  {
    self.hashmap.get(k)
  }

  /// Clears the counts of all keys and clears all keys from 
  /// the distribution.
  #[inline]
  fn clear(&mut self) {
    self.hashmap.clear()
  }

  /// Updates the frequency of the value found with the key if it 
  /// already exists. Otherwise, inserts the key into the hashmap, 
  /// and sets its frequency to 1.
  #[inline]
  fn insert(&mut self, k: K) {
    self.insert_or_incr_by(k, 1);
  }

  /// Removes a Key and its associated value from the Distrbution.
  #[inline]
  fn remove<Sized? Q>(&mut self, k: &Q) where Q: Hash<S> + Eq + BorrowFrom<K>
  {
    match self.hashmap.remove(k) {
      Some(count) => self.sum_counts -= count,
      None        => ()
    }
  }
}


/// Iterator over entries with non-zero quantities.
pub struct NonZeroKeysIter<'a, K: 'a> {
  iter: Iter<'a, K, uint> 
}

impl<'a, K: 'a> Iterator for NonZeroKeysIter<'a, K> {
  type Item = &'a K;

  #[inline]
  fn next(&mut self) -> Option<&'a K> {
    loop {
      match self.iter.next() {
        Some((k, c)) if *c > 0 => return Some(k),
        None                   => return None,
        _                      => ()
      }
    }
  }
  
  #[inline]
  fn size_hint(&self) -> (uint, Option<uint>) {
    self.iter.size_hint()
  }
}

#[cfg(test)]
mod test_frequency_distribution {
  use std::iter::FromIterator;
  use super::{Distribution, FrequencyDistribution};

  #[test]
  fn smoke_test_frequency_distribution_insert() {
    let words = vec!("alpha", "beta");
    let mut dist: FrequencyDistribution<&str> = FrequencyDistribution::new();

    dist.insert(words[0]);
    
    assert_eq!(*dist.get(&words[0]).unwrap(), 1);

    dist.insert(words[1]);

    assert_eq!(*dist.get(&words[1]).unwrap(), 1);

    for _ in range(0, 7u) { dist.insert(words[0]); }

    assert_eq!(*dist.get(&words[0]).unwrap(), 8);
  }

  #[test]
  fn smoke_test_frequency_distribution_iter() {
    let words = vec!(("a", 50u), ("b", 100u), ("c", 75u), ("d", 0u));
    let dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

    assert_eq!(*dist.get(&"a").unwrap(), 50);
    assert_eq!(*dist.get(&"b").unwrap(), 100);
    assert_eq!(*dist.get(&"c").unwrap(), 75);

    let mut iter = dist.iter_non_zero();

    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());

    assert_eq!(dist.sum_counts(), 225);
  }

  #[test]
  fn smoke_test_frequency_distribution_remove() {
    let words = vec!(("a", 50u), ("b", 100u), ("c", 25u));
    let mut dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

    assert!(dist.get(&"a").is_some());

    dist.remove(&"a");

    assert!(dist.get(&"a").is_none());
    assert_eq!(dist.sum_counts(), 125);
  }

  #[test]
  fn smoke_test_frequency_sum_counts() {
    let words = vec!(("a", 7u), ("b", 5u), ("c", 8u), ("d", 3u));
    let mut dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

    assert_eq!(dist.sum_counts(), 23);

    dist.insert("e");

    assert_eq!(dist.sum_counts(), 24);
  }
}
