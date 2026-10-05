# Releasing NEXORA

A release is produced by [`.github/workflows/release.yml`](../.github/workflows/release.yml)
from a tagged commit. Nothing is built on a laptop and uploaded by hand, which
is the only way the archives can be said to correspond to the source.

## Cutting one

```bash
git switch main && git pull
cargo test --workspace                  # the tag should not be the first time
git tag -a v0.1.0 -m "NEXORA v0.1.0"
git push origin v0.1.0
```

The tag is what triggers the workflow. It builds on Linux, Windows and macOS,
and on each platform it **runs the binaries it is about to package** before
packaging them: the vertical slice against the real binary, plus a benchmark
smoke pass and one texture generation. A build that compiles and cannot run
does not reach the release.

Then it packages, checksums, and publishes a GitHub Release carrying:

- `nexora-<version>-linux-x86_64.tar.gz`
- `nexora-<version>-windows-x86_64.zip`
- `nexora-<version>-macos-aarch64.tar.gz`
- `SHA256SUMS` covering all three

## Trying the pipeline without releasing

Run the workflow manually (`workflow_dispatch`) from the Actions tab. Every
build job runs; the publish job is skipped because there is no tag. The
archives land as workflow artifacts named after the commit
(`0.0.0-dev+<sha>`), so a dry run can never be mistaken for a release.

To test a change to the workflow itself, dispatch it on the branch that
carries the change. GitHub only *offers* the dispatch for a workflow that
exists on the default branch, but the run uses that branch's copy of the file.

> **What has actually run.** Nothing here could execute before the workflow
> reached `main`, because until then GitHub did not offer the dispatch at all.
> Its first run was a dry run from `main` right after the merge
> ([36075825860](https://github.com/Lucas-Belucci-Bellini/NEXORA/actions/runs/36075825860)):
> Linux and macOS passed every step, and Windows built, ran the slice, started
> the tools and packed the archive — then failed at the checksum, because the
> Windows runner's Git Bash has no `shasum`. The fix was proved before it
> merged, by dispatching the same dry run on the fix's own branch
> ([36077804637](https://github.com/Lucas-Belucci-Bellini/NEXORA/actions/runs/36077804637)):
> the log shows the branch's copy of the step running, and all three platforms
> built, ran, packed, checksummed and uploaded, with publish skipped.
>
> **The publish job has never run, and no dry run can run it** — it is gated
> on a tag. The first real release is the first time `gh release create`
> executes and the first time the per-platform checksum lines are merged into
> `SHA256SUMS` on a runner. That merge was exercised locally against the exact
> line format the build jobs now write, which is evidence about its input, not
> a run of the job.

## What the archive contains

The three binaries, `content/` so the texture forge has definitions to read,
the README, and both licence files. `docs/release-preamble.md` is the fixed
part of every release's notes and says the same thing to whoever downloads it;
`--generate-notes` appends what changed.

## Versioning

`version` in the workspace `Cargo.toml` is the source of truth; the tag should
match it with a `v` prefix. The workflow does not enforce that today — it takes
the version from the tag name — so bump the manifest in the same commit you
tag, or the archive name and the crate version disagree.

## Where it shows up

[`web/`](../web) is the download page. It reads the GitHub Releases API at load
time rather than hardcoding a version, so publishing a release is all it takes
for the page to offer it — there is no second place to update, and no way for
the page to advertise a build that does not exist.
