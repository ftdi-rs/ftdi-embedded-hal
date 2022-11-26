// See https://crates.io/crates/version-sync

#[test]
fn test_readme_deps() {
    version_sync::assert_markdown_deps_updated!("README.md");
}
