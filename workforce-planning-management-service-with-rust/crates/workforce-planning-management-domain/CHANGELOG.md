# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this crate adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — the crate (WPM-T529, WPM-D203)

The domain logic and business rules of WPM, moved from the service's `src/rules/` and `src/dates.rs` with no change to what
they do: 126 rule modules and the date helpers, and their 1,126 unit tests. Not published yet; the service depends on it by
path and version.
