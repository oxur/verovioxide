---
number: 3
title: "Unified Query API with Trait-based `get()` Method"
author: "Duncan McGreggor"
component: All
tags: [change-me]
created: 2026-02-02
updated: 2026-02-02
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Unified Query API with Trait-based `get()` Method

## Overview

Add a unified `get()` method that mirrors the `render()` pattern, providing type-safe, consistent access to element queries and document information. Keep legacy `get_*` methods as aliases for backwards compatibility.

## API Design

### Before (Current API)

```rust
// MEI export - awkward dual methods
let mei = voxide.get_mei()?;
let mei = voxide.get_mei_with_options(r#"{"removeIds": true}"#)?;

// Element queries - verbose and inconsistent naming
let page = voxide.get_page_with_element("note-001")?;
let attrs = voxide.get_element_attr("note-001")?;
let time = voxide.get_time_for_element("note-001")?;
let times = voxide.get_times_for_element("note-001")?;
let expansion = voxide.get_expansion_ids_for_element("note-001")?;
let midi = voxide.get_midi_values_for_element("note-001")?;
let notated = voxide.get_notated_id_for_element("note-001")?;

// Time-based query
let elements = voxide.get_elements_at_time(5000)?;

// Descriptive features
let features = voxide.get_descriptive_features(Some("{}"))?;
```

### After (Unified `get()` API)

```rust
use verovioxide::{Page, Attrs, Time, Times, Elements, MidiValues, ExpansionIds, NotatedId, Features};

// MEI - use render() for export, get_mei() kept as legacy alias
let mei = voxide.render(Mei)?;
let mei = voxide.render(Mei::with_options().remove_ids(true))?;
let mei = voxide.get_mei()?;  // Legacy alias, calls render(Mei) internally

// Element queries via get() with query types
let page: u32 = voxide.get(Page::of("note-001"))?;
let attrs: String = voxide.get(Attrs::of("note-001"))?;
let time: f64 = voxide.get(Time::of("note-001"))?;
let times: String = voxide.get(Times::of("note-001"))?;
let expansion: String = voxide.get(ExpansionIds::of("note-001"))?;
let midi: String = voxide.get(MidiValues::of("note-001"))?;
let notated: String = voxide.get(NotatedId::of("note-001"))?;

// Time-based query
let elements: String = voxide.get(Elements::at(5000))?;

// Descriptive features
let features: String = voxide.get(Features)?;
let features: String = voxide.get(Features::with_options().option_name(true))?;
```

## Type Definitions

### Query Types

```rust
// Element-based queries (take xml_id)
pub struct Page<'a> { xml_id: &'a str }
pub struct Attrs<'a> { xml_id: &'a str }
pub struct Time<'a> { xml_id: &'a str }
pub struct Times<'a> { xml_id: &'a str }
pub struct ExpansionIds<'a> { xml_id: &'a str }
pub struct MidiValues<'a> { xml_id: &'a str }
pub struct NotatedId<'a> { xml_id: &'a str }

impl<'a> Page<'a> {
    pub fn of(xml_id: &'a str) -> Self { Self { xml_id } }
}
// Similar for Attrs, Time, Times, ExpansionIds, MidiValues, NotatedId

// Time-based query
pub struct Elements { millisec: i32 }

impl Elements {
    pub fn at(millisec: i32) -> Self { Self { millisec } }
}

// Descriptive features with optional options
pub struct Features;
pub struct FeaturesWithOptions { options: String }

impl Features {
    pub fn with_options() -> FeaturesOptionsBuilder { ... }
}

pub struct FeaturesOptionsBuilder { ... }
impl FeaturesOptionsBuilder {
    // Add specific option methods as needed
    pub fn build(self) -> FeaturesWithOptions { ... }
}
```

### Trait

```rust
/// Trait for queries with type-safe output
pub trait QueryOutput {
    type Output;
    fn query(self, toolkit: &Toolkit) -> Result<Self::Output>;
}

// Implementations with specific return types
impl<'a> QueryOutput for Page<'a> {
    type Output = u32;
    fn query(self, toolkit: &Toolkit) -> Result<u32> {
        toolkit.get_page_with_element(self.xml_id)
    }
}

impl<'a> QueryOutput for Attrs<'a> {
    type Output = String;  // JSON
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_element_attr(self.xml_id)
    }
}

impl<'a> QueryOutput for Time<'a> {
    type Output = f64;  // milliseconds
    fn query(self, toolkit: &Toolkit) -> Result<f64> {
        toolkit.get_time_for_element(self.xml_id)
    }
}

impl<'a> QueryOutput for Times<'a> {
    type Output = String;  // JSON array
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_times_for_element(self.xml_id)
    }
}

impl<'a> QueryOutput for ExpansionIds<'a> {
    type Output = String;  // JSON
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_expansion_ids_for_element(self.xml_id)
    }
}

impl<'a> QueryOutput for MidiValues<'a> {
    type Output = String;  // JSON
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_midi_values_for_element(self.xml_id)
    }
}

impl<'a> QueryOutput for NotatedId<'a> {
    type Output = String;
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_notated_id_for_element(self.xml_id)
    }
}

impl QueryOutput for Elements {
    type Output = String;  // JSON
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_elements_at_time(self.millisec)
    }
}

impl QueryOutput for Features {
    type Output = String;  // JSON
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_descriptive_features(None)
    }
}

impl QueryOutput for FeaturesWithOptions {
    type Output = String;  // JSON
    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_descriptive_features(Some(&self.options))
    }
}
```

### Return Type Summary

| Query Type | Return Type | Description |
|------------|-------------|-------------|
| `Page::of(id)` | `u32` | Page number containing element |
| `Attrs::of(id)` | `String` | JSON attributes |
| `Time::of(id)` | `f64` | Time in milliseconds |
| `Times::of(id)` | `String` | JSON array of times |
| `ExpansionIds::of(id)` | `String` | JSON expansion IDs |
| `MidiValues::of(id)` | `String` | JSON MIDI values |
| `NotatedId::of(id)` | `String` | Notated element ID |
| `Elements::at(ms)` | `String` | JSON elements at time |
| `Features` | `String` | JSON descriptive features |

## Files to Modify

| File | Changes |
|------|---------|
| `crates/verovioxide/src/query.rs` | **New file**: Query types, builders, QueryOutput trait |
| `crates/verovioxide/src/toolkit.rs` | Add `get()` method, update `get_mei()` as alias |
| `crates/verovioxide/src/lib.rs` | Add `mod query`, export query types |
| `crates/verovioxide/tests/integration_test.rs` | Add query API tests |
| `README.md` | Update element query examples |

## Implementation Steps

### Step 1: Create query.rs with query types

- Element query types: `Page`, `Attrs`, `Time`, `Times`, `ExpansionIds`, `MidiValues`, `NotatedId`
- Time-based query: `Elements`
- Features query: `Features`, `FeaturesOptionsBuilder`, `FeaturesWithOptions`

### Step 2: Implement QueryOutput trait

- Define trait with associated `Output` type
- Implement for each query type with correct return type:
  - `Page` → `u32`
  - `Attrs` → `String` (JSON)
  - `Time` → `f64`
  - `Times` → `String` (JSON)
  - `ExpansionIds` → `String` (JSON)
  - `MidiValues` → `String` (JSON)
  - `NotatedId` → `String`
  - `Elements` → `String` (JSON)
  - `Features` → `String` (JSON)
- Each impl calls appropriate existing toolkit method

### Step 3: Add Toolkit::get() method

```rust
impl Toolkit {
    /// Unified query with type-safe output
    pub fn get<Q: QueryOutput>(&self, query: Q) -> Result<Q::Output> {
        query.query(self)
    }
}
```

### Step 4: Update get_mei() as legacy alias

```rust
impl Toolkit {
    /// Legacy alias for render(Mei)
    pub fn get_mei(&self) -> Result<String> {
        self.render(Mei)
    }

    /// Legacy alias for render(Mei::with_options()...)
    pub fn get_mei_with_options(&self, options: &str) -> Result<String> {
        // Keep existing implementation for raw JSON options
        // Or parse and delegate to render() with typed options
    }
}
```

### Step 5: Add tests

- Test each query type's `of()` / `at()` constructor
- Test `get()` with each query type
- Test return types are correct
- Test error cases (element not found, etc.)

### Step 6: Update lib.rs exports

```rust
pub use query::{
    Page, Attrs, Time, Times, ExpansionIds, MidiValues, NotatedId,
    Elements, Features, FeaturesOptionsBuilder, FeaturesWithOptions,
    QueryOutput,
};
```

### Step 7: Update README

- Add section on unified query API
- Show before/after examples
- Document legacy method availability

## Legacy Methods (kept as aliases)

The following methods remain available for backwards compatibility:

- `get_mei()` → calls `render(Mei)`
- `get_mei_with_options()` → keeps existing implementation
- `get_page_with_element()` → called by `Page::of().query()`
- `get_element_attr()` → called by `Attrs::of().query()`
- `get_time_for_element()` → called by `Time::of().query()`
- `get_times_for_element()` → called by `Times::of().query()`
- `get_expansion_ids_for_element()` → called by `ExpansionIds::of().query()`
- `get_midi_values_for_element()` → called by `MidiValues::of().query()`
- `get_notated_id_for_element()` → called by `NotatedId::of().query()`
- `get_elements_at_time()` → called by `Elements::at().query()`
- `get_descriptive_features()` → called by `Features.query()`

## Verification

1. `cargo build` - Compilation
2. `cargo test` - All tests pass (existing + new)
3. `cargo clippy` - No warnings
4. Verify legacy methods still work unchanged
5. Test new `get()` API with all query types
