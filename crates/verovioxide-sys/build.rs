//! Build script for verovioxide-sys.
//!
//! This script compiles the Verovio C++ library from source using the `cc` crate.
//! It handles platform-specific configuration and links the appropriate C++ standard library.
//!
//! # Smart Caching
//!
//! To avoid recompiling Verovio (~6 minutes) on every Rust code change, this script
//! implements smart caching:
//!
//! - The compiled library is cached at `target/verovio-cache/libverovio.a`
//! - Subsequent builds link to the cached library instead of recompiling
//! - Use `cargo build --features force-rebuild` to force a fresh compilation
//!
//! # Cache Location
//!
//! The cache is stored in the workspace's `target/verovio-cache/` directory to persist
//! across clean builds of individual crates while still being cleaned by `cargo clean`.

use std::io::Write;
use std::path::PathBuf;

/// Returns the path to the Verovio cache directory.
///
/// The cache is located at `<workspace_root>/target/verovio-cache/` to ensure it:
/// - Persists across incremental builds
/// - Is cleaned by `cargo clean`
/// - Is shared across all build configurations (debug/release)
fn get_cache_dir() -> PathBuf {
    // Use CARGO_MANIFEST_DIR to find the workspace root reliably.
    // This works regardless of the target directory structure (normal, llvm-cov, etc.)
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // Navigate up from crates/verovioxide-sys to workspace root
    let workspace_root = manifest_dir
        .parent() // verovioxide-sys -> crates
        .and_then(|p| p.parent()) // crates -> workspace root
        .expect("Failed to find workspace root from CARGO_MANIFEST_DIR");

    workspace_root.join("target").join("verovio-cache")
}

/// Returns the path to the cached static library.
fn get_cached_library_path() -> PathBuf {
    let cache_dir = get_cache_dir();
    if cfg!(target_os = "windows") && cfg!(target_env = "msvc") {
        cache_dir.join("verovio.lib")
    } else {
        cache_dir.join("libverovio.a")
    }
}

/// Checks if a cached Verovio library exists and should be used.
///
/// Returns `false` if:
/// - The `force-rebuild` feature is enabled
/// - The cached library file doesn't exist
fn should_use_cache() -> bool {
    // Check for force-rebuild feature via environment variable
    // (cfg! is compile-time, but we need runtime check in build scripts)
    if std::env::var("CARGO_FEATURE_FORCE_REBUILD").is_ok() {
        println!("cargo:warning=force-rebuild feature enabled, recompiling Verovio");
        return false;
    }

    let cached_lib = get_cached_library_path();
    if cached_lib.exists() {
        println!(
            "cargo:warning=Using cached Verovio library from {}",
            cached_lib.display()
        );
        true
    } else {
        println!(
            "cargo:warning=No cached Verovio library found at {}, compiling from source",
            cached_lib.display()
        );
        false
    }
}

/// Emits the linker directives to link against the Verovio library.
fn emit_link_directives(search_path: &std::path::Path) {
    println!("cargo:rustc-link-search=native={}", search_path.display());
    println!("cargo:rustc-link-lib=static=verovio");

    // Link the C++ standard library
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=c++");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if cfg!(target_os = "windows") {
        // MSVC links the C++ runtime automatically
        if cfg!(target_env = "gnu") {
            println!("cargo:rustc-link-lib=stdc++");
        }
    }
}

/// Copies the compiled library to the cache directory.
fn cache_compiled_library(out_dir: &std::path::Path) {
    let cache_dir = get_cache_dir();

    // Create cache directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        println!(
            "cargo:warning=Failed to create cache directory: {}. Caching disabled.",
            e
        );
        return;
    }

    // Determine the library filename based on platform
    let lib_name = if cfg!(target_os = "windows") && cfg!(target_env = "msvc") {
        "verovio.lib"
    } else {
        "libverovio.a"
    };

    let source = out_dir.join(lib_name);
    let dest = cache_dir.join(lib_name);

    if source.exists() {
        match std::fs::copy(&source, &dest) {
            Ok(_) => println!("cargo:warning=Cached Verovio library to {}", dest.display()),
            Err(e) => println!(
                "cargo:warning=Failed to cache library: {}. Future builds may recompile.",
                e
            ),
        }
    } else {
        println!(
            "cargo:warning=Compiled library not found at {}, caching skipped",
            source.display()
        );
    }
}

fn main() {
    // Only compile when the bundled feature is enabled
    // (use env var since cfg! is compile-time, not runtime in build scripts)
    if std::env::var("CARGO_FEATURE_BUNDLED").is_err() {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let verovio_dir = manifest_dir.join("../../verovio").canonicalize().unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Set up rerun-if-changed directives
    // These only affect when Cargo decides to re-run this build script,
    // not whether we use the cache
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rerun-if-changed={}",
        verovio_dir.join("tools/c_wrapper.cpp").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        verovio_dir.join("tools/c_wrapper.h").display()
    );

    // Check if we can use the cached library
    if should_use_cache() {
        let cache_dir = get_cache_dir();
        emit_link_directives(&cache_dir);
        return;
    }

    // --- Full compilation path ---

    // Generate git_commit.h if it doesn't exist
    let git_commit_h = verovio_dir.join("include/vrv/git_commit.h");
    if !git_commit_h.exists() {
        let mut file = std::fs::File::create(&git_commit_h).expect("Failed to create git_commit.h");
        writeln!(
            file,
            "////////////////////////////////////////////////////////"
        )
        .unwrap();
        writeln!(
            file,
            "/// Git commit version file generated at compilation ///"
        )
        .unwrap();
        writeln!(
            file,
            "////////////////////////////////////////////////////////"
        )
        .unwrap();
        writeln!(file).unwrap();
        writeln!(file, "#define GIT_COMMIT \"\"").unwrap();
        writeln!(file).unwrap();
    }

    let mut build = cc::Build::new();

    // Configure C++20 standard
    build.cpp(true).std("c++20");

    // Add include directories
    let include_dirs = [
        "include",
        "include/vrv",
        "include/crc",
        "include/midi",
        "include/hum",
        "include/json",
        "include/pugi",
        "include/zip",
        "libmei/dist",
        "libmei/addons",
    ];

    for dir in &include_dirs {
        build.include(verovio_dir.join(dir));
    }

    // Platform-specific include for Windows
    if cfg!(target_os = "windows") {
        build.include(verovio_dir.join("include/win32"));
    }

    // Add compiler definitions (matching CMakeLists.txt defaults)
    build.define("NO_DARMS_SUPPORT", None);
    build.define("NO_RUNTIME", None);

    // Set resource directory to a reasonable default
    build.define("RESOURCE_DIR", "\"/usr/local/share/verovio\"");

    // Compiler flags (matching CMakeLists.txt for non-MSVC builds)
    if !cfg!(target_env = "msvc") {
        build
            .flag("-Wall")
            .flag("-W")
            .flag("-pedantic")
            .flag("-Wno-unused-parameter")
            .flag("-Wno-dollar-in-identifier-extension")
            .flag("-Wno-conversion")
            .flag("-Wno-float-conversion")
            .flag("-Wno-missing-braces")
            .flag("-Wno-missing-field-initializers")
            .flag("-Wno-overloaded-virtual")
            .flag("-Wno-shadow")
            .flag("-Wno-sign-conversion")
            .flag("-Wno-trigraphs")
            .flag("-Wno-unknown-pragmas")
            .flag("-Wno-unused-label");
    } else {
        // MSVC-specific settings
        build.flag("/bigobj").flag("/W2").flag("/wd4244");
        build.define("NO_PAE_SUPPORT", None);
        build.define("USE_PAE_OLD_PARSER", None);
    }

    // Collect source files
    let mut sources: Vec<PathBuf> = Vec::new();

    // Main verovio sources (excluding main.cpp)
    for entry in std::fs::read_dir(verovio_dir.join("src")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp")
            && path.file_name().is_some_and(|name| name != "main.cpp")
        {
            sources.push(path);
        }
    }

    // Humdrum sources
    if let Ok(entries) = std::fs::read_dir(verovio_dir.join("src/hum")) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "cpp") {
                sources.push(path);
            }
        }
    }

    // MIDI sources
    for entry in std::fs::read_dir(verovio_dir.join("src/midi")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // CRC sources
    for entry in std::fs::read_dir(verovio_dir.join("src/crc")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // JSON source (note: .cc extension)
    sources.push(verovio_dir.join("src/json/jsonxx.cc"));

    // pugixml source
    sources.push(verovio_dir.join("src/pugi/pugixml.cpp"));

    // libmei dist sources
    for entry in std::fs::read_dir(verovio_dir.join("libmei/dist")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // libmei addons sources
    for entry in std::fs::read_dir(verovio_dir.join("libmei/addons")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // C wrapper
    sources.push(verovio_dir.join("tools/c_wrapper.cpp"));

    // Add all source files to the build
    for source in &sources {
        build.file(source);
    }

    // Compile the library
    build.compile("verovio");

    // Cache the compiled library for future builds
    cache_compiled_library(&out_dir);

    // Emit link directives (cc::Build::compile already sets up linking,
    // but we emit them explicitly for consistency)
    emit_link_directives(&out_dir);
}
