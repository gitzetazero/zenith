# Contributing to Zenith

Thank you for wanting to help! This guide is short.

## Before you start

- Check whether there is already an [issue](https://github.com/gitzetazero/zenith/issues) about the same thing.
- For a bug in the game or the launcher, open an issue with the "Bug" template. For an idea, use "Idea".
- A security problem does **not** go in a public issue: read [`SECURITY.md`](SECURITY.md).
- We all follow the [code of conduct](CODE_OF_CONDUCT.md).

## Build and test

The build commands are in the [README](README.md#build-from-source). To test:

```sh
cargo test -p horizon-protocol -p horizon-client
cd zenith/bridge && cargo test
python tools/check_public_tree.py
python -m unittest discover zenith/ci
```

## Code rules

- **Nothing from Mojang in the repository.** No classes, no assets, no jars. Zenith prepares the game from the copy each person has.
- **Original code or code under a compatible licence.** If you copy or build on code from another project, say which, with a link and its licence (MIT, BSD, Apache-2.0, ISC, zlib, 0BSD, Unlicense or CC0).
- **Small changes.** One change per pull request. Do not reformat files you do not touch.
- **New Zenith work goes in `zenith/`.** In the rest of the code, only small hooks marked `// zenith:`. Internal names (`horizon-client`, `HORIZON_*`) are not renamed. The reason and the other rules are in [`ZENITH.md`](ZENITH.md).
- **Honest documentation.** Say what is tested and what is not. Do not state performance figures you cannot back up.

## Sending a change

1. Fork and make a branch.
2. Build and test with the commands above.
3. Open a pull request with the Zenith template: add `?template=zenith.md` to the address of the new pull request.
4. Answer the comments. Small pull requests are reviewed sooner.

If your change also helps the project we come from, [Horizon](https://github.com/horizon-rust/horizon), propose it there: that keeps the difference between the two projects small.
