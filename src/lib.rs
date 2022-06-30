use num_traits::Zero;
use std::borrow::Borrow;
use std::collections::{hash_map::RandomState, HashMap};
use std::hash::{BuildHasher, Hash};
use std::ops::{Add, AddAssign, Index};

#[derive(Clone, Debug, Default)]
pub struct HashMultiset<T, C = usize, S = RandomState> {
    items: HashMap<T, C, S>,
}

impl<T, C, S> PartialEq for HashMultiset<T, C, S>
where
    HashMap<T, C, S>: PartialEq,
{
    fn eq(&self, other: &HashMultiset<T, C, S>) -> bool {
        self.items.eq(&other.items)
    }
}

impl<T, C, S> Eq for HashMultiset<T, C, S> where HashMap<T, C, S>: Eq {}

impl<T, C, S, Q: ?Sized> Index<&'_ Q> for HashMultiset<T, C, S>
where
    T: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
    S: BuildHasher,
{
    type Output = C;
    fn index(&self, item: &Q) -> &C {
        self.items.index(item)
    }
}

impl<T, C> HashMultiset<T, C> {
    pub fn new() -> HashMultiset<T, C> {
        HashMultiset {
            items: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> HashMultiset<T, C> {
        HashMultiset {
            items: HashMap::with_capacity(capacity),
        }
    }
}

impl<T, C, S> HashMultiset<T, C, S> {
    pub fn with_hasher(hash_builder: S) -> HashMultiset<T, C, S> {
        HashMultiset {
            items: HashMap::with_hasher(hash_builder),
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMultiset<T, C, S> {
        HashMultiset {
            items: HashMap::with_capacity_and_hasher(capacity, hash_builder),
        }
    }

    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    pub fn clear(&mut self) {
        self.items.clear()
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn(&T, &C) -> bool,
        //        ^^ purposely constraint `&C` instead of `&mut C` so that
        //           `f` cannot modify the count
    {
        // self.items.retain(|t: &T, c: &mut C| f(t, &*c))
        self.items.retain(|t, c| f(t, c)) // implicit cast `c as &C`
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, &C)> {
        self.items.iter()
    }

    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.items.keys()
    }

    pub fn into_items(self) -> impl Iterator<Item = T> {
        self.items.into_keys()
    }

    pub fn counts(&self) -> impl Iterator<Item = &C> {
        self.items.values()
    }

    pub fn into_counts(self) -> impl Iterator<Item = C> {
        self.items.into_values()
    }

    // drain
    // to_set
}

impl<T, C, S> HashMultiset<T, C, S>
where
    C: for<'a> std::iter::Sum<&'a C>,
{
    pub fn cardinality(&self) -> C {
        self.items.values().sum()
    }
}

impl<T, C, S> HashMultiset<T, C, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    // reserve
    // try_reserve
    // shrink_to_fit
    // shrink_to

    pub fn get<Q: ?Sized>(&self, item: &Q) -> Option<&C>
    where
        T: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.items.get(item)
    }

    pub fn get_key_value<Q: ?Sized>(&self, item: &Q) -> Option<(&T, &C)>
    where
        T: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.items.get_key_value(item)
    }

    pub fn into_set(self) -> std::collections::HashSet<T, S>
    where
        S: Default,
    {
        self.into_items().collect()
    }
}

impl<T, C, S> HashMultiset<T, C, S>
where
    T: Eq + Hash,
    S: BuildHasher,
    C: Copy + Zero,
{
    pub fn multiplicity<Q: ?Sized>(&self, item: &Q) -> C
    where
        T: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.items.get(item).map(|c| *c).unwrap_or(C::zero())
    }
}

impl<T, C, S> HashMultiset<T, C, S>
where
    T: Eq + Hash,
    S: BuildHasher,
    C: Add<C, Output = C> + AddAssign,
{
    pub fn insert(&mut self, item: T, multiplicity: C) -> &C {
        // self.items
        //     .entry(item)
        //     .and_modify(|c| *c += multiplicity) // `multiplicity` is moved here
        //     .or_insert(multiplicity)            // invalid second use of `multiplicity`

        match self.items.entry(item) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                *entry.get_mut() += multiplicity;
                entry.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(entry) => entry.insert(multiplicity),
        }
    }

    pub fn remove_all<Q: ?Sized>(&mut self, item: &Q) -> Option<C>
    where
        T: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.items.remove(item)
    }
}

// impl<T, C, S> HashMultiset<T, C, S>
// where
//     T: Eq + Hash,
//     S: BuildHasher,
//     C: num_traits::Zero,
// {
//     pub fn outer_join<'a>(
//         &'a self,
//         other: &'a HashMultiset<T, C, S>,
//     ) -> impl Iterator<Item = (&'a T, C, C)> {
//         self.items
//             .iter()
//             .map(move |(x, &self_multiplicity)| (x, self_multiplicity, other.multiplicity(x)))
//             .chain(other.items.iter().filter_map(move |(x, &other_multiplicity)| {
//                 let self_multiplicity = self.multiplicity(x);
//                 if self_multiplicity.is_zero() {
//                     Some((x, self_multiplicity, other_multiplicity))
//                 } else {
//                     None
//                 }
//             }))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct_empty_and_debug() {
        let mset = HashMultiset::<i32>::new();
        println!("{mset:?}");
    }

    #[test]
    fn can_insert_stuff() {
        let mut mset = HashMultiset::<i32, usize>::new();
        mset.insert(1, 1);
        mset.insert(2, 42);
        mset.insert(1, 3);
        assert_eq!(mset.get(&0), None);
        assert_eq!(mset.get(&1), Some(&4));
        assert_eq!(mset.get(&2), Some(&42));
        println!("{mset:?}");
    }

    #[test]
    fn can_remove_stuff() {
        let mut mset = HashMultiset::<i32, isize>::new();
        mset.insert(1, 1);
        mset.insert(2, 42);
        mset.insert(1, 3);
        mset.remove_all(&1);
        assert_eq!(mset.get(&0), None);
        assert_eq!(mset.get(&1), None);
        assert_eq!(mset.get(&2), Some(&42));
        println!("{mset:?}");
    }

    #[test]
    fn len_and_cardinality() {
        let mut mset = HashMultiset::<i32, usize>::new();
        mset.insert(1, 1);
        mset.insert(2, 42);
        mset.insert(1, 3);
        assert_eq!(mset.len(), 2);
        assert_eq!(mset.cardinality(), 46);
        println!("{mset:?}");
    }
}
