// Copyright Materialize, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository, or online at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::mem::{self, MaybeUninit};

pub fn unwrap_err<T, E>(res: Result<T, E>) -> E {
    match res {
        Ok(_) => panic!("Result was unexpectedly Ok"),
        Err(e) => e,
    }
}

/// Copies the elements from `src` to `this`, returning a mutable reference to
/// the now initialized contents of `this`.
///
/// This is a forward port of an unstable API.
/// See: https://github.com/rust-lang/rust/issues/79995
pub fn copy_to_uninit_slice<'a, T>(this: &'a mut [MaybeUninit<T>], src: &[T]) -> &'a mut [T]
where
    T: Copy,
{
    // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout.
    let uninit_src: &[MaybeUninit<T>] = unsafe { mem::transmute(src) };

    this.copy_from_slice(uninit_src);

    // SAFETY: Valid elements have just been copied into `this` so it is
    // initialized.
    unsafe { &mut *(this as *mut [MaybeUninit<T>] as *mut [T]) }
}
