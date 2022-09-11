# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->

## [Unreleased] - ReleaseDate
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

- Fix output being different for body/non-body requets

## [0.3.0] - 2019-09-12

- Add TLS support (#25)
- Print body (#13)

<!-- next-url -->
[Unreleased]: https://github.com/svenstaro/dummyhttp/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/svenstaro/dummyhttp/compare/0.5.0...v0.6.0
