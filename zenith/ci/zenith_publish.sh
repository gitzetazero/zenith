#!/usr/bin/env bash
# Publishes Zenith's signed Nightly (called by .github/workflows/zenith-release.yml, publish job).
# Needs GH_TOKEN, GITHUB_REPOSITORY, DATE (yyyy-mm-dd), SHA (the commit built) and dist/ holding the eight files
# (the two game archives, the bridge exe, zenith-build.txt, the launcher installer and portable zip, SHA256SUMS
# and SHA256SUMS.sig).
#
# What players see: the dated release (the repository's "latest") holds ONLY the launcher installer and portable zip.
# The signed game channel (game archives, bridge, build stamp, SHA256SUMS + .sig) lives in the `nightly` release, which
# the launcher and the game fetch from; it is not meant for download.
#
# Ordering (the old `nightly` is never deleted, so launchers always find a release):
#   1. a DRAFT nightly-<date> gets the two launcher files and its file list is checked (a leftover from an earlier
#      attempt, draft or published, is removed first);
#   2. `nightly` is updated in place: the four game files, then SHA256SUMS, then SHA256SUMS.sig LAST, so during that
#      short window a launcher finds sums that do not match the signature or the archives and fails closed; then its
#      tag moves to this commit and its title changes; its assets are downloaded and verified again;
#   3. only then is nightly-<date> published, as a normal release (prerelease=false) and the repository's one "latest"
#      (`nightly` is a normal release too but never "latest"). Its existence means "done", so the gate lets a retry
#      through while it is missing or `nightly` is older than the date.
set -euo pipefail
repo=$GITHUB_REPOSITORY
dated="nightly-$DATE"
title="Zenith $DATE · ${SHA:0:7}"
channel_title="Update channel $DATE · ${SHA:0:7} (used by the launcher, not for download)"
want_channel="SHA256SUMS SHA256SUMS.sig horizon-linux-x86_64.tar.gz horizon-windows-x64.zip zenith-bridge-windows-x64.exe zenith-build.txt" # sorted as jq sorts
want_launcher="Zenith-Launcher-Portable-x64.zip Zenith-Launcher-Setup-x64.exe"
key="$(tr -d '
' < zenith/bridge/zenith-release-keys.txt)"
setup_sum=$(sha256sum dist/Zenith-Launcher-Setup-x64.exe | cut -d' ' -f1)
portable_sum=$(sha256sum dist/Zenith-Launcher-Portable-x64.zip | cut -d' ' -f1)
{
  echo "Zenith build of commit ${SHA:0:7}."
  echo
  echo '**Players (Windows):** download `Zenith-Launcher-Setup-x64.exe`, run it and open "Zenith Launcher". It installs for your user only (no administrator). Prefer no installer? `Zenith-Launcher-Portable-x64.zip` is the same in a folder. Steps: zenith/README.md.'
  echo
  echo "SHA-256:"
  echo
  echo '```'
  echo "$setup_sum  Zenith-Launcher-Setup-x64.exe"
  echo "$portable_sum  Zenith-Launcher-Portable-x64.zip"
  echo '```'
  echo
  echo 'The files are **not code-signed for Windows**, so SmartScreen will warn ("Windows protected your PC": More info, Run anyway). Comparing the hash only catches a corrupt download; the game builds the launcher downloads later are checked with an Ed25519 signature.'
  echo
  echo "Zenith is an independent project. It is not affiliated with, endorsed by or sponsored by Mojang or Microsoft; Minecraft is Mojang's trademark. No Minecraft code or assets are included: the game files come from your own copy."
  echo
  echo "Built on [Horizon](https://github.com/horizon-rust/horizon)."
} > notes.md
{
  echo "The signed game channel of Zenith build ${SHA:0:7} (SHA256SUMS, SHA256SUMS.sig). The Zenith Launcher and the game download and verify these files themselves: **there is nothing to install from here**. Get the launcher from the latest release."
} > notes-channel.md
channel=(dist/horizon-windows-x64.zip dist/horizon-linux-x86_64.tar.gz dist/zenith-bridge-windows-x64.exe dist/zenith-build.txt)
launcher=(dist/Zenith-Launcher-Setup-x64.exe dist/Zenith-Launcher-Portable-x64.zip)
# The id of the release (draft or not) with this tag, empty if none.
release_id() { gh api --paginate "repos/$repo/releases" --jq ".[] | select(.tag_name == \"$1\") | .id" | sed -n 1p; }
assets_of() { gh api "repos/$repo/releases/$1" --jq '[.assets[].name] | sort | join(" ")'; }
step() { echo "::group::$*"; }

step "1. draft $dated"
old=$(release_id "$dated")
if [[ -n "$old" ]]; then
  echo "removing the earlier $dated (release $old)"
  gh api -X DELETE "repos/$repo/releases/$old"
fi
id=$(gh api -X POST "repos/$repo/releases" -f tag_name="$dated" -f target_commitish="$SHA" -f name="$title"   -F body=@notes.md -F draft=true -F prerelease=false --jq .id)
for file in "${launcher[@]}"; do
  gh api -X POST "https://uploads.github.com/repos/$repo/releases/$id/assets?name=$(basename "$file")"     -H "Content-Type: application/octet-stream" --input "$file" > /dev/null
done
[[ "$(assets_of "$id")" == "$want_launcher" ]] || { echo "::error::the draft's files are not the launcher installer and portable zip"; exit 1; }
# The token cannot delete tags (403), so a tag left by an earlier attempt is moved instead; a new one is made
# from target_commitish when the release goes public.
if gh api "repos/$repo/git/ref/tags/$dated" > /dev/null 2>&1; then
  gh api -X PATCH "repos/$repo/git/refs/tags/$dated" -f sha="$SHA" -F force=true > /dev/null
fi

step "2. nightly (the game channel)"
nightly=$(release_id nightly)
if [[ -n "$nightly" ]]; then
  # an earlier layout also kept the launcher files here
  for stale in Zenith-Launcher-Setup-x64.exe Zenith-Launcher-Portable-x64.zip; do
    gh release delete-asset nightly "$stale" -R "$repo" --yes 2>/dev/null || true
  done
  gh release upload nightly --clobber -R "$repo" "${channel[@]}"
  gh release upload nightly --clobber -R "$repo" dist/SHA256SUMS
  gh release upload nightly --clobber -R "$repo" dist/SHA256SUMS.sig
  gh api -X PATCH "repos/$repo/git/refs/tags/nightly" -f sha="$SHA" -F force=true > /dev/null
  gh api -X PATCH "repos/$repo/releases/$nightly" -f name="$channel_title" -F body=@notes-channel.md -F prerelease=false -f make_latest=false > /dev/null
else
  gh release create nightly --latest=false --target "$SHA" --title "$channel_title" --notes-file notes-channel.md     "${channel[@]}" dist/SHA256SUMS dist/SHA256SUMS.sig
fi
[[ "$(assets_of "$(release_id nightly)")" == "$want_channel" ]] || { echo "::error::nightly does not hold exactly the expected six files"; exit 1; }
rm -rf check && mkdir check
gh release download nightly -R "$repo" -D check
cargo run --locked --quiet -p horizon-release-sign -- verify --tag nightly --key "$key" check

step "3. publish $dated"
gh api -X PATCH "repos/$repo/releases/$id" -F draft=false -F prerelease=false -f make_latest=true > /dev/null
echo "::endgroup::"
echo "published $dated and updated nightly"
