# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [1.1.1] - 2025-04-14
- Fix wrong date formatting [#531](https://github.com/svenstaro/dummyhttp/pull/531)

## [1.1.0] - 2024-08-23
- Added `--delay` to delay HTTP responses [#514](https://github.com/svenstaro/dummyhttp/pull/514) (thanks @Sebekerga)

## [1.0.3] - 2023-04-28
- Update dependencies
- Build binary for aarch64-apple-darwin

## [1.0.2] - 2022-09-14
- Add completions (`--print-completions`) and manpage (`--print-manpage`)

## [1.0.1] - 2022-09-14
- Add more architectures
- Renamed `--cert` to `--tls-cert` and `--key` to `--tls-key` but the old flags are
  still usable (though hidden)
- Update dependencies

## [1.0.0] - 2022-09-11
- Use ubuntu base image for container images

## [0.6.2] - 2022-09-11
- Update dependencies
- Modernize CI
- Build and release more architectures on GitHub and Docker Hub

## [0.6.0] - 2022-09-03
- Add [Tera](https://tera.netlify.app/) templating to body responses.

## [0.5.0] - 2022-08-24

- Rewritten to Axum which makes the code a lot simpler and allows for using more up-to-date
  dependencies
- Will now have accurate headers logged (including body size)

## [0.4.3] - 2019-11-14

- Improve verbose output in some corner cases

## [0.4.2] - 2019-10-21

- Improve user-visible errors (thanks to `anyhow`)

## [0.4.1] - 2019-10-14

- Print sent body

## [0.4.0] - 2019-10-01

- Add really sweet colors (#30)
- Add integration tests (#26)
- Fallback to basic log output in case we don't have a terminal attached

## [0.3.1] - 2019-09-13

- Fix output being different for body/non-body requests

## [0.3.0] - 2019-09-12

- Add TLS support (#25)
- Print body (#13)

<!-- next-url -->
[Unreleased]: https://github.com/svenstaro/dummyhttp/compare/v1.1.1...HEAD
[1.1.1]: https://github.com/svenstaro/dummyhttp/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/svenstaro/dummyhttp/compare/v1.0.3...v1.1.0
[1.0.3]: https://github.com/svenstaro/dummyhttp/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/svenstaro/dummyhttp/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/svenstaro/dummyhttp/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/svenstaro/dummyhttp/compare/v0.6.2...v1.0.0
[0.6.2]: https://github.com/svenstaro/dummyhttp/compare/v0.6.0...v0.6.2
[0.6.0]: https://github.com/svenstaro/dummyhttp/compare/0.5.0...v0.6.0
