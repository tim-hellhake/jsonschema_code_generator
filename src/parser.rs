/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::schema::{Schema, Types};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(PartialEq, Debug)]
pub struct Root {
    pub file: PathBuf,
    pub data_type: Rc<DataType>,
    pub definitions: HashMap<String, Rc<DataType>>,
}

#[derive(PartialEq, Debug)]
pub enum DataType {
    PrimitiveType(PrimitiveType),
    Array(Rc<DataType>),
    Object(Object),
    Map(Rc<DataType>),
    Ref(Ref),
    OneOf(OneOf),
    AnyOf(AnyOf),
    AllOf(AllOf),
    Any,
}

#[derive(PartialEq, Debug)]
pub enum PrimitiveType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
}

#[derive(PartialEq, Debug)]
pub struct PrimitiveTypeInfos {
    pub enum_values: Vec<Value>,
    pub constant: Option<Value>,
}

#[derive(PartialEq, Debug)]
pub struct Object {
    pub src: String,
    pub name: String,
    pub properties: Vec<ObjectProperty>,
}

#[derive(PartialEq, Debug)]
pub struct ObjectProperty {
    pub name: String,
    pub required: bool,
    pub data_type: Rc<DataType>,
}

#[derive(PartialEq, Debug)]
pub struct Ref {
    pub ref_path: String,
}

#[derive(PartialEq, Debug)]
pub struct OneOf {
    pub types: Vec<DataType>,
}

#[derive(PartialEq, Debug)]
pub struct AnyOf {
    pub types: Vec<DataType>,
}

#[derive(PartialEq, Debug)]
pub struct AllOf {
    pub types: Vec<DataType>,
}

pub fn parse_from_file(file: &Path) -> Root {
    let file = match file.exists() {
        true => file.to_path_buf(),
        false => file.to_path_buf().with_extension("json"),
    };

    match fs::read_to_string(&file) {
        Ok(json_schema) => parse_from_string(&file, &json_schema),
        Err(err) => panic!("Could not open {}: {}", &file.display(), err),
    }
}

pub fn parse_from_string(file: &Path, json_schema: &str) -> Root {
    let src = file.display().to_string();
    match serde_json::from_str(json_schema) {
        Ok(schema) => {
            let definitions = parse_definitions(src.clone(), &schema);
            let data_type = Rc::new(parse_type(src, schema, None, None));
            let mut file_buf = PathBuf::new();
            file_buf.push(file);
            Root {
                file: file_buf,
                data_type,
                definitions,
            }
        }
        Err(err) => {
            panic!("Could not parse {}: {}", file.display(), err)
        }
    }
}

fn parse_definitions(src: String, schema: &Schema) -> HashMap<String, Rc<DataType>> {
    let mut definitions = HashMap::new();

    for (name, definition) in schema.defs.clone() {
        let src = format!("{}/$defs/{}", src, name);
        definitions.insert(
            name.clone(),
            Rc::new(parse_type(src, definition, None, Some(name))),
        );
    }

    for (name, definition) in schema.definitions.clone() {
        let src = format!("{}/definitions/{}", src, name);
        definitions.insert(
            name.clone(),
            Rc::new(parse_type(src, definition, None, Some(name))),
        );
    }

    definitions
}

fn parse_type(
    src: String,
    schema: Schema,
    parent_schema: Option<&Schema>,
    property_name: Option<String>,
) -> DataType {
    match schema.ref_ {
        Some(ref_path) => DataType::Ref(Ref { ref_path }),
        None => {
            if schema.one_of.len() > 0 {
                let mut data_types = vec![];

                for (i, alternative) in (0..).zip(schema.clone().one_of) {
                    data_types.push(parse_type(
                        format!("{}/oneOf/{}", src, i),
                        alternative,
                        Some(&schema),
                        None,
                    ));
                }

                return DataType::OneOf(OneOf { types: data_types });
            }

            if schema.any_of.len() > 0 {
                let mut data_types = vec![];

                for (i, alternative) in (0..).zip(schema.clone().any_of) {
                    data_types.push(parse_type(
                        format!("{}/anyOf/{}", src, i),
                        alternative,
                        Some(&schema),
                        None,
                    ));
                }

                return DataType::AnyOf(AnyOf { types: data_types });
            }

            if schema.all_of.len() > 0 {
                let mut data_types = vec![];

                for (i, alternative) in (0..).zip(schema.clone().all_of) {
                    data_types.push(parse_type(
                        format!("{}/allOf/{}", src, i),
                        alternative,
                        Some(&schema),
                        None,
                    ));
                }

                return DataType::AllOf(AllOf { types: data_types });
            }

            let mut enum_values = match &schema.enum_ {
                Some(enum_values) => enum_values.clone(),
                None => vec![],
            };

            match parent_schema {
                Some(parent) => match &parent.enum_ {
                    Some(values) => {
                        for value in values {
                            enum_values.push(value.clone());
                        }
                    }
                    None => {}
                },
                None => {}
            }

            match &schema.type_ {
                Some(type_) => match type_ {
                    Types::Null => DataType::PrimitiveType(PrimitiveType::Null),
                    Types::Boolean => DataType::PrimitiveType(PrimitiveType::Boolean),
                    Types::Integer => DataType::PrimitiveType(PrimitiveType::Integer),
                    Types::Number => DataType::PrimitiveType(PrimitiveType::Number),
                    Types::String => DataType::PrimitiveType(PrimitiveType::String),
                    Types::Array => parse_array_type(src, schema),
                    Types::Object => match schema.pattern_properties.values().nth(0) {
                        Some(schema) => DataType::Map(Rc::new(parse_type(
                            format!("{}/patternProperties", src),
                            schema.clone(),
                            None,
                            None,
                        ))),
                        None => {
                            if schema.properties.len() > 0 {
                                parse_object_type(src, schema, parent_schema, property_name)
                            } else {
                                DataType::Map(Rc::new(DataType::Any))
                            }
                        }
                    },
                },
                None => DataType::Any,
            }
        }
    }
}

fn parse_array_type(src: String, schema: Schema) -> DataType {
    match *schema.items {
        Some(items) => {
            let data_type = parse_type(format!("{}/items", src), items, None, None);

            DataType::Array(Rc::new(data_type))
        }
        None => DataType::Array(Rc::new(DataType::Any)),
    }
}

fn parse_object_type(
    src: String,
    schema: Schema,
    x_of_parent: Option<&Schema>,
    property_name: Option<String>,
) -> DataType {
    let name = match schema.title {
        Some(title) => title,
        None => match x_of_parent {
            Some(parent) => match &parent.title {
                Some(title) => title.to_string(),
                None => match &property_name {
                    Some(title) => title.to_string(),
                    None => String::from("Unknown"),
                },
            },
            None => match &property_name {
                Some(title) => title.to_string(),
                None => String::from("Unknown"),
            },
        },
    };

    let mut required_properties = match schema.required {
        Some(required) => required,
        None => vec![],
    };

    match x_of_parent {
        Some(parent) => match &parent.required {
            Some(required) => {
                for r in required {
                    required_properties.push(r.to_string());
                }
            }
            None => {}
        },
        None => {}
    }

    let mut properties: Vec<ObjectProperty> = vec![];

    for (name, property) in schema.properties {
        let required = required_properties.contains(&name);
        let property = parse_property(
            format!("{}/properties/{}", src, name),
            &name,
            property,
            required,
        );
        properties.push(property);
    }

    return DataType::Object(Object {
        src,
        name,
        properties,
    });
}

fn parse_property(src: String, name: &str, schema: Schema, required: bool) -> ObjectProperty {
    let fallback_name = match &schema.title {
        Some(title) => title.to_string(),
        None => name.to_string(),
    };

    ObjectProperty {
        name: name.to_string(),
        required,
        data_type: Rc::new(parse_type(src, schema, None, Some(fallback_name))),
    }
}

#[cfg(test)]
mod parser_tests {
    use crate::parser::{
        parse_from_file, parse_from_string, AllOf, AnyOf, DataType, Object, ObjectProperty, OneOf,
        PrimitiveType, Root,
    };
    use std::collections::HashMap;
    use std::path::Path;
    use std::rc::Rc;

    #[test]
    fn should_parse_null() {
        let schema = parse_from_file(Path::new("src/examples/parser/null.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &primitive_type(PrimitiveType::Null)
        );
    }

    #[test]
    fn should_parse_boolean() {
        let schema = parse_from_file(Path::new("src/examples/parser/boolean.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &primitive_type(PrimitiveType::Boolean)
        );
    }

    #[test]
    fn should_parse_integer() {
        let schema = parse_from_file(Path::new("src/examples/parser/integer.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &primitive_type(PrimitiveType::Integer)
        );
    }

    #[test]
    fn should_parse_number() {
        let schema = parse_from_file(Path::new("src/examples/parser/number.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &primitive_type(PrimitiveType::Number)
        );
    }

    #[test]
    fn should_parse_string() {
        let schema = parse_from_file(Path::new("src/examples/parser/string.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &primitive_type(PrimitiveType::String)
        );
    }

    #[test]
    fn should_parse_array() {
        let schema = parse_from_file(Path::new("src/examples/parser/array.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &array_type(primitive_type(PrimitiveType::String))
        );
    }

    #[test]
    fn should_parse_nested_array() {
        let schema = parse_from_file(Path::new("src/examples/parser/array.nested.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &array_type(array_type(primitive_type(PrimitiveType::String)))
        );
    }

    #[test]
    fn should_parse_object_in_array() {
        let schema = parse_from_file(Path::new("src/examples/parser/array.object.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &array_type(object_type(
                String::from("src/examples/parser/array.object.schema.json/items"),
                vec![property(
                    String::from("subProperty"),
                    primitive_type(PrimitiveType::String),
                )]
            ))
        );
    }

    #[test]
    fn should_parse_object() {
        let schema = parse_from_file(Path::new("src/examples/parser/object.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &object_type(
                String::from("src/examples/parser/object.schema.json"),
                vec![property(
                    String::from("property"),
                    primitive_type(PrimitiveType::String),
                )]
            )
        );
    }

    #[test]
    fn should_parse_pattern_properties_to_map() {
        let schema = parse_from_file(Path::new(
            "src/examples/parser/object.pattern.properties.schema.json",
        ));

        assert_eq!(
            &schema.data_type as &DataType,
            &DataType::Map(Rc::new(primitive_type(PrimitiveType::Boolean)))
        );
    }

    #[test]
    fn should_use_title() {
        let schema = parse_from_file(Path::new("src/examples/parser/object.title.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &DataType::Object(Object {
                src: String::from("src/examples/parser/object.title.schema.json"),
                name: String::from("Some object"),
                properties: vec![property(
                    String::from("property"),
                    primitive_type(PrimitiveType::String),
                )],
            })
        );
    }

    #[test]
    fn should_use_property_name_as_fallback() {
        let schema = parse_from_file(Path::new(
            "src/examples/parser/object.nested.property.name.fallback.schema.json",
        ));

        assert_eq!(
            &schema.data_type as &DataType,
            &object_type(
                String::from("src/examples/parser/object.nested.property.name.fallback.schema.json"),
                vec![property(
                    String::from("someProperty"),
                    DataType::Object(Object {
                        src: String::from("src/examples/parser/object.nested.property.name.fallback.schema.json/properties/someProperty"),
                        name:String::from( "someProperty"),
                        properties: vec![property(
                            String::from("property"),
                            primitive_type(PrimitiveType::String),
                        )],
                    }),
                )]
            )
        );
    }

    #[test]
    fn should_make_property_required() {
        let schema = parse_from_file(Path::new("src/examples/parser/object.required.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &object_type(
                String::from("src/examples/parser/object.required.schema.json"),
                vec![ObjectProperty {
                    name: String::from("property"),
                    required: true,
                    data_type: Rc::new(primitive_type(PrimitiveType::String)),
                }]
            )
        );
    }

    #[test]
    fn should_read_defs() {
        let root = parse_from_file(Path::new("src/examples/parser/defs.schema.json"));
        check_defs(
            "src/examples/parser/defs.schema.json/$defs/referenced",
            root,
        );
    }

    #[test]
    fn should_read_definitions() {
        let root = parse_from_file(Path::new("src/examples/parser/definitions.schema.json"));
        check_defs(
            "src/examples/parser/definitions.schema.json/definitions/referenced",
            root,
        );
    }

    fn check_defs(src: &str, root: Root) {
        let mut definitions = HashMap::new();

        definitions.insert(
            String::from("referenced"),
            Rc::new(DataType::Object(Object {
                src: String::from(src),
                name: String::from("referenced"),
                properties: vec![property(
                    String::from("property"),
                    primitive_type(PrimitiveType::String),
                )],
            })),
        );

        assert_eq!(root.definitions, definitions);
    }

    #[test]
    fn should_parse_one_of() {
        let schema = parse_from_file(Path::new("src/examples/parser/oneof.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &one_of_type(generate_types(String::from(
                "src/examples/parser/oneof.schema.json/oneOf"
            )))
        );
    }

    #[test]
    fn should_parse_any_of() {
        let schema = parse_from_file(Path::new("src/examples/parser/anyof.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &any_of_type(generate_types(String::from(
                "src/examples/parser/anyof.schema.json/anyOf"
            )))
        );
    }

    #[test]
    fn should_parse_all_of() {
        let schema = parse_from_file(Path::new("src/examples/parser/allof.schema.json"));

        assert_eq!(
            &schema.data_type as &DataType,
            &all_of_type(generate_types(String::from(
                "src/examples/parser/allof.schema.json/allOf"
            )))
        );
    }

    fn generate_types(src: String) -> Vec<DataType> {
        vec![
            object_type(
                format!("{}/0", src),
                vec![property(
                    String::from("name"),
                    primitive_type(PrimitiveType::String),
                )],
            ),
            object_type(
                format!("{}/1", src),
                vec![property(
                    String::from("alias"),
                    primitive_type(PrimitiveType::String),
                )],
            ),
        ]
    }

    #[test]
    fn should_inherit_root_properties() {
        let schema = parse_from_file(Path::new(
            "src/examples/parser/oneof.inherit.properties.schema.json",
        ));

        assert_eq!(
            &schema.data_type as &DataType,
            &one_of_type(vec![
                DataType::Object(Object {
                    src: String::from(
                        "src/examples/parser/oneof.inherit.properties.schema.json/oneOf/0"
                    ),
                    name: String::from("Root title"),
                    properties: vec![ObjectProperty {
                        name: String::from("property"),
                        required: true,
                        data_type: Rc::new(primitive_type(PrimitiveType::String)),
                    }],
                }),
                DataType::PrimitiveType(PrimitiveType::String,)
            ])
        );
    }

    fn primitive_type(primitive_type: PrimitiveType) -> DataType {
        DataType::PrimitiveType(primitive_type)
    }

    fn object_type(src: String, properties: Vec<ObjectProperty>) -> DataType {
        DataType::Object(Object {
            src,
            name: String::from("Unknown"),
            properties,
        })
    }

    fn property(name: String, data_type: DataType) -> ObjectProperty {
        ObjectProperty {
            name,
            required: false,
            data_type: Rc::new(data_type),
        }
    }

    fn array_type(nested_type: DataType) -> DataType {
        DataType::Array(Rc::new(nested_type))
    }

    fn one_of_type(types: Vec<DataType>) -> DataType {
        DataType::OneOf(OneOf { types })
    }

    fn any_of_type(types: Vec<DataType>) -> DataType {
        DataType::AnyOf(AnyOf { types })
    }

    fn all_of_type(types: Vec<DataType>) -> DataType {
        DataType::AllOf(AllOf { types })
    }

    #[test]
    fn should_fallback_to_map_for_empty_objects() {
        let schema = parse_from_string(Path::new(""), "{\"type\": \"object\"}");

        assert_eq!(
            &schema.data_type as &DataType,
            &DataType::Map(Rc::new(DataType::Any))
        );
    }

    #[test]
    fn should_fallback_to_any() {
        let schema = parse_from_string(Path::new(""), "{}");

        assert_eq!(&schema.data_type as &DataType, &DataType::Any);
    }

    #[test]
    fn should_fallback_to_any_if_items_is_missing() {
        let schema = parse_from_string(Path::new(""), "{\"type\": \"array\"}");

        assert_eq!(
            &schema.data_type as &DataType,
            &DataType::Array(Rc::new(DataType::Any))
        );
    }
}
