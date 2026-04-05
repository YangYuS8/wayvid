#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 3 ]; then
	echo "Usage: $0 <package_dir> <aur_repo> <commit_message>"
	exit 1
fi

package_dir="$1"
aur_repo="$2"
commit_message="$3"

if [ -z "${AUR_SSH_PRIVATE_KEY:-}" ]; then
	echo "AUR_SSH_PRIVATE_KEY is required"
	exit 1
fi

mkdir -p ~/.ssh
chmod 700 ~/.ssh
cat >~/.ssh/known_hosts <<'EOF'
aur.archlinux.org ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDKF9vAFWdgm9Bi8uc+tYRBmXASBb5cB5iZsB7LOWWFeBrLp3r14w0/9S2vozjgqY5sJLDPONWoTTaVTbhe3vwO8CBKZTEt1AcWxuXNlRnk9FliR1/eNB9uz/7y1R0+c1Md+P98AJJSJWKN12nqIDIhjl2S1vOUvm7FNY43fU2knIhEbHybhwWeg+0wxpKwcAd/JeL5i92Uv03MYftOToUijd1pqyVFdJvQFhqD4v3M157jxS5FTOBrccAEjT+zYmFyD8WvKUa9vUclRddNllmBJdy4NyLB8SvVZULUPrP3QOlmzemeKracTlVOUG1wsDbxknF1BwSCU7CmU6UFP90kpWIyz66bP0bl67QAvlIc52Yix7pKJPbw85+zykvnfl2mdROsaT8p8R9nwCdFsBc9IiD0NhPEHcyHRwB8fokXTajk2QnGhL+zP5KnkmXnyQYOCUYo3EKMXIlVOVbPDgRYYT/XqvBuzq5S9rrU70KoI/S5lDnFfx/+lPLdtcnnEPk=
aur.archlinux.org ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBLMiLrP8pVi5BFX2i3vepSUnpedeiewE5XptnUnau+ZoeUOPkpoCgZZuYfpaIQfhhJJI5qgnjJmr4hyJbe/zxow=
aur.archlinux.org ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEuBKrPzbawxA/k2g6NcyV5jmqwJ2s+zpgZGZ7tpLIcN
EOF
chmod 644 ~/.ssh/known_hosts

eval "$(ssh-agent -s)"
trap 'ssh-agent -k' EXIT
printf "%s\n" "$AUR_SSH_PRIVATE_KEY" | ssh-add -

ssh_opts="-o StrictHostKeyChecking=yes -o UserKnownHostsFile=$HOME/.ssh/known_hosts -o GlobalKnownHostsFile=/dev/null"

set +e
ssh $ssh_opts -T aur@aur.archlinux.org
ssh_rc=$?
set -e
if [ "$ssh_rc" -ne 0 ] && [ "$ssh_rc" -ne 1 ]; then
	echo "AUR SSH connectivity check failed with code ${ssh_rc}"
	exit "$ssh_rc"
fi

repo_dir="aur-repo"
rm -rf "$repo_dir"
GIT_SSH_COMMAND="ssh $ssh_opts" git clone "ssh://aur@aur.archlinux.org/${aur_repo}.git" "$repo_dir"

cp "${package_dir}/PKGBUILD" "${repo_dir}/PKGBUILD"
cp "${package_dir}/.SRCINFO" "${repo_dir}/.SRCINFO"

cd "$repo_dir"
git add PKGBUILD .SRCINFO
if git diff --cached --quiet; then
	echo "No AUR metadata changes to publish"
	exit 0
fi

git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git commit -m "$commit_message"
GIT_SSH_COMMAND="ssh $ssh_opts" git push origin HEAD:master
