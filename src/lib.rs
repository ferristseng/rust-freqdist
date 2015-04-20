#![feature(std_misc)]
#![cfg_attr(test, feature(test))]

#[cfg(test)] extern crate test;

use std::ops::Index;
use std::default::Default;
use std::hash::{Hasher, Hash, SipHasher};
use std::iter::{FromIterator, IntoIterator};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::hash_map::{Keys, IntoIter, Iter};
use std::collections::hash_state::{HashState, DefaultState};

/// Distribution doesn't require a cryptographically secure hash, and by 
/// default will not use one.
pub trait Distribution<K, H = SipHasher> {
  type Quantity;

  fn len(&self) -> usize;
  fn get<Q: ?Sized>(&self, k: &Q) -> Self::Quantity
    where K: Borrow<Q>, Q: Hash + Eq; 
  fn clear(&mut self);
  fn insert(&mut self, k: K);
  fn remove<Q: ?Sized>(&mut self, k: &Q) 
    where K: Borrow<Q>, Q: Hash + Eq;
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
/// fdist.insert("goodbye");
///
/// assert_eq!(fdist.get(&"hello"), 2);
/// ```
pub struct FrequencyDistribution<K, S = DefaultState<SipHasher>> {
  hashmap: HashMap<K, usize, S>,
  sum_counts: usize
}

/// Used as a return value if a key is not found. 
static ZERO: &'static usize = &0;

impl<K> FrequencyDistribution<K> 
  where K: Eq + Hash 
{
  /// Creates a new FrequencyDistribution where the size of the
  /// HashMap is unknown.
  #[inline]
  pub fn new() -> FrequencyDistribution<K> {
    FrequencyDistribution::with_hash_state(Default::default())
  }
  
  /// Creates a new FrequencyDistribution where the size of the HashMap
  /// is known, or a estimate can be made.
  #[inline]
  pub fn with_capacity(
    size: usize
  ) -> FrequencyDistribution<K> {
    FrequencyDistribution::with_capacity_and_hash_state(size, Default::default())
  }
}

impl<K, S, H> FrequencyDistribution<K, S> 
  where K: Eq + Hash, 
        S: HashState<Hasher=H>, 
        H: Hasher 
{
  /// Creates a new FrequencyDistrbution with a hash state and size, where 
  /// the size is known or can be estimated.
  #[inline]
  pub fn with_capacity_and_hash_state(
    size: usize, 
    state: S 
  ) -> FrequencyDistribution<K, S> {
    FrequencyDistribution {
      hashmap: HashMap::with_capacity_and_hash_state(size, state),
      sum_counts: 0    
    }
  }

  /// Creates a new FrequencyDistribution with a hash state and default size.
  #[inline]
  pub fn with_hash_state(state: S) -> FrequencyDistribution<K, S> {
    FrequencyDistribution {
      hashmap: HashMap::with_hash_state(state),
      sum_counts: 0
    }
  }
}

impl<K, S, H> FrequencyDistribution<K, S> 
  where K: Eq + Hash,
        S: HashState<Hasher=H>,
        H: Hasher 
{
  /// Iterator over the keys.
  #[inline]
  #[stable]
  pub fn keys(&self) -> Keys<K, usize> {
    self.hashmap.keys()
  }

  /// Iterator over the key, frequency pairs.
  #[inline]
  #[stable]
  pub fn iter(&self) -> Iter<K, usize> {
    self.hashmap.iter()
  }

  /// Iterator over the non-zero frequency keys.
  /// 
  /// # Example
  ///
  /// ```rust
  /// use std::iter::{FromIterator, IntoIterator};
  /// use freqdist::FrequencyDistribution;
  ///
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
  #[inline]
  #[unstable]
  pub fn iter_non_zero(&self) -> NonZeroKeysIter<K> {
    NonZeroKeysIter { iter: self.iter() }
  }

  /// Sum of the total number of items counted thus far. 
  #[inline]
  #[stable]
  pub fn sum_counts(&self) -> usize {
    self.sum_counts
  }

  /// Inserts a value sizeo the hashmap if it does not exist with a new quantity
  /// specified by the increment. If the value already exists, increments by 
  /// the specified amount.
  #[inline]
  fn insert_or_incr_by(&mut self, k: K, incr: usize) {
    if !self.hashmap.contains_key(&k) {
      self.hashmap.insert(k, incr);
    } else {
      *self.hashmap.get_mut(&k).unwrap() += incr;
    }

    self.sum_counts += incr;
  }
}

impl<K> Default for FrequencyDistribution<K> 
  where K: Eq + Hash
{
  /// Creates a default FrequencyDistribution with an XXHasher.
  #[inline]
  #[stable]
  fn default() -> FrequencyDistribution<K> {
    FrequencyDistribution::new()
  }
}

impl<K, S, H> FromIterator<(K, usize)> for FrequencyDistribution<K, S> 
  where K: Eq + Hash, 
        S: HashState<Hasher = H> + Default, 
        H: Hasher 
{ 
  /// Iterates through an iterator, and creates a new FrequencyDistribution from 
  /// it. The iterator should be an iterator over keys and frequencies. If a 
  /// upper bounded `size_hsize` is available, then it is used, otherwise the lower 
  /// bounded `size_hsize` is used.
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
  /// let fdist: FrequencyDistribution<&str> = 
  ///   FromIterator::from_iter(existing.into_iter());
  ///
  /// assert_eq!(fdist.get(&"apples"), 3);
  /// assert_eq!(fdist.get(&"oranges"), 4);
  /// assert_eq!(fdist.get(&"bannana"), 7);
  /// ```
  fn from_iter<T>(iter: T) -> FrequencyDistribution<K, S> 
    where T: IntoIterator<Item = (K, usize)> 
  {
    let iterator = iter.into_iter();
    let mut fdist = if iterator.size_hint().1.is_some() {
      FrequencyDistribution::with_capacity_and_hash_state(
        iterator.size_hint().1.unwrap(),
        Default::default())
    } else {
      FrequencyDistribution::with_capacity_and_hash_state(
        iterator.size_hint().0, 
        Default::default())
    };

    for (k, freq) in iterator { fdist.insert_or_incr_by(k, freq); }

    fdist
  }
}

impl<K, S, H> Extend<(K, usize)> for FrequencyDistribution<K, S> 
  where K: Eq + Hash, 
        S: HashState<Hasher=H>,
        H: Hasher 
{
  /// Extends the hashmap by adding the keys or updating the frequencies of the keys.
  #[inline]
  fn extend<T>(&mut self, iter: T) 
    where T: IntoIterator<Item = (K, usize)>
  {
    for (k, freq) in iter.into_iter() { self.insert_or_incr_by(k, freq); }
  }
}

impl<K, S, H> Distribution<K, H> for FrequencyDistribution<K, S> 
  where K: Eq + Hash, 
        S: HashState<Hasher=H>, 
        H: Hasher 
{
  type Quantity = usize;

  /// Returns the number of entries in the distribution
  #[inline]
  #[stable]
  fn len(&self) -> usize {
    self.hashmap.len()
  }

  /// Gets the frequency in which the key occurs.
  #[inline]
  #[stable]
  fn get<Q: ?Sized>(&self, k: &Q) -> usize 
    where K: Borrow<Q>, Q: Hash + Eq
  {
    self[k]
  }

  /// Clears the counts of all keys and clears all keys from 
  /// the distribution.
  #[inline]
  #[stable]
  fn clear(&mut self) {
    self.hashmap.clear()
  }

  /// Updates the frequency of the value found with the key if it 
  /// already exists. Otherwise, inserts the key sizeo the hashmap, 
  /// and sets its frequency to 1.
  #[inline]
  #[stable]
  fn insert(&mut self, k: K) {
    self.insert_or_incr_by(k, 1);
  }

  /// Removes a Key and its associated value from the Distrbution.
  #[inline]
  #[stable]
  fn remove<Q: ?Sized>(&mut self, k: &Q) 
    where K: Borrow<Q>, Q: Hash + Eq
  {
    match self.hashmap.remove(k) {
      Some(count) => self.sum_counts -= count,
      None        => ()
    }
  }
}

impl<K, S, H> IntoIterator for FrequencyDistribution<K, S>
  where K: Eq + Hash,
        S: HashState<Hasher=H>,
        H: Hasher
{
  type Item = (K, usize);
  type IntoIter = IntoIter<K, usize>;

  /// Consumes the distribution, and creates an iterator over the 
  /// (Key, Quantity: usize) pairs.
  #[inline]
  #[stable]
  fn into_iter(self) -> IntoIter<K, usize> {
    self.hashmap.into_iter()
  }
}

impl<'a, K, S, H, Q: ?Sized> Index<&'a Q> for FrequencyDistribution<K, S>
  where K: Eq + Hash + Borrow<Q>,
        S: HashState<Hasher=H>,
        H: Hasher,
        Q: Eq + Hash
{
  type Output = usize;


  fn index<'b>(&'b self, index: &Q) -> &'b usize {
    self.hashmap.get(index).unwrap_or(ZERO)
  }
}

/// Iterator over entries with non-zero quantities.
#[stable]
pub struct NonZeroKeysIter<'a, K: 'a> {
  iter: Iter<'a, K, usize> 
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
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
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

  for _ in (0..7us) { dist.insert(words[0]); }

  assert_eq!(dist.get(&words[0]), 8);
}

#[test]
fn smoke_test_frequency_distribution_iter() {
  let words = vec!(("a", 50us), ("b", 100us), ("c", 75us), ("d", 0us));
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
  let words = vec!(("a", 50us), ("b", 100us), ("c", 25us));
  let mut dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

  assert_eq!(dist.get(&"a"), 50);

  dist.remove(&"a");

  assert_eq!(dist.get(&"a"), 0);
  assert_eq!(dist.sum_counts(), 125);
}

#[test]
fn smoke_test_frequency_sum_counts() {
  let words = vec!(("a", 7us), ("b", 5us), ("c", 8us), ("d", 3us));
  let mut dist: FrequencyDistribution<&str> = FromIterator::from_iter(words.into_iter());

  assert_eq!(dist.sum_counts(), 23);

  dist.insert("e");

  assert_eq!(dist.sum_counts(), 24);
}
