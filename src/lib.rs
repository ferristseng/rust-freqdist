// Copyright 2016 rust-freqdist Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Implementation of a Frequency Distribution in Rust. Keeps track of how many 
//! times an object appears in a larger context (for example, how many times a 
//! word appears in a piece of text). The underlying data structure of the 
//! Frequency Distribution is a HashMap, so the object that is being counted
//! must be hashable.
//!
//! # Example
//!
//! ```
//! # use freqdist::FrequencyDistribution;
//! #
//! let mut fdist: FrequencyDistribution<&str> = FrequencyDistribution::new();
//!
//! fdist.insert("hello");
//! fdist.insert("hello");
//! fdist.insert("goodbye");
//!
//! assert_eq!(fdist.get(&"hello"), 2);
//! ```

#![warn(missing_docs)]
#![cfg_attr(test, feature(test))]

#[cfg(test)] extern crate test;

use std::ops::Index;
use std::default::Default;
use std::hash::{Hasher, Hash, BuildHasher, SipHasher};
use std::iter::{FromIterator, IntoIterator};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::hash_map::{Keys, IntoIter, Iter, RandomState};


static ZERO: usize = 0;


#[allow(missing_docs)] pub struct FrequencyDistribution<K, S = RandomState> {
  hashmap: HashMap<K, usize, S>,
  sum_counts: usize
}

impl<K, H = SipHasher, S = RandomState> FrequencyDistribution<K, S>
  where K : Eq + Hash,
        H : Hasher,
        S : BuildHasher<Hasher = H>
{
  /// Creates a new FrequencyDistrbution with a hasher and size, where 
  /// the size is known or can be estimated.
  #[inline(always)] pub fn with_capacity_and_hasher(
    size: usize, 
    state: S 
  ) -> FrequencyDistribution<K, S> {
    FrequencyDistribution {
      hashmap: HashMap::with_capacity_and_hasher(size, state),
      sum_counts: 0    
    }
  }

  /// Creates a new FrequencyDistribution with a hasher and default size.
  #[inline(always)] pub fn with_hasher(state: S) -> FrequencyDistribution<K, S> {
    FrequencyDistribution {
      hashmap: HashMap::with_hasher(state),
      sum_counts: 0
    }
  }

  /// Iterator over the keys.
  #[inline(always)] pub fn keys(&self) -> Keys<K, usize> {
    self.hashmap.keys()
  }

  /// Iterator over the key, frequency pairs.
  #[inline(always)] pub fn iter(&self) -> Iter<K, usize> {
    self.hashmap.iter()
  }

  /// Iterator over the non-zero frequency keys.
  /// 
  /// # Example
  ///
  /// ```
  /// # use std::iter::{FromIterator, IntoIterator};
  /// # use freqdist::FrequencyDistribution;
  /// #
  /// let existing = vec![
  ///   ("shoes", 1),
  ///   ("scarf", 0),
  ///   ("shirt", 13),
  ///   ("pants", 4)
  /// ];
  ///
  /// let fdist: FrequencyDistribution<&str> = 
  ///   FromIterator::from_iter(existing.into_iter());
  /// let mut iter = fdist.iter_non_zero();
  ///
  /// assert_eq!(*iter.next().unwrap(), "shirt");
  /// assert_eq!(*iter.next().unwrap(), "shoes");
  /// assert_eq!(*iter.next().unwrap(), "pants");
  /// assert!(iter.next().is_none());
  /// ```
  #[inline(always)] pub fn iter_non_zero(&self) -> NonZeroKeysIter<K> {
    NonZeroKeysIter { iter: self.iter() }
  }

  /// Sum of the total number of items counted thus far. 
  #[inline(always)] pub fn sum_counts(&self) -> usize {
    self.sum_counts
  }

  /// Returns the number of entries in the distribution
  #[inline(always)] pub fn len(&self) -> usize {
    self.hashmap.len()
  }

  /// Gets the frequency in which the key occurs.
  #[inline(always)] pub fn get<Q : ?Sized>(&self, k: &Q) -> usize 
    where K : Borrow<Q>, Q : Hash + Eq
  {
    self[k]
  }

  /// Clears the counts of all keys and clears all keys from 
  /// the distribution.
  #[inline(always)] pub fn clear(&mut self) {
    self.hashmap.clear()
  }

  /// Updates the frequency of the value found with the key if it 
  /// already exists. Otherwise, inserts the key sizeo the hashmap, 
  /// and sets its frequency to 1.
  #[inline(always)] pub fn insert(&mut self, k: K) {
    self.insert_or_incr_by(k, 1);
  }

  /// Removes an item and its associated counts.
  #[inline(always)] pub fn remove<Q : ?Sized>(&mut self, k: &Q) 
    where K : Borrow<Q>, Q : Hash + Eq
  {
    match self.hashmap.remove(k) {
      Some(count) => self.sum_counts -= count,
      None => ()
    }
  }

  /// Inserts a value sizeo the hashmap if it does not exist with a new quantity
  /// specified by the increment. If the value already exists, increments by 
  /// the specified amount.
  #[inline] fn insert_or_incr_by(&mut self, k: K, incr: usize) {
    if !self.hashmap.contains_key(&k) {
      self.hashmap.insert(k, incr);
    } else {
      *self.hashmap.get_mut(&k).unwrap() += incr;
    }

    self.sum_counts += incr;
  }
}

impl<K, H = SipHasher, S = RandomState> FrequencyDistribution<K, S> 
  where K : Eq + Hash, 
        H : Hasher + Default, 
        S : BuildHasher<Hasher = H> + Default
{
  /// Creates a new FrequencyDistribution where the size of the
  /// HashMap is unknown.
  #[inline(always)] pub fn new() -> FrequencyDistribution<K, S> {
    FrequencyDistribution::with_hasher(Default::default())
  }
  
  /// Creates a new FrequencyDistribution where the size of the HashMap
  /// is known, or a estimate can be made.
  #[inline(always)] pub fn with_capacity(
    size: usize
  ) -> FrequencyDistribution<K, S> {
    FrequencyDistribution::with_capacity_and_hasher(size, Default::default())
  }
}

impl<K, H = SipHasher, S = RandomState> Default for FrequencyDistribution<K, S> 
  where K : Eq + Hash,
        H : Hasher + Default,
        S : BuildHasher<Hasher = H> + Default
{
  /// Creates a default FrequencyDistribution.
  #[inline(always)] fn default() -> FrequencyDistribution<K, S> {
    FrequencyDistribution::new()
  }
}

impl<K, H, S> FromIterator<(K, usize)> for FrequencyDistribution<K, S> 
  where K : Eq + Hash, 
        H : Hasher,
        S : BuildHasher<Hasher = H> + Default 
{ 
  /// Iterates through an iterator, and creates a new FrequencyDistribution from 
  /// it. The iterator should be an iterator over keys and frequencies. If a 
  /// upper bounded `size_hsize` is available, then it is used, otherwise the lower 
  /// bounded `size_hsize` is used.
  ///
  /// # Example
  ///
  /// ```
  /// # use std::iter::FromIterator;
  /// # use freqdist::FrequencyDistribution;
  /// #
  /// let existing = vec![
  ///   ("apples", 3),
  ///   ("oranges", 4),
  ///   ("bannana", 7)
  /// ];
  ///
  /// let fdist: FrequencyDistribution<&str> = 
  ///   FromIterator::from_iter(existing.into_iter());
  ///
  /// assert_eq!(fdist.get(&"apples"), 3);
  /// assert_eq!(fdist.get(&"oranges"), 4);
  /// assert_eq!(fdist.get(&"bannana"), 7);
  /// ```
  fn from_iter<T>(iter: T) -> FrequencyDistribution<K, S> 
    where T : IntoIterator<Item = (K, usize)> 
  {
    let iterator = iter.into_iter();
    let mut fdist = if iterator.size_hint().1.is_some() {
      FrequencyDistribution::with_capacity_and_hasher(
        iterator.size_hint().1.unwrap(),
        Default::default())
    } else {
      FrequencyDistribution::with_capacity_and_hasher(
        iterator.size_hint().0, 
        Default::default())
    };

    for (k, freq) in iterator { fdist.insert_or_incr_by(k, freq); }

    fdist
  }
}

impl<K, H, S> Extend<(K, usize)> for FrequencyDistribution<K, S> 
  where K : Eq + Hash, 
        H : Hasher, 
        S : BuildHasher<Hasher = H>
{
  /// Extends the hashmap by adding the keys or updating the frequencies of the keys.
  fn extend<T>(&mut self, iter: T) 
    where T: IntoIterator<Item = (K, usize)>
  {
    for (k, freq) in iter.into_iter() { self.insert_or_incr_by(k, freq); }
  }
}

impl<K, H, S> IntoIterator for FrequencyDistribution<K, S>
  where K : Eq + Hash,
        H : Hasher,
        S : BuildHasher<Hasher = H>
{
  type Item = (K, usize);
  type IntoIter = IntoIter<K, usize>;

  /// Consumes the distribution, and creates an iterator over the 
  /// (Key, Quantity: usize) pairs.
  #[inline] fn into_iter(self) -> IntoIter<K, usize> {
    self.hashmap.into_iter()
  }
}

impl<'a, K, H, S, Q : ?Sized> Index<&'a Q> for FrequencyDistribution<K, S>
  where K : Eq + Hash + Borrow<Q>,
        H : Hasher,
        S : BuildHasher<Hasher = H>,
        Q : Eq + Hash
{
  type Output = usize;

  #[inline] fn index<'b>(&'b self, index: &Q) -> &'b usize {
    self.hashmap.get(index).unwrap_or(&ZERO)
  }
}

/// Iterator over entries with non-zero quantities.
pub struct NonZeroKeysIter<'a, K: 'a> {
  iter: Iter<'a, K, usize> 
}

impl<'a, K: 'a> Iterator for NonZeroKeysIter<'a, K> {
  type Item = &'a K;

  #[inline(always)] fn next(&mut self) -> Option<&'a K> {
    loop {
      match self.iter.next() {
        Some((k, c)) if *c > 0 => return Some(k),
        None => return None,
        _ => ()
      }
    }
  }
}

#[test]
fn smoke_test_frequency_distribution_insert() {
  let words = vec!("alpha", "beta");
  let mut dist: FrequencyDistribution<&str> = FrequencyDistribution::new();

  dist.insert(words[0]);
  
  assert_eq!(dist.get(&words[0]), 1);

  dist.insert(words[1]);

  assert_eq!(dist.get(&words[1]), 1);

  for _ in (0..7u32) { dist.insert(words[0]); }

  assert_eq!(dist.get(&words[0]), 8);
}

#[test]
fn smoke_test_frequency_distribution_iter() {
  let words = vec!(("a", 50usize), ("b", 100usize), ("c", 75usize), ("d", 0usize));
  let dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

  assert_eq!(dist.get(&"a"), 50);
  assert_eq!(dist.get(&"b"), 100);
  assert_eq!(dist.get(&"c"), 75);

  let mut iter = dist.iter_non_zero();

  assert!(iter.next().is_some());
  assert!(iter.next().is_some());
  assert!(iter.next().is_some());
  assert!(iter.next().is_none());

  assert_eq!(dist.sum_counts(), 225);
}

#[test]
fn smoke_test_frequency_distribution_remove() {
  let words = vec!(("a", 50usize), ("b", 100usize), ("c", 25usize));
  let mut dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

  assert_eq!(dist.get(&"a"), 50);

  dist.remove(&"a");

  assert_eq!(dist.get(&"a"), 0);
  assert_eq!(dist.sum_counts(), 125);
}

#[test]
fn smoke_test_frequency_sum_counts() {
  let words = vec!(("a", 7usize), ("b", 5usize), ("c", 8usize), ("d", 3usize));
  let mut dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

  assert_eq!(dist.sum_counts(), 23);

  dist.insert("e");

  assert_eq!(dist.sum_counts(), 24);
}
