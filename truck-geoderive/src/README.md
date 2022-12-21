# truck-geoderive

Define some derive macros for implements geotraits.

The macros defined here are all called by `truck-geotrait`, so there is no need for the user to specify a priori any dependencies on this crate.
To use the macros used in this crate, activate the feature `derive` in `truck-geotrait`.

```toml
truck-geotrait = { version = "0.3.0", features = ["derive"] }
```
