// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// ignore-windows TempDir may cause IoError on windows: #10462

#![feature(macro_rules)]

extern crate glob;

use glob::glob;
use std::os;
use std::io;
use std::io::TempDir;

macro_rules! assert_eq { ($e1:expr, $e2:expr) => (
    if $e1 != $e2 {
        panic!("{} != {}", stringify!($e1), stringify!($e2))
    }
) }

#[test]
fn main() {
    fn mk_file(path: &str, directory: bool) {
        if directory {
            io::fs::mkdir(&Path::new(path), io::USER_RWX).unwrap();
        } else {
            io::File::create(&Path::new(path)).unwrap();
        }
    }

    fn abs_path(path: &str) -> Path {
        os::getcwd().unwrap().join(&Path::new(path))
    }

    fn glob_vec(pattern: &str) -> Vec<Path> {
        glob(pattern).collect()
    }

    let root = TempDir::new("glob-tests");
    let root = root.ok().expect("Should have created a temp directory");
    assert!(os::change_dir(root.path()).is_ok());

    mk_file("aaa", true);
    mk_file("aaa/apple", true);
    mk_file("aaa/orange", true);
    mk_file("aaa/tomato", true);
    mk_file("aaa/tomato/tomato.txt", false);
    mk_file("aaa/tomato/tomoto.txt", false);
    mk_file("bbb", true);
    mk_file("bbb/specials", true);
    mk_file("bbb/specials/!", false);

    // windows does not allow `*` or `?` characters to exist in filenames
    if os::consts::FAMILY != "windows" {
        mk_file("bbb/specials/*", false);
        mk_file("bbb/specials/?", false);
    }

    mk_file("bbb/specials/[", false);
    mk_file("bbb/specials/]", false);
    mk_file("ccc", true);
    mk_file("xyz", true);
    mk_file("xyz/x", false);
    mk_file("xyz/y", false);
    mk_file("xyz/z", false);

    mk_file("r", true);
    mk_file("r/current_dir.md", false);
    mk_file("r/one", true);
    mk_file("r/one/a.md", false);
    mk_file("r/one/another", true);
    mk_file("r/one/another/a.md", false);
    mk_file("r/another", true);
    mk_file("r/another/a.md", false);
    mk_file("r/two", true);
    mk_file("r/two/b.md", false);
    mk_file("r/three", true);
    mk_file("r/three/c.md", false);

    // all recursive entities
    assert_eq!(glob_vec("r/**"), vec!(
        abs_path("r/another"),
        abs_path("r/another/a.md"),
        abs_path("r/current_dir.md"),
        abs_path("r/one"),
        abs_path("r/one/a.md"),
        abs_path("r/one/another"),
        abs_path("r/one/another/a.md"),
        abs_path("r/three"),
        abs_path("r/three/c.md"),
        abs_path("r/two"),
        abs_path("r/two/b.md")));

    // collapse consecutive recursive patterns
    assert_eq!(glob_vec("r/**/**"), vec!(
        abs_path("r/another"),
        abs_path("r/another/a.md"),
        abs_path("r/current_dir.md"),
        abs_path("r/one"),
        abs_path("r/one/a.md"),
        abs_path("r/one/another"),
        abs_path("r/one/another/a.md"),
        abs_path("r/three"),
        abs_path("r/three/c.md"),
        abs_path("r/two"),
        abs_path("r/two/b.md")));

    // followed by a wildcard
    assert_eq!(glob_vec("r/**/*.md"), vec!(
        abs_path("r/another/a.md"),
        abs_path("r/current_dir.md"),
        abs_path("r/one/a.md"),
        abs_path("r/one/another/a.md"),
        abs_path("r/three/c.md"),
        abs_path("r/two/b.md")));

    // followed by a precise pattern
    assert_eq!(glob_vec("r/one/**/a.md"), vec!(
        abs_path("r/one/a.md"),
        abs_path("r/one/another/a.md")));

    // followed by another recursive pattern
    // collapses consecutive recursives into one
    assert_eq!(glob_vec("r/one/**/**/a.md"), vec!(
        abs_path("r/one/a.md"),
        abs_path("r/one/another/a.md")));

    // followed by two precise patterns
    assert_eq!(glob_vec("r/**/another/a.md"), vec!(
        abs_path("r/another/a.md"),
        abs_path("r/one/another/a.md")));

    assert_eq!(glob_vec(""), Vec::new());
    assert_eq!(glob_vec("."), vec!(os::getcwd().unwrap()));
    assert_eq!(glob_vec(".."), vec!(os::getcwd().unwrap().join("..")));

    assert_eq!(glob_vec("aaa"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("aaa/"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("a"), Vec::new());
    assert_eq!(glob_vec("aa"), Vec::new());
    assert_eq!(glob_vec("aaaa"), Vec::new());

    assert_eq!(glob_vec("aaa/apple"), vec!(abs_path("aaa/apple")));
    assert_eq!(glob_vec("aaa/apple/nope"), Vec::new());

    // windows should support both / and \ as directory separators
    if os::consts::FAMILY == "windows" {
        assert_eq!(glob_vec("aaa\\apple"), vec!(abs_path("aaa/apple")));
    }

    assert_eq!(glob_vec("???/"), vec!(
        abs_path("aaa"),
        abs_path("bbb"),
        abs_path("ccc"),
        abs_path("xyz")));

    assert_eq!(glob_vec("aaa/tomato/tom?to.txt"), vec!(
        abs_path("aaa/tomato/tomato.txt"),
        abs_path("aaa/tomato/tomoto.txt")));

    assert_eq!(glob_vec("xyz/?"), vec!(
        abs_path("xyz/x"),
        abs_path("xyz/y"),
        abs_path("xyz/z")));

    assert_eq!(glob_vec("a*"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("*a*"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("a*a"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("aaa*"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("*aaa"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("*aaa*"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("*a*a*a*"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("aaa*/"), vec!(abs_path("aaa")));

    assert_eq!(glob_vec("aaa/*"), vec!(
        abs_path("aaa/apple"),
        abs_path("aaa/orange"),
        abs_path("aaa/tomato")));

    assert_eq!(glob_vec("aaa/*a*"), vec!(
        abs_path("aaa/apple"),
        abs_path("aaa/orange"),
        abs_path("aaa/tomato")));

    assert_eq!(glob_vec("*/*/*.txt"), vec!(
        abs_path("aaa/tomato/tomato.txt"),
        abs_path("aaa/tomato/tomoto.txt")));

    assert_eq!(glob_vec("*/*/t[aob]m?to[.]t[!y]t"), vec!(
        abs_path("aaa/tomato/tomato.txt"),
        abs_path("aaa/tomato/tomoto.txt")));

    assert_eq!(glob_vec("./aaa"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("./*"), glob_vec("*"));
    assert_eq!(glob_vec("*/..").pop().unwrap(), abs_path("."));
    assert_eq!(glob_vec("aaa/../bbb"), vec!(abs_path("bbb")));
    assert_eq!(glob_vec("nonexistent/../bbb"), Vec::new());
    assert_eq!(glob_vec("aaa/tomato/tomato.txt/.."), Vec::new());

    assert_eq!(glob_vec("aaa/tomato/tomato.txt/"), Vec::new());

    assert_eq!(glob_vec("aa[a]"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("aa[abc]"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("a[bca]a"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("aa[b]"), Vec::new());
    assert_eq!(glob_vec("aa[xyz]"), Vec::new());
    assert_eq!(glob_vec("aa[]]"), Vec::new());

    assert_eq!(glob_vec("aa[!b]"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("aa[!bcd]"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("a[!bcd]a"), vec!(abs_path("aaa")));
    assert_eq!(glob_vec("aa[!a]"), Vec::new());
    assert_eq!(glob_vec("aa[!abc]"), Vec::new());

    assert_eq!(glob_vec("bbb/specials/[[]"), vec!(abs_path("bbb/specials/[")));
    assert_eq!(glob_vec("bbb/specials/!"), vec!(abs_path("bbb/specials/!")));
    assert_eq!(glob_vec("bbb/specials/[]]"), vec!(abs_path("bbb/specials/]")));

    if os::consts::FAMILY != "windows" {
        assert_eq!(glob_vec("bbb/specials/[*]"), vec!(abs_path("bbb/specials/*")));
        assert_eq!(glob_vec("bbb/specials/[?]"), vec!(abs_path("bbb/specials/?")));
    }

    if os::consts::FAMILY == "windows" {

        assert_eq!(glob_vec("bbb/specials/[![]"), vec!(
            abs_path("bbb/specials/!"),
            abs_path("bbb/specials/]")));

        assert_eq!(glob_vec("bbb/specials/[!]]"), vec!(
            abs_path("bbb/specials/!"),
            abs_path("bbb/specials/[")));

        assert_eq!(glob_vec("bbb/specials/[!!]"), vec!(
            abs_path("bbb/specials/["),
            abs_path("bbb/specials/]")));

    } else {

        assert_eq!(glob_vec("bbb/specials/[![]"), vec!(
            abs_path("bbb/specials/!"),
            abs_path("bbb/specials/*"),
            abs_path("bbb/specials/?"),
            abs_path("bbb/specials/]")));

        assert_eq!(glob_vec("bbb/specials/[!]]"), vec!(
            abs_path("bbb/specials/!"),
            abs_path("bbb/specials/*"),
            abs_path("bbb/specials/?"),
            abs_path("bbb/specials/[")));

        assert_eq!(glob_vec("bbb/specials/[!!]"), vec!(
            abs_path("bbb/specials/*"),
            abs_path("bbb/specials/?"),
            abs_path("bbb/specials/["),
            abs_path("bbb/specials/]")));

        assert_eq!(glob_vec("bbb/specials/[!*]"), vec!(
            abs_path("bbb/specials/!"),
            abs_path("bbb/specials/?"),
            abs_path("bbb/specials/["),
            abs_path("bbb/specials/]")));

        assert_eq!(glob_vec("bbb/specials/[!?]"), vec!(
            abs_path("bbb/specials/!"),
            abs_path("bbb/specials/*"),
            abs_path("bbb/specials/["),
            abs_path("bbb/specials/]")));

    }
}
