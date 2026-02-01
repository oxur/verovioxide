//! Build script for verovioxide-sys.
//!
//! This script compiles the Verovio C++ library from source using the `cc` crate.
//! It handles platform-specific configuration and links the appropriate C++ standard library.
//!
//! # Source Discovery
//!
//! The build script looks for Verovio source code in the following order:
//!
//! 1. `VEROVIO_SOURCE_DIR` environment variable - for corporate/restricted networks
//! 2. Local submodule at `../../verovio` - for development workflows
//! 3. Cached download at `target/verovio-cache/verovio-source/` - for repeat builds
//! 4. Download from GitHub release - for first-time crates.io users
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

use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;

/// Verovio version to download from GitHub.
/// This must match the version in the project Makefile (VEROVIO_VERSION).
const VEROVIO_VERSION: &str = "5.7.0";

/// Expected SHA256 hash of the release tarball.
/// This ensures integrity of downloaded sources and guards against supply chain attacks.
///
/// To compute/verify this hash, run:
/// ```sh
/// curl -sL https://github.com/rism-digital/verovio/archive/refs/tags/version-5.7.0.tar.gz | shasum -a 256
/// ```
const VEROVIO_TARBALL_SHA256: &str =
    "bf7483504ddbf2d7ff59ae53b547e6347f89f82583559bf264d97b3624279d5e";

/// GitHub release tarball URL.
fn get_download_url() -> String {
    format!(
        "https://github.com/rism-digital/verovio/archive/refs/tags/version-{}.tar.gz",
        VEROVIO_VERSION
    )
}

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

/// Returns the path to the cached Verovio source directory.
fn get_cached_source_dir() -> PathBuf {
    get_cache_dir().join("verovio-source")
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

/// Result of SHA256 verification.
enum HashVerification {
    /// Hash matches expected value.
    Match,
    /// Hash does not match; includes the actual hash for error reporting.
    Mismatch { actual: String },
}

/// Verifies the SHA256 hash of a file matches the expected value.
///
/// Returns `HashVerification::Match` if the hash matches, or `HashVerification::Mismatch`
/// with the actual hash if it doesn't (useful for error messages).
fn verify_sha256(path: &std::path::Path, expected_hash: &str) -> Result<HashVerification, String> {
    let data =
        std::fs::read(path).map_err(|e| format!("Failed to read file for hashing: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let actual_hash = format!("{:x}", result);

    if actual_hash == expected_hash {
        Ok(HashVerification::Match)
    } else {
        Ok(HashVerification::Mismatch { actual: actual_hash })
    }
}

/// Downloads a file from a URL to a destination path.
fn download_file(url: &str, dest: &std::path::Path) -> Result<(), String> {
    println!("cargo:warning=Downloading Verovio source from: {}", url);
    println!(
        "cargo:warning=This may take a moment on first build..."
    );

    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("Failed to download Verovio source: {}", e))?;

    if response.status() != 200 {
        return Err(format!(
            "HTTP error downloading Verovio source: status {}",
            response.status()
        ));
    }

    let mut reader = response.into_reader();
    let mut data = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut data)
        .map_err(|e| format!("Failed to read download response: {}", e))?;

    std::fs::write(dest, &data).map_err(|e| format!("Failed to write downloaded file: {}", e))?;

    Ok(())
}

/// Extracts a gzipped tarball to a destination directory.
fn extract_tarball(tarball_path: &std::path::Path, dest_dir: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::open(tarball_path)
        .map_err(|e| format!("Failed to open tarball: {}", e))?;

    let gz_decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz_decoder);

    archive
        .unpack(dest_dir)
        .map_err(|e| format!("Failed to extract tarball: {}", e))?;

    Ok(())
}

/// Discovers the Verovio source directory using the priority order.
///
/// Returns the path to the Verovio source directory, or an error message.
fn discover_verovio_source() -> Result<PathBuf, String> {
    // Priority 1: VEROVIO_SOURCE_DIR environment variable
    if let Ok(env_path) = std::env::var("VEROVIO_SOURCE_DIR") {
        let path = PathBuf::from(&env_path);
        if path.exists() && path.join("src").exists() {
            println!(
                "cargo:warning=Using Verovio source from VEROVIO_SOURCE_DIR: {}",
                path.display()
            );
            return Ok(path);
        } else {
            return Err(format!(
                "VEROVIO_SOURCE_DIR is set to '{}' but it doesn't appear to be a valid Verovio source directory. \
                 Expected to find a 'src' subdirectory.",
                env_path
            ));
        }
    }

    // Priority 2: Local submodule path
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let submodule_path = manifest_dir.join("../../verovio");
    if let Ok(canonical) = submodule_path.canonicalize() {
        if canonical.join("src").exists() {
            println!(
                "cargo:warning=Using Verovio source from local submodule: {}",
                canonical.display()
            );
            return Ok(canonical);
        }
    }

    // Priority 3: Cached download
    let cache_dir = get_cache_dir();
    let source_cache_dir = get_cached_source_dir();
    let extracted_dir = source_cache_dir.join(format!("verovio-version-{}", VEROVIO_VERSION));

    if extracted_dir.exists() && extracted_dir.join("src").exists() {
        println!(
            "cargo:warning=Using cached Verovio source from: {}",
            extracted_dir.display()
        );
        return Ok(extracted_dir);
    }

    // Priority 4: Download from GitHub
    println!("cargo:warning=Verovio source not found locally, downloading from GitHub...");

    // Create cache directories
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    std::fs::create_dir_all(&source_cache_dir)
        .map_err(|e| format!("Failed to create source cache directory: {}", e))?;

    let tarball_path = source_cache_dir.join(format!("verovio-{}.tar.gz", VEROVIO_VERSION));
    let url = get_download_url();

    // Download the tarball
    download_file(&url, &tarball_path)?;

    // Verify the SHA256 hash
    println!("cargo:warning=Verifying download integrity...");
    match verify_sha256(&tarball_path, VEROVIO_TARBALL_SHA256) {
        Ok(HashVerification::Match) => {
            println!("cargo:warning=SHA256 hash verified successfully");
        }
        Ok(HashVerification::Mismatch { actual }) => {
            // Remove the file that failed verification
            let _ = std::fs::remove_file(&tarball_path);
            return Err(format!(
                "SHA256 hash mismatch for downloaded Verovio source.\n\n\
                 Expected: {}\n\
                 Actual:   {}\n\n\
                 This could indicate:\n\
                 1. A corrupted download - try again\n\
                 2. The VEROVIO_TARBALL_SHA256 constant needs updating for version {}\n\
                 3. A supply chain attack (unlikely but verify manually)\n\n\
                 To update the hash, run:\n\
                 curl -sL {} | shasum -a 256\n\n\
                 Or set VEROVIO_SOURCE_DIR to use a local copy.",
                VEROVIO_TARBALL_SHA256,
                actual,
                VEROVIO_VERSION,
                get_download_url()
            ));
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tarball_path);
            return Err(format!("Failed to verify download hash: {}", e));
        }
    }

    // Extract the tarball
    println!("cargo:warning=Extracting Verovio source...");
    extract_tarball(&tarball_path, &source_cache_dir)?;

    // Verify extraction succeeded
    if extracted_dir.exists() && extracted_dir.join("src").exists() {
        println!(
            "cargo:warning=Verovio source extracted to: {}",
            extracted_dir.display()
        );
        // Clean up the tarball to save space
        let _ = std::fs::remove_file(&tarball_path);
        Ok(extracted_dir)
    } else {
        Err(format!(
            "Extraction completed but expected directory not found: {}\n\
             Please report this issue or set VEROVIO_SOURCE_DIR to use a local copy.",
            extracted_dir.display()
        ))
    }
}

fn main() {
    // Only compile when the bundled feature is enabled
    // (use env var since cfg! is compile-time, not runtime in build scripts)
    if std::env::var("CARGO_FEATURE_BUNDLED").is_err() {
        return;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Set up rerun-if-changed directives
    // These only affect when Cargo decides to re-run this build script,
    // not whether we use the cache
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=VEROVIO_SOURCE_DIR");

    // Check if we can use the cached library
    if should_use_cache() {
        let cache_dir = get_cache_dir();
        emit_link_directives(&cache_dir);
        return;
    }

    // --- Full compilation path ---

    // Discover Verovio source directory
    let verovio_dir = match discover_verovio_source() {
        Ok(path) => path,
        Err(e) => {
            panic!(
                "\n\nFailed to locate Verovio source:\n\n{}\n\n\
                 To resolve this, you can:\n\
                 1. Set VEROVIO_SOURCE_DIR environment variable to your local Verovio source\n\
                 2. Initialize the git submodule: git submodule update --init\n\
                 3. Ensure you have network access to download from GitHub\n\n",
                e
            );
        }
    };

    // Set up rerun-if-changed for source files now that we have the path
    println!(
        "cargo:rerun-if-changed={}",
        verovio_dir.join("tools/c_wrapper.cpp").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        verovio_dir.join("tools/c_wrapper.h").display()
    );

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
