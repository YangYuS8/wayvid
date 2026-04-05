#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
	echo "Usage: $0 <version>"
	echo "Example: $0 0.6.0"
	exit 1
fi

version="$1"

if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "Version must follow semantic version format: X.Y.Z"
	exit 1
fi

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

sed -i "s/^version = \"[0-9]\+\.[0-9]\+\.[0-9]\+\"$/version = \"${version}\"/" "$repo_root/Cargo.toml"
sed -i "s/^pkgver=.*/pkgver=${version}/" "$repo_root/packaging/aur/lwe/PKGBUILD"
sed -i "s/^pkgrel=.*/pkgrel=1/" "$repo_root/packaging/aur/lwe/PKGBUILD"
sed -i "s/^pkgver=.*/pkgver=${version}.beta.0.0000000/" "$repo_root/packaging/aur/lwe-git/PKGBUILD"
sed -i "s/^pkgrel=.*/pkgrel=1/" "$repo_root/packaging/aur/lwe-git/PKGBUILD"

(cd "$repo_root/packaging/aur/lwe" && makepkg --printsrcinfo >.SRCINFO)
(cd "$repo_root/packaging/aur/lwe-git" && makepkg --printsrcinfo >.SRCINFO)

echo "Synchronized project version to ${version}"
