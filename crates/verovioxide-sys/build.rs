//! Build script for verovioxide-sys.
//!
//! This script compiles the Verovio C++ library from source using the `cc` crate.
//! It handles platform-specific configuration and links the appropriate C++ standard library.

use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Only compile when the bundled feature is enabled
    if !cfg!(feature = "bundled") {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let verovio_dir = manifest_dir.join("../../verovio").canonicalize().unwrap();

    println!("cargo:rerun-if-changed=build.rs");
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
        writeln!(file, "////////////////////////////////////////////////////////").unwrap();
        writeln!(file, "/// Git commit version file generated at compilation ///").unwrap();
        writeln!(file, "////////////////////////////////////////////////////////").unwrap();
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
