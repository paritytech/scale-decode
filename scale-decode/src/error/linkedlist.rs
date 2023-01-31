// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-value crate.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//         http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

// A simple singly-linked `Send`-able linked list
// implementation to allow fairly cheap path cloning
// and appending. This is used as part of our `Context`
// type and is not a part of the public API.
#[derive(Debug, Clone)]
pub struct LinkedList<T>(Option<Arc<Inner<T>>>);

#[derive(Debug)]
struct Inner<T> {
    item: T,
    prev: LinkedList<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self(None)
    }
    pub fn push(self, item: T) -> Self {
        Self(Some(Arc::new(Inner { item, prev: self })))
    }
    pub fn len(&self) -> usize {
        self.iter_back().count()
    }
    pub fn iter_back(&self) -> LinkedListIter<'_, T> {
        LinkedListIter { list: self }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LinkedListIter<'a, T> {
    list: &'a LinkedList<T>,
}

impl<'a, T> Iterator for LinkedListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        match &self.list.0 {
            None => None,
            Some(list) => {
                let item = &list.item;
                self.list = &list.prev;
                Some(item)
            }
        }
    }
}
