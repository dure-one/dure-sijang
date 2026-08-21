# Dure-Sijang Flatpak

Build and test Dure-Sijang as a Flatpak package.

## Prerequisites

- Docker (for containerized builds)
- Flatpak and flatpak-builder (for local builds)
- Built Dure-Sijang binary in `../../artifacts/dure-sijang-x86_64-unknown-linux-musl/`

## Building with Docker

```bash
$ docker build -t dure-sijang-flatpak .
$ docker run -it -v /home/wj/work/dure-sijang:/home/builder/dure-sijang-source dure-sijang-flatpak bash
$ make all
```

## Building Locally

```bash
$ make all          # Build complete Flatpak package
$ make test         # Install and test the built Flatpak
$ make run          # Run the installed Flatpak
$ make uninstall    # Uninstall the Flatpak
$ make clean        # Clean all build artifacts
$ make help         # Show all available targets
```

## Notes

- The app ID is `app.dure.sijang`
- Requires `org.freedesktop.Platform` and `org.freedesktop.Sdk` version 24.08
- Binary must be built before creating the Flatpak package