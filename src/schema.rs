/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum Types {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "object")]
    Object,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(rename = "$ref")]
    pub ref_: Option<String>,

    pub title: Option<String>,

    #[serde(rename = "type")]
    pub type_: Option<Types>,

    #[serde(rename = "enum")]
    pub enum_: Option<Vec<Value>>,

    pub required: Option<Vec<String>>,

    pub constant: Option<Value>,

    #[serde(default)]
    pub properties: BTreeMap<String, Schema>,

    #[serde(default, rename = "patternProperties")]
    pub pattern_properties: BTreeMap<String, Schema>,

    #[serde(default)]
    pub items: Box<Option<Schema>>,

    #[serde(default)]
    pub definitions: BTreeMap<String, Schema>,

    #[serde(default, rename = "$defs")]
    pub defs: BTreeMap<String, Schema>,

    #[serde(default, rename = "oneOf")]
    pub one_of: Vec<Schema>,

    #[serde(default, rename = "anyOf")]
    pub any_of: Vec<Schema>,

    #[serde(default, rename = "allOf")]
    pub all_of: Vec<Schema>,
}
