/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[derive(PartialEq, Debug)]
pub struct RefPath {
    pub file: Option<String>,
    pub path: Option<String>,
}

pub fn parse_ref(full_path: String) -> RefPath {
    let parts: Vec<&str> = full_path.split("#").collect();

    let file = match parts[0] {
        "" => None,
        _ => Some(parts[0].to_string()),
    };

    let path = match parts.len() {
        1 => None,
        2 => match parts[1] {
            "" => None,
            _ => Some(parts[1].to_string()),
        },
        _ => panic!("Malformed ref path: {}", full_path),
    };

    RefPath { file, path }
}

#[cfg(test)]
mod ref_parser_tests {
    use crate::ref_parser::{parse_ref, RefPath};

    #[test]
    fn should_parse_empty_path() {
        assert_eq!(
            RefPath {
                file: None,
                path: None,
            },
            parse_ref(String::from(""))
        );
    }

    #[test]
    fn should_parse_file_path() {
        assert_eq!(
            RefPath {
                file: Some(String::from("definitions.json")),
                path: None,
            },
            parse_ref(String::from("definitions.json"))
        );
    }

    #[test]
    fn should_parse_local_path() {
        assert_eq!(
            RefPath {
                file: None,
                path: Some(String::from("/abc")),
            },
            parse_ref(String::from("#/abc"))
        );
    }

    #[test]
    fn should_parse_combined_path() {
        assert_eq!(
            RefPath {
                file: Some(String::from("definitions.json")),
                path: Some(String::from("/abc")),
            },
            parse_ref(String::from("definitions.json#/abc"))
        );
    }
}
