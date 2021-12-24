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

use std::mem;

use cxx::memory::{UniquePtr, UniquePtrTarget};

/// Extensions to [`UniquePtr`].
pub trait UniquePtrExt<T> {
    // These are forward ports of: https://github.com/dtolnay/cxx/pull/892.

    /// Returns a pointer to the object owned by this `UniquePtr`, if any,
    /// otherwise returns a null pointer.
    fn as_ptr(&self) -> *const T;

    /// Returns a mutable pointer to the object owned by this UniquePtr
    /// if any, otherwise the null pointer.
    ///
    /// As with [std::unique_ptr\<T\>::get](https://en.cppreference.com/w/cpp/memory/unique_ptr/get),
    /// this doesn't require that you hold a mutable reference to the `UniquePtr`.
    /// This differs from Rust norms, so extra care should be taken in
    /// the way the pointer is used.
    fn as_mut_ptr(&self) -> *mut T;

    /// Upcasts this unique pointer.
    ///
    /// A unique pointer can be upcast to any type `U` for which `Inherit<U>`
    /// is implemented for `T`.
    fn upcast<U>(&self) -> &UniquePtr<U>
    where
        T: Inherit<U>,
        U: UniquePtrTarget;

    /// Mutably upcasts this unique ptr.
    ///
    /// A unique pointer can be upcast to any type `U` for which `Inherit<U>`
    /// is implemented for `T`.
    fn upcast_mut<U>(&mut self) -> &mut UniquePtr<U>
    where
        T: Inherit<U>,
        U: UniquePtrTarget;
}

impl<T> UniquePtrExt<T> for UniquePtr<T>
where
    T: UniquePtrTarget,
{
    fn as_ptr(&self) -> *const T {
        match self.as_ref() {
            Some(target) => target as *const T,
            None => std::ptr::null(),
        }
    }

    fn as_mut_ptr(&self) -> *mut T {
        self.as_ptr() as *mut T
    }

    fn upcast<U>(&self) -> &UniquePtr<U>
    where
        U: UniquePtrTarget,
    {
        unsafe { mem::transmute(self) }
    }

    fn upcast_mut<U>(&mut self) -> &mut UniquePtr<U>
    where
        U: UniquePtrTarget,
    {
        unsafe { mem::transmute(self) }
    }
}

/// A marker trait indicating that it safe to upcast values of type `T` to type
/// `U`.
///
/// # Safety
///
/// Implementing this trait is unsafe because it enables safely transmuting from
/// the implementor to `UniquePtr<T>` via [`UniquePtrExt::upcast`] and
/// [`UniquePtrExt::upcast_mut`]. You must be careful to only implement this
/// trait when that inheritance relationship exists in C++.
pub unsafe trait Inherit<T> {}
