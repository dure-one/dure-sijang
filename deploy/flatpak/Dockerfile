# Container to test Flatpak builds for Dure-Sijang
# Based on https://github.com/vrutkovs/flatpak-bat
FROM fedora:39

LABEL maintainer="Dure-Sijang Development Team"
LABEL description="Container for building and testing Dure-Sijang Flatpak"

# Install Flatpak and dependencies
RUN dnf update -y && \
    dnf install flatpak flatpak-builder wget git bzip2 elfutils make ostree \
        gcc gcc-c++ pkg-config openssl-devel -y && \
    dnf clean all

# Create a build user
RUN useradd -m builder
USER builder
WORKDIR /home/builder

# Set up Flatpak for the user
RUN flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

COPY --chown=builder:builder . /home/builder/dure-sijang-flatpak/

WORKDIR /home/builder/dure-sijang-flatpak