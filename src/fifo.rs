//! Implementation of an "adressable queue", that is a FIFO queue where it is
//! possible to directly remove values directly by a key.

// Copyright 2018 Leonardo Schwarz <mail@leoschwarz.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

struct Item<K, V> {
    pub key: K,
    pub val: Mutex<Option<V>>,
}

/// An addressable FIFO queue.
///
/// This data structure combines operations from a FIFO queue with the option to remove elements by
/// directly specifying their key, in an efficient manner.
pub struct Queue<K, V> {
    items: VecDeque<Arc<Item<K, V>>>,
    pointers: HashMap<K, Arc<Item<K, V>>>,
}

impl<K, V> Queue<K, V>
where
    K: Clone + Eq + Hash,
{
    /// Create a new instance of the queue.
    pub fn new() -> Self {
        Queue {
            items: VecDeque::new(),
            pointers: HashMap::new(),
        }
    }

    /// Insert an entry at the end of the queue.
    ///
    /// ```
    /// use addressable_queue::fifo::Queue;
    ///
    /// let mut queue = Queue::new();
    /// queue.insert(2u8, 4u8);
    /// queue.insert(3u8, 6u8);
    /// queue.insert(4u8, 8u8);
    ///
    /// assert_eq!(Some((2, 4)), queue.remove_head());
    /// assert_eq!(Some((3, 6)), queue.remove_head());
    /// assert_eq!(Some((4, 8)), queue.remove_head());
    /// assert_eq!(None, queue.remove_head());
    /// ```
    pub fn insert(&mut self, key: K, value: V) {
        let arc = Arc::new(Item {
            key: key.clone(),
            val: Mutex::new(Some(value)),
        });
        self.items.push_back(Arc::clone(&arc));
        self.pointers.insert(key, arc);
    }

    /// Insert an entry at the front of the queue.
    ///
    /// This is mostly useful when removing the head and
    /// then deciding to put it back into the queue.
    ///
    /// ```
    /// use addressable_queue::fifo::Queue;
    ///
    /// let mut queue = Queue::new();
    /// queue.insert_head(2u8, 4u8);
    /// queue.insert_head(3u8, 6u8);
    /// queue.insert_head(4u8, 8u8);
    ///
    /// assert_eq!(Some((4, 8)), queue.remove_head());
    /// assert_eq!(Some((3, 6)), queue.remove_head());
    /// assert_eq!(Some((2, 4)), queue.remove_head());
    /// assert_eq!(None, queue.remove_head());
    /// ```
    pub fn insert_head(&mut self, key: K, value: V) {
        let arc = Arc::new(Item {
            key: key.clone(),
            val: Mutex::new(Some(value)),
        });
        self.items.push_front(Arc::clone(&arc));
        self.pointers.insert(key, arc);
    }

    /// Remove the current head of the queue, and return the value if there was one.
    ///
    /// ```
    /// use addressable_queue::fifo::Queue;
    ///
    /// let mut queue = Queue::new();
    /// queue.insert(2u8, 4u8);
    /// queue.insert(3u8, 6u8);
    /// queue.insert(4u8, 8u8);
    ///
    /// assert_eq!(Some((2, 4)), queue.remove_head());
    /// assert_eq!(Some((3, 6)), queue.remove_head());
    /// assert_eq!(Some((4, 8)), queue.remove_head());
    /// assert_eq!(None, queue.remove_head());
    /// ```
    pub fn remove_head(&mut self) -> Option<(K, V)> {
        while let Some(item) = self.items.pop_front() {
            let is_some = item.val.lock().unwrap().is_some();
            if is_some {
                self.pointers.remove(&item.key);
                let key = item.key.clone();
                let value = Arc::try_unwrap(item)
                    .ok()
                    .unwrap()
                    .val
                    .into_inner()
                    .unwrap()
                    .unwrap();
                return Some((key, value));
            }
        }
        None
    }

    /// Remove the current tail of the queue, and return the value if there was one.
    ///
    /// ```
    /// use addressable_queue::fifo::Queue;
    ///
    /// let mut queue = Queue::new();
    /// queue.insert(2u8, 4u8);
    /// queue.insert(3u8, 6u8);
    /// queue.insert(4u8, 8u8);
    ///
    /// assert_eq!(Some((4, 8)), queue.remove_tail());
    /// assert_eq!(Some((3, 6)), queue.remove_tail());
    /// assert_eq!(Some((2, 4)), queue.remove_tail());
    /// assert_eq!(None, queue.remove_tail());
    /// ```
    pub fn remove_tail(&mut self) -> Option<(K, V)> {
        while let Some(item) = self.items.pop_back() {
            let is_some = item.val.lock().unwrap().is_some();
            if is_some {
                self.pointers.remove(&item.key);
                let key = item.key.clone();
                let value = Arc::try_unwrap(item)
                    .ok()
                    .unwrap()
                    .val
                    .into_inner()
                    .unwrap()
                    .unwrap();
                return Some((key, value));
            }
        }
        None
    }

    /// Remove a value by specifying its key.
    ///
    /// ```
    /// use addressable_queue::fifo::Queue;
    ///
    /// let mut queue = Queue::new();
    /// queue.insert(2u8, 4u8);
    /// queue.insert(3u8, 6u8);
    /// queue.insert(4u8, 8u8);
    ///
    /// assert_eq!(Some(4), queue.remove_key(&2));
    /// assert_eq!(Some(6), queue.remove_key(&3));
    /// assert_eq!(None, queue.remove_key(&3));
    /// assert_eq!(Some(8), queue.remove_key(&4));
    /// assert_eq!(None, queue.remove_head());
    /// ```
    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        if let Some(item) = self.pointers.remove(key) {
            let mut val = None;
            ::std::mem::swap(&mut val, &mut *item.val.lock().unwrap());
            return val;
        }
        None
    }

    /*
    /// Convert the queue into a vec, where the first element is the head (oldest element).
    pub fn into_vec(self) -> Vec<(K,V)> {
        let mut vec = Vec::new();
        while let Some(item) = self.remove_head() {
            vec.push(item);
        }
        vec
    }
    */
}
