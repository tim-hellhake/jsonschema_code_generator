use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[doc = "Generated from schemas/draft-04.json"]
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Unknown {
    #[serde(rename = "$schema")]
    pub dollar_schema: Option<String>,
    #[serde(rename = "additionalItems")]
    pub additional_items: Option<Value>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Value>,
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<Unknown>>,
    #[serde(rename = "anyOf")]
    pub any_of: Option<Vec<Unknown>>,
    pub default: Option<Value>,
    pub definitions: Option<BTreeMap<String, Value>>,
    pub dependencies: Option<BTreeMap<String, Value>>,
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enum_: Option<Vec<Value>>,
    #[serde(rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<bool>,
    #[serde(rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<bool>,
    pub format: Option<String>,
    pub id: Option<String>,
    pub items: Option<Value>,
    #[serde(rename = "maxItems")]
    pub max_items: Option<i64>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<i64>,
    #[serde(rename = "maxProperties")]
    pub max_properties: Option<i64>,
    pub maximum: Option<f64>,
    #[serde(rename = "minItems")]
    pub min_items: Option<Value>,
    #[serde(rename = "minLength")]
    pub min_length: Option<Value>,
    #[serde(rename = "minProperties")]
    pub min_properties: Option<Value>,
    pub minimum: Option<f64>,
    #[serde(rename = "multipleOf")]
    pub multiple_of: Option<f64>,
    pub not: Option<Box<Unknown>>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<Unknown>>,
    pub pattern: Option<String>,
    #[serde(rename = "patternProperties")]
    pub pattern_properties: Option<BTreeMap<String, Value>>,
    pub properties: Option<BTreeMap<String, Value>>,
    pub required: Option<Vec<String>>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<Value>,
    #[serde(rename = "uniqueItems")]
    pub unique_items: Option<bool>,
}
