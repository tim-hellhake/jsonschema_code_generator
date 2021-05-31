/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate serde_derive;

use std::path::Path;

use crate::generator::Generator;
use proc_macro2::TokenStream;

mod generated;
mod generator;
mod keywords;
mod parser;
mod ref_parser;
mod resolver;
mod sanitizer;
mod schema;

pub fn generate(path: &Path) -> String {
    generate_token_stream(path).to_string()
}

pub fn generate_token_stream(path: &Path) -> TokenStream {
    let mut generator = Generator::new();
    generator.add_file(path);
    generator.into()
}

#[cfg(test)]
mod lib_tests {
    use proc_macro2::TokenStream;

    use std::{
        fs,
        io::Write,
        path::Path,
        process::{Command, Stdio},
    };

    use crate::generator::Generator;

    #[test]
    fn test() {
        let mut generator = Generator::new();
        generator.add_file(Path::new("schemas/draft-04.json"));
        let tokens: TokenStream = generator.into();
        let actual = tokens.to_string();
        let expected = fs::read_to_string("schemas/draft-04.rs").unwrap();

        assert_eq!(format(actual), expected);
    }

    fn format(text: impl std::fmt::Display) -> String {
        let mut rustfmt = Command::new("rustfmt")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        write!(rustfmt.stdin.take().unwrap(), "{}", text).unwrap();
        let output = rustfmt.wait_with_output().unwrap();
        String::from_utf8(output.stdout).unwrap()
    }
}
