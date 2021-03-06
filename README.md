id3-music-organiser
======

A simple command line program to organise your mp3 files. Written in Rust. Built and tested with travis.

[![Rust](https://img.shields.io/badge/Rust%20%3E%3D%201.53-000.svg?style=flat-square&logo=rust&colorA=ffffff&style=popout)](https://rust-lang.org/)
[![Unit Test Coverage](https://img.shields.io/badge/Unit%20Test%20Coverage-88%25-brightgreen)]()
[![Build Status](https://travis-ci.org/craigmayhew/id3-music-organiser.svg?branch=master)](https://travis-ci.org/craigmayhew/id3-music-organiser)

## Usage ##
You will need an "unsorted" directory containing your files. This folder should be at the same level as id3org.
```bash
./id3org
./id3org --skipalbums
```

## Roadmap ##
 - [x] Break code into testable functions
 - [x] Add tests and report coverage
 - [x] Add command line switches e.g. --skipalbums
 - [x] Create a prerelease for people to test
 - [ ] Implement BDD https://github.com/bbqsrc/cucumber-rust
 - [ ] Launch version 1.0 with binary downloads
