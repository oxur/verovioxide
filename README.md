# verovioxide

Safe Rust bindings to [Verovio](https://www.verovio.org/), the music notation engraving library.

## Vision

**verovioxide** aims to bring professional-quality music notation rendering to the Rust ecosystem. Verovio is a fast, lightweight C++ library that renders MusicXML, MEI, ABC, and Humdrum notation to beautiful SVG output — and now Rust developers can use it too.

This project is part of a larger effort to build modern, composable tools for working with music notation programmatically. We believe that:

- **Music notation should be accessible** — not locked inside proprietary software
- **Rendering should be fast and embeddable** — suitable for servers, CLI tools, and interactive applications
- **The Rust ecosystem deserves first-class music tooling**

## Status

🚧 **Early Development** — Early planning phases

We're actively building the foundation:

- [ ] `verovioxide-sys` — Raw FFI bindings to Verovio's C API
- [ ] `verovioxide-data` — Bundled SMuFL fonts (Leipzig, Bravura, etc.)
- [ ] `verovioxide` — Safe, idiomatic Rust API

## Planned Features

- 🎼 **Multi-format input** — MusicXML, MEI, ABC notation, Humdrum
- 🎨 **SVG output** — Clean, scalable vector graphics
- 📦 **Bundled fonts** — No external dependencies required
- 🔧 **Typed options API** — Configure rendering with Rust's type safety
- ⚡ **Static linking** — Single binary deployment

## Quick Preview

*API is subject to change*

```rust
use verovioxide::{Toolkit, Options};

fn main() -> verovioxide::Result<()> {
    let mut toolkit = Toolkit::new()?;

    let options = Options::new()
        .scale(40)
        .adjust_page_height(true);
    toolkit.set_options(&options)?;

    let musicxml = std::fs::read_to_string("score.musicxml")?;
    toolkit.load_data(&musicxml)?;

    for page in 1..=toolkit.page_count() {
        let svg = toolkit.render_to_svg(page)?;
        std::fs::write(format!("page-{}.svg", page), &svg)?;
    }

    Ok(())
}
```

## The Bigger Picture

**verovioxide** is one piece of a growing ecosystem for music notation in Rust:

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│    Fermata Lisp                                         │
│    (S-expression DSL for music notation)                │
│                         │                               │
│                         ▼                               │
│    ┌───────────────────────────────────────┐            │
│    │                MusicXML               │            │
│    └───────────────────────────────────────┘            │
│                         │                               │
│                         ▼                               │
│    ┌───────────────────────────────────────┐            │
│    │               verovioxide             │  ◄── you are here
│    │        (Rust bindings to Verovio)     │            │
│    └───────────────────────────────────────┘            │
│                         │                               │
│                         ▼                               │
│                        SVG                              │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

We're also developing [**Fermata**](https://github.com/oxur/fermata) — a Lisp-based DSL that compiles to MusicXML, making it easy to express musical ideas in code:

```lisp
;; Fermata syntax (coming soon)
(score
  (part :piano
    (measure
      (chord :q (c4 e4 g4))
      (chord :q (d4 f4 a4))
      (chord :h (g3 b3 d4 g4)))))
```

## Building

```bash
# Clone with submodules
git clone --recursive https://github.com/oxur/verovioxide.git
cd verovioxide

# Build
cargo build --release

# Run tests
cargo test

# Run example
cargo run --example render_musicxml -- path/to/score.musicxml output.svg
```

### Requirements

- Rust 1.75+ (we use edition 2024 features)
- CMake 3.14+
- C++20 compiler (clang 14+ or gcc 11+)

## License

This project is dual-licensed under MIT OR Apache-2.0, at your option.

**Note:** verovioxide links against [Verovio](https://github.com/rism-digital/verovio) (LGPL-2.1-or-later) and includes SMuFL fonts (SIL Open Font License 1.1). See [NOTICE](NOTICE) for full attribution.

## Acknowledgments

- [Verovio](https://www.verovio.org/) by RISM Digital — the incredible engraving library that makes this possible
- [SMuFL](https://www.smufl.org/) — Standard Music Font Layout specification
- The Rust community for inspiration and tooling

## Contributing

We welcome contributions! This project is in early stages, so there's plenty of opportunity to shape its direction.

- 🐛 **Bug reports** — Open an issue
- 💡 **Feature ideas** — Start a discussion
- 🔧 **Code contributions** — PRs welcome (please open an issue first for major changes)

---

*Part of the [oxur](https://github.com/oxur) organization*
