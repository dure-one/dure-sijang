#!/usr/bin/env bash
set -euo pipefail

# Android Keystore Setup Script
# Generates release keystore for app signing and GitHub Actions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
KEYSTORE_DIR="${PROJECT_ROOT}/.upload"
KEYSTORE_FILE="${KEYSTORE_DIR}/release.keystore"
KEYSTORE_PROPS="${KEYSTORE_DIR}/keystore.properties"
JDK_DIR="${HOME}/.local/jdk-24.0.1"
KEYTOOL="${JDK_DIR}/bin/keytool"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*"
    exit 1
}

check_keytool() {
    if [[ -x "${KEYTOOL}" ]]; then
        info "Found keytool at ${KEYTOOL}"
        return 0
    fi

    # Check system keytool
    if command -v keytool &> /dev/null; then
        KEYTOOL="$(command -v keytool)"
        info "Found system keytool at ${KEYTOOL}"
        return 0
    fi

    return 1
}

download_jdk() {
    warn "keytool not found. Need to download JDK 24."

    # Detect platform
    local os_type="$(uname -s)"
    local arch="$(uname -m)"
    local jdk_url=""

    case "${os_type}" in
        Linux)
            if [[ "${arch}" == "x86_64" ]]; then
                jdk_url="https://download.java.net/java/GA/jdk24/7c6f13394a1eed323792e0c96e8738e4/36/GPL/openjdk-24_linux-x64_bin.tar.gz"
            elif [[ "${arch}" == "aarch64" ]]; then
                jdk_url="https://download.java.net/java/GA/jdk24/7c6f13394a1eed323792e0c96e8738e4/36/GPL/openjdk-24_linux-aarch64_bin.tar.gz"
            else
                error "Unsupported architecture: ${arch}"
            fi
            ;;
        Darwin)
            if [[ "${arch}" == "x86_64" ]]; then
                jdk_url="https://download.java.net/java/GA/jdk24/7c6f13394a1eed323792e0c96e8738e4/36/GPL/openjdk-24_macos-x64_bin.tar.gz"
            elif [[ "${arch}" == "arm64" ]]; then
                jdk_url="https://download.java.net/java/GA/jdk24/7c6f13394a1eed323792e0c96e8738e4/36/GPL/openjdk-24_macos-aarch64_bin.tar.gz"
            else
                error "Unsupported architecture: ${arch}"
            fi
            ;;
        OpenBSD)
            warn "OpenBSD detected. Please install JDK manually: pkg_add jdk"
            error "After installation, update KEYTOOL path in this script."
            ;;
        *)
            error "Unsupported OS: ${os_type}"
            ;;
    esac

    info "Downloading JDK 24 from ${jdk_url}"
    mkdir -p "${HOME}/.local"
    cd "${HOME}/.local"

    curl -L -o jdk-24.tar.gz "${jdk_url}"
    tar -xzf jdk-24.tar.gz
    rm jdk-24.tar.gz

    # Extract directory name (varies by platform)
    local jdk_extracted=$(find . -maxdepth 1 -type d -name "jdk-24*" | head -1)
    if [[ -z "${jdk_extracted}" ]]; then
        error "JDK extraction failed. Could not find extracted directory."
    fi

    # Rename to standard path
    if [[ "${jdk_extracted}" != "./jdk-24.0.1" ]]; then
        mv "${jdk_extracted}" jdk-24.0.1
    fi

    info "JDK 24 installed to ${JDK_DIR}"
    KEYTOOL="${JDK_DIR}/bin/keytool"
}

generate_keystore() {
    if [[ -f "${KEYSTORE_FILE}" ]]; then
        warn "Keystore already exists at ${KEYSTORE_FILE}"
        read -p "Overwrite? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "Keeping existing keystore."
            return 0
        fi
        rm -f "${KEYSTORE_FILE}"
    fi

    info "Generating new keystore at ${KEYSTORE_FILE}"

    # Prompt for passwords
    echo ""
    echo "Enter keystore password (will be used for both store and key):"
    read -s STORE_PASSWORD
    echo ""
    echo "Confirm password:"
    read -s STORE_PASSWORD_CONFIRM
    echo ""

    if [[ "${STORE_PASSWORD}" != "${STORE_PASSWORD_CONFIRM}" ]]; then
        error "Passwords do not match!"
    fi

    if [[ ${#STORE_PASSWORD} -lt 6 ]]; then
        error "Password must be at least 6 characters long!"
    fi

    mkdir -p "${KEYSTORE_DIR}"

    # Generate keystore
    "${KEYTOOL}" -genkeypair \
        -v \
        -keystore "${KEYSTORE_FILE}" \
        -alias upload \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -storepass "${STORE_PASSWORD}" \
        -keypass "${STORE_PASSWORD}" \
        -dname "CN=Dure Sijang, OU=Mobile, O=Dure, L=Seoul, ST=Seoul, C=KR"

    info "Keystore generated successfully!"

    # Verify keystore
    info "Verifying keystore..."
    "${KEYTOOL}" -list -v \
        -keystore "${KEYSTORE_FILE}" \
        -storepass "${STORE_PASSWORD}" \
        -alias upload \
        -keypass "${STORE_PASSWORD}"

    # Create keystore.properties
    info "Creating keystore.properties..."
    cat > "${KEYSTORE_PROPS}" <<EOF
storePassword=${STORE_PASSWORD}
keyPassword=${STORE_PASSWORD}
keyAlias=upload
storeFile=release.keystore
EOF

    info "keystore.properties created at ${KEYSTORE_PROPS}"

    # Export base64 for GitHub Actions
    info "Exporting base64 for GitHub Actions..."
    local base64_file="${KEYSTORE_DIR}/release.keystore.base64"
    base64 < "${KEYSTORE_FILE}" > "${base64_file}"

    info "Base64 keystore saved to ${base64_file}"

    # Show GitHub secrets instructions
    echo ""
    echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}GitHub Actions Setup Instructions${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Add the following secrets to your GitHub repository:"
    echo "  Settings > Secrets and variables > Actions > Repository secrets"
    echo ""
    echo -e "${YELLOW}Secret Name: KEYSTORE_BASE64${NC}"
    echo "  Value: (paste contents of ${base64_file})"
    echo ""
    echo -e "${YELLOW}Secret Name: STORE_PASSWORD${NC}"
    echo "  Value: (get from ${KEYSTORE_PROPS})"
    echo ""
    echo -e "${YELLOW}Secret Name: KEY_PASSWORD${NC}"
    echo "  Value: (get from ${KEYSTORE_PROPS})"
    echo ""
    echo -e "${YELLOW}Secret Name: KEY_ALIAS${NC}"
    echo "  Value: upload"
    echo ""
    echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
    echo ""

    warn "Add keystore files to .gitignore:"
    echo "  .upload/release.keystore"
    echo "  .upload/keystore.properties"
    echo "  .upload/release.keystore.base64"
}

main() {
    info "Android Keystore Setup Script"
    info "Project: ${PROJECT_ROOT}"

    # Step 1: Check for keytool
    if ! check_keytool; then
        download_jdk
    fi

    # Step 2: Generate keystore
    generate_keystore

    info "Setup complete!"
}

main "$@"
