use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[doc = "Generated from schemas/draft-04.json"]
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Unknown {
    #[serde(rename = "$schema")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dollar_schema: Option<String>,
    #[serde(rename = "additionalItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_items: Option<Value>,
    #[serde(rename = "additionalProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Value>,
    #[serde(rename = "allOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<Unknown>>,
    #[serde(rename = "anyOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Unknown>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definitions: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<Value>>,
    #[serde(rename = "exclusiveMaximum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<bool>,
    #[serde(rename = "exclusiveMinimum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Value>,
    #[serde(rename = "maxItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i64>,
    #[serde(rename = "maxLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i64>,
    #[serde(rename = "maxProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(rename = "minItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<Value>,
    #[serde(rename = "minLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<Value>,
    #[serde(rename = "minProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(rename = "multipleOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Unknown>>,
    #[serde(rename = "oneOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Unknown>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(rename = "patternProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_properties: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<Value>,
    #[serde(rename = "uniqueItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
}
