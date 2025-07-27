#!/bin/bash

# Script to update all package versions in Moosync repository
# This script should be run from the package directory

set -e
echo "Moosync Package Updater"
echo "======================="

# Get current version from Tauri config (not modifying this)
VERSION=$(grep -o '"version": "[^"]*"' ../src-tauri/tauri.conf.json | cut -d'"' -f4)
echo "Current Tauri version: $VERSION (will not be modified)"
echo "Downloading binaries and updating package files for version $VERSION"

# Path to Nix flake
NIX_FLAKE="nixos/flake.nix"

# Create a temporary directory for downloads
TEMP_DIR=$(mktemp -d)
echo "Using temporary directory: $TEMP_DIR"

# Function to check if directory exists and is a git repository
check_dir() {
    if [ ! -d "$1" ]; then
        echo "Directory $1 does not exist. Skipping..."
        return 1
    fi
    return 0
}

# Download required files and calculate checksums
echo "Downloading binaries and calculating checksums..."

# DEB file info
DEB_URL="https://github.com/Moosync/Moosync/releases/download/Moosync-v${VERSION}/Moosync_${VERSION}_amd64.deb"
DEB_FILE="$TEMP_DIR/Moosync_${VERSION}_amd64.deb"
DEB_CHECKSUM=""

# Download DEB file
echo "Downloading $DEB_URL"
if curl -L -s -o "$DEB_FILE" "$DEB_URL"; then
    DEB_CHECKSUM=$(sha256sum "$DEB_FILE" | cut -d' ' -f1)
    echo "✓ DEB file downloaded. Checksum: $DEB_CHECKSUM"
else
    echo "⚠️ Failed to download DEB file. Some package updates may fail."
    rm -f "$DEB_FILE"
fi

# Update Flatpak manifest
update_flatpak() {
    echo "Updating Flatpak package..."
    if check_dir "flatpak"; then
        if [ -z "$DEB_CHECKSUM" ]; then
            echo "⚠️ Cannot update Flatpak - DEB checksum not available"
            return 1
        fi

        # Update metainfo.xml
        sed -i "s/<release version=\"[0-9.]*\"/<release version=\"$VERSION\"/" flatpak/app.moosync.moosync/app.moosync.moosync.metainfo.xml
        # Update date in metainfo.xml
        current_date=$(date +%Y-%m-%d)
        sed -i "s/date=\"[0-9-]*\"/date=\"$current_date\"/" flatpak/app.moosync.moosync/app.moosync.moosync.metainfo.xml

        # Update version in yml file
        sed -i "s|Moosync-v[0-9.]\+/Moosync_[0-9.]\+|Moosync-v$VERSION/Moosync_$VERSION|g" flatpak/app.moosync.moosync/app.moosync.moosync.yml

        # Update checksum in yml file
        sed -i "s|sha256: [a-f0-9]\+|sha256: $DEB_CHECKSUM|g" flatpak/app.moosync.moosync/app.moosync.moosync.yml

        echo "✓ Flatpak package updated with new checksum: $DEB_CHECKSUM"
    fi
}

# Update Snap package
update_snap() {
    echo "Updating Snap package..."
    if check_dir "snap"; then
        if [ -f "snap/moosync/snapcraft.yaml" ]; then
            sed -i "s/version: .*/version: '$VERSION'/" snap/moosync/snapcraft.yaml
            echo "✓ Snap package updated"
        else
            echo "snapcraft.yaml not found"
        fi
    fi
}

# Update AUR packages
update_aur() {
    echo "Updating AUR packages..."
    if check_dir "aur"; then
        if [ -z "$DEB_CHECKSUM" ]; then
            echo "⚠️ Cannot update AUR packages - DEB checksum not available"
            return 1
        fi

        # Update PKGBUILD for stable version
        if [ -f "aur/moosync/PKGBUILD" ]; then
            # Update version
            sed -i "s/pkgver=.*/pkgver=$VERSION/" aur/moosync/PKGBUILD
            # Reset release number to 1 for new versions
            sed -i "s/pkgrel=.*/pkgrel=1/" aur/moosync/PKGBUILD

            # Update checksum in PKGBUILD using awk for proper multiline handling
            awk -v checksum="$DEB_CHECKSUM" '
                /^sha256sums=/ {
                    print "sha256sums=(\047" checksum "\047"
                    print "            \0474b63fa17717239db8a87ebeae1fdd96c5318b71d7d851d6c5a4f337793d3fecd\047)"
                    skip=1
                    next
                }
                /^[[:space:]]*\047.*\047\)/ {
                    if (skip) {
                        skip=0
                        next
                    }
                }
                { if (!skip) print }
            ' aur/moosync/PKGBUILD > aur/moosync/PKGBUILD.tmp && mv aur/moosync/PKGBUILD.tmp aur/moosync/PKGBUILD

            # Generate new .SRCINFO
            cd aur/moosync
            makepkg --printsrcinfo > .SRCINFO
            cd ../..

            echo "✓ AUR stable package updated with new checksum: $DEB_CHECKSUM"
        else
            echo "PKGBUILD not found for stable package"
        fi

        # Update git version if needed
        if [ -f "aur/moosync-git/PKGBUILD" ]; then
            echo "Note: moosync-git package uses latest git version, no version update needed"
        fi
    fi
}

# Update Fedora/RPM package
update_fedora() {
    echo "Updating Fedora/RPM package..."
    if check_dir "fedora"; then
        # Download RPM file directly to fedora/dnf directory
        RPM_URL="https://github.com/Moosync/Moosync/releases/download/Moosync-v${VERSION}/Moosync-${VERSION}-1.x86_64.rpm"
        RPM_FILE="fedora/dnf/Moosync-${VERSION}-1.x86_64.rpm"

        # Create dnf directory if it doesn't exist
        mkdir -p fedora/dnf

        echo "Downloading $RPM_URL to fedora/dnf directory..."
        if curl -L -s -o "$RPM_FILE" "$RPM_URL"; then
            echo "✓ RPM file downloaded successfully"

            # Update repository metadata
            echo "Updating repository metadata..."
            cd fedora/dnf
            createrepo_c --update .
            cd ../..

            echo "✓ Fedora repository updated"
        else
            echo "⚠️ Failed to download RPM file"
            rm -f "$RPM_FILE"
        fi
    fi
}

# Update Debian package using existing update.sh script
update_debian() {
    echo "Updating Debian package..."
    if check_dir "deb"; then
        if [ -f "deb/ppa/ubuntu/update.sh" ]; then
            echo "Running existing Debian PPA update script..."

            # Skip download if we already have the DEB file
            if [ -f "$DEB_FILE" ]; then
                # Copy to the ubuntu directory
                cp "$DEB_FILE" "deb/ppa/ubuntu/"
            fi

            # Save current directory
            CURRENT_DIR=$(pwd)

            # Change to the ubuntu directory and run the update script
            cd deb/ppa/ubuntu
            bash update.sh "$VERSION"

            # Return to original directory
            cd "$CURRENT_DIR"

            echo "✓ Debian PPA updated using existing script"
        else
            echo "Debian PPA update script not found"
        fi
    fi
}

# Update Nix flake with new version and checksum
update_nix_flake() {
    echo "Updating Nix flake..."
    if [ -f "$NIX_FLAKE" ]; then
        # Update version in flake.nix
        sed -i "s/version = \".*\";/version = \"$VERSION\";/" "$NIX_FLAKE"
        # Update sha256 for moosync deb package using the calculated checksum
        sed -i "s/sha256 = \"[a-f0-9]\{64\}\";/sha256 = \"$DEB_CHECKSUM\";/" "$NIX_FLAKE"
        echo "✓ Nix flake updated with version $VERSION and checksum $DEB_CHECKSUM"
    else
        echo "Nix flake not found at $NIX_FLAKE"
    fi
}

# Function to publish changes for each package type
publish_changes() {
    echo "Publishing changes..."

    # Publish AUR changes
    if check_dir "aur/moosync"; then
        echo "Publishing AUR changes..."
        cd aur/moosync
        if git diff --quiet && git diff --staged --quiet; then
            echo "No changes in AUR package"
        else
            git add PKGBUILD .SRCINFO
            git commit -m "Update to version $VERSION"
            git push upstream master
        fi
        cd ../..
        echo "✓ AUR changes checked"
    fi

    # Publish Flatpak changes
    if check_dir "flatpak/app.moosync.moosync"; then
        echo "Publishing Flatpak changes..."
        cd flatpak/app.moosync.moosync
        if git diff --quiet && git diff --staged --quiet; then
            echo "No changes in Flatpak package"
        else
            git add app.moosync.moosync.yml app.moosync.moosync.metainfo.xml
            git commit -m "Update to version $VERSION"
            git checkout -b "v$VERSION"
            git push origin "v$VERSION"
        fi
        cd ../../
        echo "✓ Flatpak changes checked"
    fi

    # Publish Debian changes
    if check_dir "deb/ppa"; then
        echo "Publishing Debian changes..."
        cd deb/ppa
        if git diff --quiet && git diff --staged --quiet; then
            echo "No changes in Debian package"
        else
            git add -A
            git commit -m "Update to version $VERSION"
            git push origin main
        fi
        cd ../../
        echo "✓ Debian changes checked"
    fi

    # Publish Snap changes
    if check_dir "snap/moosync"; then
        echo "Publishing Snap changes..."
        cd snap/moosync
        if git diff --quiet && git diff --staged --quiet; then
            echo "No changes in Snap package"
        else
            git add snapcraft.yaml
            git commit -m "Update to version $VERSION"
            git push origin main
        fi
        cd ../../
        echo "✓ Snap changes checked"
    fi

    # Publish Fedora changes
    if check_dir "fedora/dnf"; then
        echo "Publishing Fedora changes..."
        cd fedora/dnf
        if git diff --quiet && git diff --staged --quiet; then
            echo "No changes in Fedora package"
        else
            git add -A
            git commit -m "Update to version $VERSION"
            git push origin main
        fi
        cd ../../
        echo "✓ Fedora changes checked"
    fi

    echo "✓ All changes checked and published where needed"
}

# Run all update functions
update_flatpak
update_snap
update_aur
update_fedora
update_debian
update_nix_flake

# Clean up temporary files
rm -rf "$TEMP_DIR"
echo "Temporary files cleaned up"

echo "All packages updated for version $VERSION"
echo "Please review changes before publishing"

echo $(pwd)
# Ask user if they want to publish changes
read -p "Do you want to publish the changes? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    publish_changes
fi
