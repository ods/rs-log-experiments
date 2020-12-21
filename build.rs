use regex::Regex;

fn main() {
    // The resulting format is `major.minor.patch[+N+hash][+dirty]`.  The parts
    // are:
    //   * release (major.minor.patch)
    //   * number of commits `N` on top of released version in development
    //     version with commit hash abbreviation `hash`
    //   * `dirty` marker for local uncommitted changes
    //
    // The release and revision come from annotated tag matching "*.*.*"
    // pattern.  It falls back on `hash[+dirty]` when no release tag yet.
    //
    // `--match='*.*.*'` to take into account release tags only (and only
    //      annotated, not lightweight)
    // `--always` is for fallback while there were no releases yet
    // `--first-parent` discards merge with revision tag after release (beware
    //      that this might become undesired if we switch to git-flow)
    // `--dirty` adds "-dirty" suffix when there are uncommited changes
    //
    // "-" are replaced with "+" since RPM doesn't allow dashes in version
    // ("." is not used to avoid collision with revision).
    // "g" (stands for "git") before hash is removed to avoid
    // misinterpretation.

    let app_version = Regex::new("-(?P<n_commits>[0-9]+)-g").unwrap().replace(
        git_version::git_version!(
            args = ["--always", "--match=*.*.*", "--first-parent", "--dirty"]
        ),
        "+$n_commits+",
    );
    println!("cargo:rustc-env=APP_VERSION={}", app_version);
}
