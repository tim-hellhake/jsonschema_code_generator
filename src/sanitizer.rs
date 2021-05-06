/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::keywords::RUST_KEYWORDS;
use convert_case::{Case, Casing};

pub fn sanitize_property_name(name: String) -> String {
    escape_keywords(
        split_camel_case(name)
            .replace("@", " at ")
            .replace("$", " dollar ")
            .to_case(Case::Snake),
    )
}

fn escape_keywords(name: String) -> String {
    if RUST_KEYWORDS.contains(&name.as_str()) {
        name + "_"
    } else {
        name
    }
}

fn split_camel_case(name: String) -> String {
    name.chars()
        .flat_map(|c| {
            if c.is_uppercase() {
                vec![' ', c]
            } else {
                vec![c]
            }
        })
        .collect()
}

pub fn sanitize_struct_name(name: String) -> String {
    name.replace("@", " at ")
        .replace("$", " dollar ")
        .to_case(Case::Pascal)
}

#[cfg(test)]
mod sanitizer_tests {
    use crate::sanitizer::{sanitize_property_name, sanitize_struct_name};

    #[test]
    fn should_replace_at_in_property_names() {
        let s = sanitize_property_name(String::from("@type"));
        assert_eq!(s, "at_type");
    }

    #[test]
    fn should_replace_dollar_in_property_names() {
        let s = sanitize_property_name(String::from("$type"));
        assert_eq!(s, "dollar_type");
    }

    #[test]
    fn should_create_snake_case_property_names() {
        let s = sanitize_property_name(String::from("a-Wonderful rustProperty"));
        assert_eq!(s, "a_wonderful_rust_property");
    }

    #[test]
    fn should_create_snake_case_property_names_from_camel_case() {
        let s = sanitize_property_name(String::from("aWonderfulProperty"));
        assert_eq!(s, "a_wonderful_property");
    }

    #[test]
    fn should_rename_reserved_keywords() {
        let s = sanitize_property_name(String::from("enum"));
        assert_eq!(s, "enum_");
    }

    #[test]
    fn should_create_pascal_case_struct_names() {
        let s = sanitize_struct_name(String::from("a-wonderful_rust struct"));
        assert_eq!(s, "AWonderfulRustStruct");
    }

    #[test]
    fn should_replace_at_in_struct_names() {
        let s = sanitize_struct_name(String::from("@type"));
        assert_eq!(s, "AtType");
    }

    #[test]
    fn should_replace_dollar_in_struct_names() {
        let s = sanitize_struct_name(String::from("$type"));
        assert_eq!(s, "DollarType");
    }
}
