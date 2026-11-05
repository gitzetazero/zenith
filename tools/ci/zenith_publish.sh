#!/usr/bin/env bash
# Publishes Zenith's signed Nightly (called by .github/workflows/zenith-release.yml, publish job).
# Needs GH_TOKEN, GITHUB_REPOSITORY, DATE (yyyy-mm-dd), SHA (the commit built) and dist/ holding the four files.
#
# Ordering (the old `nightly` is never deleted, so launchers always find a release):
#   1. a DRAFT nightly-<date> gets all four files and its file list is checked (a leftover from an earlier
#      attempt, draft or published, is removed first);
#   2. `nightly` is updated in place: the archives, then SHA256SUMS, then SHA256SUMS.sig LAST, so during that short
#      window a launcher finds sums that do not match the signature or the archives and fails closed; then its tag
#      moves to this commit and its title changes; its assets are downloaded and verified again;
#   3. only then is nightly-<date> published. Its existence means "done", so the gate lets a retry through while
#      it is missing or `nightly` is older than the date.
set -euo pipefail
repo=$GITHUB_REPOSITORY
dated="nightly-$DATE"
title="Nightly $DATE · ${SHA:0:7}"
want="SHA256SUMS SHA256SUMS.sig horizon-linux-x86_64.tar.gz horizon-windows-x64.zip"
key="$(tr -d '\r\n' < zenith/bridge/zenith-release-keys.txt)"
{
  echo "Zenith build of commit ${SHA:0:7}, signed for the nightly channel (SHA256SUMS, SHA256SUMS.sig)."
  echo
  echo "Zenith is an independent project. It is not affiliated with, endorsed by or sponsored by Mojang or Microsoft; Minecraft is their trademark. No Minecraft code or assets are included: the game files come from your own copy."
  echo
  echo "Built on [Horizon](https://github.com/horizon-rust/horizon)."
} > notes.md
archives=(dist/horizon-windows-x64.zip dist/horizon-linux-x86_64.tar.gz)
# The id of the release (draft or not) with this tag, empty if none.
release_id() { gh api --paginate "repos/$repo/releases" --jq ".[] | select(.tag_name == \"$1\") | .id" | sed -n 1p; }
assets_of() { gh api "repos/$repo/releases/$1" --jq '[.assets[].name] | sort | join(" ")'; }
step() { echo "::group::$*"; }

step "1. draft $dated"
old=$(release_id "$dated")
if [[ -n "$old" ]]; then
  echo "removing the earlier $dated (release $old)"
  gh api -X DELETE "repos/$repo/releases/$old"
  if gh api "repos/$repo/git/ref/tags/$dated" > /dev/null 2>&1; then
    gh api -X DELETE "repos/$repo/git/refs/tags/$dated"
  fi
fi
id=$(gh api -X POST "repos/$repo/releases" -f tag_name="$dated" -f target_commitish="$SHA" -f name="$title" \
  -F body=@notes.md -F draft=true -F prerelease=true --jq .id)
for file in "${archives[@]}" dist/SHA256SUMS dist/SHA256SUMS.sig; do
  gh api -X POST "https://uploads.github.com/repos/$repo/releases/$id/assets?name=$(basename "$file")" \
    -H "Content-Type: application/octet-stream" --input "$file" > /dev/null
done
[[ "$(assets_of "$id")" == "$want" ]] || { echo "::error::the draft's files are not the expected four"; exit 1; }

step "2. nightly"
nightly=$(release_id nightly)
if [[ -n "$nightly" ]]; then
  gh release upload nightly --clobber -R "$repo" "${archives[@]}"
  gh release upload nightly --clobber -R "$repo" dist/SHA256SUMS
  gh release upload nightly --clobber -R "$repo" dist/SHA256SUMS.sig
  gh api -X PATCH "repos/$repo/git/refs/tags/nightly" -f sha="$SHA" -F force=true > /dev/null
  gh api -X PATCH "repos/$repo/releases/$nightly" -f name="$title" -F body=@notes.md -F prerelease=true -f make_latest=false > /dev/null
else
  gh release create nightly --prerelease --latest=false --target "$SHA" --title "$title" --notes-file notes.md \
    "${archives[@]}" dist/SHA256SUMS dist/SHA256SUMS.sig
fi
[[ "$(assets_of "$(release_id nightly)")" == "$want" ]] || { echo "::error::nightly does not hold exactly the expected four files"; exit 1; }
rm -rf check && mkdir check
gh release download nightly -R "$repo" -D check
cargo run --locked --quiet -p horizon-release-sign -- verify --tag nightly --key "$key" check

step "3. publish $dated"
gh api -X PATCH "repos/$repo/releases/$id" -F draft=false -f make_latest=false > /dev/null
echo "::endgroup::"
echo "published $dated and updated nightly"
