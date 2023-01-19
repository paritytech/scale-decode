// Copyright (C) 2022 Parity Technologies (UK) Ltd. (admin@parity.io)
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

/// A quick hopefully-stack-only vec implementation for holding TypeIds.
pub struct StackVec<T, const N: usize> {
    inner: StackVecInner<T, N>,
}

enum StackVecInner<T, const N: usize> {
    Stack { len: usize, items: [T; N] },
    Heap { items: Vec<T> },
}

impl<T: Default + Copy, const N: usize> StackVec<T, N> {
    pub fn new() -> Self {
        StackVec { inner: StackVecInner::Stack { len: 0, items: [Default::default(); N] } }
    }
    pub fn as_slice(&self) -> &[T] {
        match &self.inner {
            StackVecInner::Stack { len, items } => &items[0..*len],
            StackVecInner::Heap { items } => items,
        }
    }
    pub fn push(&mut self, item: T) {
        match &mut self.inner {
            StackVecInner::Heap { items } => items.push(item),
            StackVecInner::Stack { len, items } => {
                if *len == N {
                    let mut v = items[0..*len].to_vec();
                    v.push(item);
                    self.inner = StackVecInner::Heap { items: v };
                } else {
                    items[*len] = item;
                    *len += 1;
                }
            }
        }
    }
    #[cfg(test)]
    pub fn is_heap(&self) -> bool {
        match self.inner {
            StackVecInner::Stack { .. } => false,
            StackVecInner::Heap { .. } => true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn doesnt_overflow() {
        let mut s = StackVec::<_, 4>::new();

        assert_eq!(s.as_slice(), &[]);
        assert!(!s.is_heap());

        s.push(1);
        assert_eq!(s.as_slice(), &[1]);
        assert!(!s.is_heap());

        s.push(2);
        assert_eq!(s.as_slice(), &[1, 2]);
        assert!(!s.is_heap());

        s.push(3);
        assert_eq!(s.as_slice(), &[1, 2, 3]);
        assert!(!s.is_heap());

        s.push(4);
        assert_eq!(s.as_slice(), &[1, 2, 3, 4]);
        assert!(!s.is_heap());

        // Allocates on the heap after 4 items pushed..

        s.push(5);
        assert_eq!(s.as_slice(), &[1, 2, 3, 4, 5]);
        assert!(s.is_heap());

        s.push(6);
        assert_eq!(s.as_slice(), &[1, 2, 3, 4, 5, 6]);
        assert!(s.is_heap());
    }
}
