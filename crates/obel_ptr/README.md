# Obel Ptr

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/ntakulatech/nita_obel#license)
[![Crates.io](https://img.shields.io/crates/v/obel_ptr.svg)](https://crates.io/crates/obel_ptr)
[![Downloads](https://img.shields.io/crates/d/obel_ptr.svg)](https://crates.io/crates/obel_ptr)
[![Docs](https://docs.rs/obel_ptr/badge.svg)](https://docs.rs/obel_ptr/latest/obel_ptr/)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/bevy)

## Overview

Pointers are fundamental building blocks in computer programming that store memory addresses. While powerful, they are also notorious for causing memory safety issues like null pointer dereferences, use-after-free bugs, and type safety violations.

Rust addresses these challenges through its type system, offering various pointer types with different safety guarantees. `obel_ptr` bridges the gap between raw pointers (`*mut ()`) and safe references (`&'a T`), allowing developers to choose specific safety invariants while building progressively safer abstractions.

## Building Safe Borrows

Creating valid and safe borrows from pointers requires satisfying several critical conditions:

- Proper pointer alignment
- Non-null pointer value
- Valid allocated object bounds
- Initialized target instance
- Valid lifetime for the target value
- Compliance with Rust's aliasing rules (one mutable or multiple immutable borrows)

## Standard Pointer Types

| Pointer Type        | Lifetime'ed | Mutable | Strongly Typed | Aligned | Not Null | Forbids Aliasing | Forbids Arithmetic |
| ------------------- | ----------- | ------- | -------------- | ------- | -------- | ---------------- | ------------------ |
| `Box<T>`            | Owned       | Yes     | Yes            | Yes     | Yes      | Yes              | Yes                |
| `&'a mut T`         | Yes         | Yes     | Yes            | Yes     | Yes      | Yes              | Yes                |
| `&'a T`             | Yes         | No      | Yes            | Yes     | Yes      | No               | Yes                |
| `&'a UnsafeCell<T>` | Yes         | Maybe   | Yes            | Yes     | Yes      | Yes              | Yes                |
| `NonNull<T>`        | No          | Yes     | Yes            | No      | Yes      | No               | No                 |
| `*const T`          | No          | No      | Yes            | No      | No       | No               | No                 |
| `*mut T`            | No          | Yes     | Yes            | No      | No       | No               | No                 |
| `*const ()`         | No          | No      | No             | No      | No       | No               | No                 |
| `*mut ()`           | No          | Yes     | No             | No      | No       | No               | No                 |

### Key Points

- `&T`, `&mut T`, and `Box<T>` are the safest, most commonly used pointer types
- `UnsafeCell<T>` enables interior mutability (used by `Cell`, `RefCell`, `Mutex`, etc.)
- `NonNull<T>` provides non-null guarantees without lifetime or alignment constraints
- Raw pointers (`*const T`, `*mut T`) offer maximum flexibility but minimal safety guarantees
- Untyped pointers (`*const ()`, `*mut ()`) are primarily used for FFI and special cases

## Nightly-Only Pointer Types

| Pointer Type | Lifetime'ed | Mutable | Strongly Typed | Aligned | Not Null | Forbids Aliasing | Forbids Arithmetic |
| ------------ | ----------- | ------- | -------------- | ------- | -------- | ---------------- | ------------------ |
| `Unique<T>`  | Owned       | Yes     | Yes            | Yes     | Yes      | Yes              | Yes                |
| `Shared<T>`  | Owned\*     | Yes     | Yes            | Yes     | Yes      | No               | Yes                |

- `Unique<T>`: Ownership-based pointer type used in `Box<T>`, `Vec<T>`, etc.
- `Shared<T>`: Reference-counting pointer type used in `Rc<T>` and `Arc<T>`

## obel_ptr Types

| Pointer Type          | Lifetime'ed | Mutable | Strongly Typed | Aligned | Not Null | Forbids Aliasing | Forbids Arithmetic |
| --------------------- | ----------- | ------- | -------------- | ------- | -------- | ---------------- | ------------------ |
| `ConstNonNull<T>`     | No          | No      | Yes            | No      | Yes      | No               | Yes                |
| `ThinSlicePtr<'a, T>` | Yes         | No      | Yes            | Yes     | Yes      | Yes              | Yes                |
| `OwningPtr<'a>`       | Yes         | Yes     | No             | Maybe   | Yes      | Yes              | No                 |
| `Ptr<'a>`             | Yes         | No      | No             | Maybe   | Yes      | No               | No                 |
| `PtrMut<'a>`          | Yes         | Yes     | No             | Maybe   | Yes      | Yes              | No                 |

### Features

- `ConstNonNull<T>`: Immutable variant of `NonNull<T>`
- `ThinSlicePtr<'a, T>`: Space-efficient slice pointer with debug-mode bounds checking
- `OwningPtr<'a>`, `Ptr<'a>`, `PtrMut<'a>`: Type-erased storage with progressive safety guarantees

These types enable efficient heterogeneous storage (e.g., ECS tables, typemaps) while maintaining safety through compile-time and debug-time checks.
