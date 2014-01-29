cmake_minimum_required(VERSION 2.8)
project(upnp NONE)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")
find_package(rustc)
find_package(rustdoc)
include(Rust)

set(RUSTC_FLAGS "-L${CMAKE_BINARY_DIR}/lib")
set(RUSTDOC_FLAGS "-L${CMAKE_BINARY_DIR}/lib")

# Get the dependencies of all the crates
get_rust_deps(src/lib.rs UPNP_DEPS OTHER_RUSTC_FLAGS --rlib)

# Build the library
rust_crate(src/lib.rs
    TARGET_NAME upnp
    DESTINATION lib
    DEPENDS "${UPNP_DEPS};sqlite3;xml"
    OTHER_RUSTC_FLAGS --rlib
    )

rust_crate(/deps/RustyXML/src/xml/lib.rs
    TARGET_NAME xml
    DESTINATION lib
    OTHER_RUSTC_FLAGS --rlib
    )

rust_crate(/deps/rustsqlite/src/sqlite3/lib.rs
    TARGET_NAME sqlite3
    DESTINATION lib
    OTHER_RUSTC_FLAGS --rlib
    )

rust_crate(src/test_server.rs
    TARGET_NAME test_server
    DESTINATION bin
    DEPENDS "upnp"
    )
