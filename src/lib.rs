/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate serde_derive;

use std::path::Path;

use crate::generator::Generator;

mod generator;
mod keywords;
mod parser;
mod ref_parser;
mod resolver;
mod sanitizer;
mod schema;

pub fn generate(path: &Path) -> String {
    let mut generator = Generator::new();
    generator.add_file(path);
    generator.serialize()
}

#[cfg(test)]
mod lib_tests {
    use std::fs;
    use std::path::Path;

    use crate::generator::Generator;

    #[test]
    fn test() {
        let mut generator = Generator::new();
        generator.add_file(Path::new("schemas/draft-04.json"));
        let actual = generator.serialize();
        let expected = fs::read_to_string("schemas/draft-04.rs").unwrap();

        assert_eq!(actual, expected);
    }
}
