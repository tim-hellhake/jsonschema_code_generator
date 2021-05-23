/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::parser::{
    parse_from_file, AllOf, AnyOf, DataType, Object, ObjectProperty, OneOf, PrimitiveType, Ref,
    Root,
};
use crate::resolver::{ResolveResult, Resolver};
use crate::sanitizer::{sanitize_property_name, sanitize_struct_name};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

#[derive(Eq, PartialEq, Debug)]
pub struct EntryWithPosition<T> {
    position: u64,
    payload: T,
}

impl<T: Eq> Ord for EntryWithPosition<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl<T: Eq> PartialOrd for EntryWithPosition<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Type {
    src: String,
    name: String,
    properties: Vec<Property>,
}

impl Type {
    pub fn serialize(&self) -> String {
        let mut result = String::from("");

        result.push_str(&format!("// from {}\n", self.src));
        result.push_str("#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]\n");
        result.push_str(&format!("struct {} {{\n", self.name));

        let properties: Vec<String> = self.properties.iter().map(|x| x.serialize()).collect();
        result.push_str(&properties.join(""));

        result.push_str("}");

        result
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Property {
    name: String,
    property_type: String,
    serde_options: SerdeOptions,
}

impl Property {
    pub fn serialize(&self) -> String {
        let mut result = String::from("");

        match &self.serde_options.rename {
            Some(rename) => {
                result.push_str(&format!("    #[serde(rename = \"{}\")]\n", rename));
            }
            None => {}
        }

        result.push_str(&format!("    pub {}: {},\n", self.name, self.property_type));

        result
    }
}

#[derive(Eq, PartialEq, Debug)]
struct SerdeOptions {
    rename: Option<String>,
}

pub struct Generator {
    resolver: Resolver,
    types: HashMap<String, EntryWithPosition<Type>>,
    next_position: u64,
    known_type_names: HashMap<String, String>,
}

impl Generator {
    pub fn new() -> Self {
        Generator {
            resolver: Resolver::new(),
            types: HashMap::new(),
            next_position: 0,
            known_type_names: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: &Path) -> String {
        match path.parent() {
            Some(base_path) => {
                let root = Rc::new(parse_from_file(path));
                self.add(
                    &base_path.display().to_string(),
                    root.clone(),
                    &root.data_type,
                )
            }
            None => panic!("'{}' has no parent", path.display()),
        }
    }

    pub fn add(&mut self, base_path: &String, root: Rc<Root>, data_type: &DataType) -> String {
        self.add_type(base_path, root, None, data_type, false, Vec::new())
    }

    fn add_object(
        &mut self,
        base_path: &String,
        root: Rc<Root>,
        src: String,
        Object {
            src: _,
            name,
            properties,
        }: &Object,
        visited_objects: Vec<String>,
    ) -> String {
        let cycle_detected = visited_objects.contains(&src);
        let mut visited_objects = visited_objects;

        if cycle_detected {
            visited_objects.clear();
        }

        let name = match self.known_type_names.get(&src) {
            Some(name) => name.clone(),
            None => match self.types.get(&src) {
                Some(EntryWithPosition {
                    position: _,
                    payload,
                }) => payload.name.clone(),
                None => {
                    let position = self.next_position;
                    self.next_position += 1;
                    let name = self.get_collision_free_name(sanitize_struct_name(name.clone()));
                    self.known_type_names.insert(src.clone(), name.clone());
                    visited_objects.push(src.clone());

                    let mut new_properties = Vec::new();

                    for property in properties as &Vec<ObjectProperty> {
                        new_properties.push(self.create_property(
                            base_path,
                            root.clone(),
                            &property,
                            visited_objects.clone(),
                        ));
                    }

                    let new_type = Type {
                        src: src.clone(),
                        name: name.clone(),
                        properties: new_properties,
                    };

                    self.types.insert(
                        src,
                        EntryWithPosition {
                            position,
                            payload: new_type,
                        },
                    );

                    name
                }
            },
        };

        match cycle_detected {
            true => format!("Box<{}>", name),
            false => name,
        }
    }

    fn get_collision_free_name(&self, name: String) -> String {
        let mut counter = 1;
        let mut new_name = name.clone();

        while self.known_type_names.values().any(|val| val == &new_name) {
            new_name = format!("{}{}", name, counter);
            counter += 1;
        }

        new_name
    }

    fn create_property(
        &mut self,
        base_path: &String,
        root: Rc<Root>,
        ObjectProperty {
            name,
            required,
            data_type,
        }: &ObjectProperty,
        visited_objects: Vec<String>,
    ) -> Property {
        let property_name = sanitize_property_name(name.clone());

        let rename = if name == &property_name {
            None
        } else {
            Some(name.clone())
        };

        Property {
            name: property_name,
            property_type: self.add_type(
                base_path,
                root,
                None,
                &*data_type,
                required.clone(),
                visited_objects,
            ),
            serde_options: SerdeOptions { rename },
        }
    }

    fn add_type(
        &mut self,
        base_path: &String,
        root: Rc<Root>,
        src_override: Option<String>,
        data_type: &DataType,
        required: bool,
        visited_objects: Vec<String>,
    ) -> String {
        match data_type {
            DataType::PrimitiveType(primitive_type) => {
                let type_name = match primitive_type {
                    PrimitiveType::Null => "Value",
                    PrimitiveType::Boolean => "bool",
                    PrimitiveType::Integer => "i64",
                    PrimitiveType::Number => "f64",
                    PrimitiveType::String => "String",
                };

                match required {
                    true => String::from(type_name),
                    false => format!("Option<{}>", type_name),
                }
            }
            DataType::Array(items) => {
                let type_name =
                    self.add_type(base_path, root, src_override, &*items, true, Vec::new());
                format!("Vec<{}>", type_name)
            }
            DataType::Object(object) => {
                let type_name = self.add_object(
                    base_path,
                    root,
                    src_override.unwrap_or(object.src.to_string()),
                    object.clone(),
                    visited_objects,
                );

                match required {
                    true => String::from(type_name),
                    false => format!("Option<{}>", type_name),
                }
            }
            DataType::Map(data_type) => {
                format!(
                    "BTreeMap<String, {}>",
                    self.add_type(base_path, root, None, data_type, true, Vec::new(),)
                )
            }
            DataType::Ref(Ref { ref_path }) => {
                let ResolveResult {
                    root,
                    path,
                    data_type,
                } = self.resolver.resolve(root, ref_path.clone());
                let file = root.file.display().to_string();

                let src = match path {
                    Some(path) => format!("{}#{}", file, path),
                    None => file,
                };

                self.add_type(
                    &base_path,
                    root,
                    Some(src),
                    &data_type,
                    required,
                    visited_objects,
                )
            }
            DataType::OneOf(OneOf { types }) => {
                for data_type in types {
                    self.add(base_path, root.clone(), data_type.clone());
                }

                String::from("Value")
            }
            DataType::AnyOf(AnyOf { types }) => {
                for data_type in types {
                    self.add(base_path, root.clone(), data_type.clone());
                }

                String::from("Value")
            }
            DataType::AllOf(AllOf { types }) => {
                for data_type in types {
                    self.add(base_path, root.clone(), data_type.clone());
                }

                String::from("Value")
            }
            DataType::Any => String::from("Value"),
        }
    }

    pub fn serialize(&self) -> String {
        let mut result = String::from("");

        result.push_str("use serde_json::Value;\n");
        result.push_str("use std::collections::BTreeMap;\n");

        let mut types: Vec<&EntryWithPosition<Type>> = self.types.values().collect();
        types.sort();
        let types: Vec<String> = types.into_iter().map(|x| x.payload.serialize()).collect();
        result.push_str(&types.join("\n\n"));
        result.push_str("\n");

        result
    }
}

#[cfg(test)]
mod generator_tests {
    use crate::generator::{EntryWithPosition, Generator, Property, SerdeOptions, Type};
    use crate::parser::{
        AllOf, AnyOf, DataType, Object, ObjectProperty, OneOf, PrimitiveType, Ref, Root,
    };
    use std::collections::HashMap;
    use std::path::Path;
    use std::rc::Rc;

    #[test]
    fn should_be_ordered_by_position() {
        let mut list = vec![
            EntryWithPosition {
                payload: String::from("a"),
                position: 3,
            },
            EntryWithPosition {
                payload: String::from("b"),
                position: 1,
            },
            EntryWithPosition {
                payload: String::from("c"),
                position: 2,
            },
        ];

        list.sort();

        assert_eq!(
            list,
            vec![
                EntryWithPosition {
                    payload: String::from("b"),
                    position: 1,
                },
                EntryWithPosition {
                    payload: String::from("c"),
                    position: 2,
                },
                EntryWithPosition {
                    payload: String::from("a"),
                    position: 3,
                },
            ]
        );
    }

    #[test]
    fn should_serialize_type_with_derive() {
        let t = Type {
            src: String::from(""),
            name: String::from(""),
            properties: Vec::new(),
        };

        assert_eq!(
            t.serialize()
                .contains("#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]"),
            true
        )
    }

    #[test]
    fn should_serialize_type_with_src() {
        let t = Type {
            src: String::from("foo"),
            name: String::from(""),
            properties: Vec::new(),
        };

        assert_eq!(t.serialize().contains("from foo"), true)
    }

    #[test]
    fn should_serialize_type_with_struct() {
        let t = Type {
            src: String::from(""),
            name: String::from("Foo"),
            properties: Vec::new(),
        };

        assert_eq!(t.serialize().contains("struct Foo"), true)
    }

    #[test]
    fn should_serialize_type_with_properties() {
        let t = Type {
            src: String::from(""),
            name: String::from(""),
            properties: vec![Property {
                name: String::from("foo"),
                property_type: String::from("Obsidian"),
                serde_options: SerdeOptions { rename: None },
            }],
        };

        assert_eq!(t.serialize().contains("foo: Obsidian"), true)
    }

    #[test]
    fn should_serialize_property_with_name() {
        let property = Property {
            name: String::from("foo"),
            property_type: String::from("Obsidian"),
            serde_options: SerdeOptions { rename: None },
        };

        assert_eq!(property.serialize().contains("pub foo: Obsidian"), true)
    }

    #[test]
    fn should_serialize_property_with_rename() {
        let property = Property {
            name: String::from(""),
            property_type: String::from(""),
            serde_options: SerdeOptions {
                rename: Some(String::from("foo")),
            },
        };

        assert_eq!(
            property.serialize().contains("#[serde(rename = \"foo\")]"),
            true
        )
    }

    #[test]
    fn should_serialize_with_serde_json_import() {
        let generator = Generator::new();

        assert_eq!(
            generator.serialize().contains("use serde_json::Value"),
            true
        )
    }

    #[test]
    fn should_serialize_with_btree_import() {
        let generator = Generator::new();

        assert_eq!(
            generator
                .serialize()
                .contains("use std::collections::BTreeMap"),
            true
        )
    }

    #[test]
    fn should_add_object() {
        let mut generator = Generator::new();

        let type_name = add_object(&mut generator);

        assert_eq!(type_name, "AwesomeFoo");

        assert_eq!(
            generator.types.get("correct src"),
            Some(&EntryWithPosition {
                position: 0,
                payload: Type {
                    src: String::from("correct src"),
                    name: String::from("AwesomeFoo"),
                    properties: vec![Property {
                        name: String::from("awesome_property"),
                        property_type: String::from("Value"),
                        serde_options: SerdeOptions {
                            rename: Some(String::from("awesome property"))
                        },
                    }]
                }
            })
        )
    }

    #[test]
    fn should_add_known_type() {
        let mut generator = Generator::new();

        add_object(&mut generator);

        assert_eq!(
            generator.known_type_names.get("correct src"),
            Some(&String::from("AwesomeFoo"))
        );
    }

    #[test]
    fn should_detect_type_cycles() {
        let mut generator = Generator::new();
        generator
            .known_type_names
            .insert(String::from("correct src"), String::from("some type"));

        let type_name = add_object(&mut generator);

        assert_eq!(type_name, "some type");

        assert_eq!(generator.types.len(), 0)
    }

    #[test]
    fn should_detect_reference_cycles() {
        let mut generator = Generator::new();

        let type_name = generator.add_object(
            &String::from(""),
            Rc::new(Root {
                file: Path::new("").to_path_buf(),
                data_type: Rc::new(DataType::Any),
                definitions: HashMap::new(),
            }),
            String::from("correct src"),
            &object_with_property(),
            vec![String::from("correct src")],
        );

        assert_eq!(type_name, "Box<AwesomeFoo>");

        assert_eq!(
            generator.known_type_names.get("correct src"),
            Some(&String::from("AwesomeFoo"))
        );
    }

    #[test]
    fn should_not_add_the_same_type_twice() {
        let mut generator = Generator::new();

        let type_name = add_object(&mut generator);
        assert_eq!(type_name, "AwesomeFoo");

        let type_name = add_object(&mut generator);
        assert_eq!(type_name, "AwesomeFoo");

        assert_eq!(generator.types.len(), 1);

        assert_eq!(generator.known_type_names.len(), 1);
    }

    #[test]
    fn should_add_types_in_the_correct_order() {
        let mut generator = Generator::new();

        generator.add_object(
            &String::from(""),
            Rc::new(Root {
                file: Path::new("").to_path_buf(),
                data_type: Rc::new(DataType::Any),
                definitions: HashMap::new(),
            }),
            String::from("correct src"),
            &Object {
                src: String::from("wrong src"),
                name: String::from("awesome foo"),
                properties: vec![ObjectProperty {
                    name: String::from("awesome property"),
                    required: false,
                    data_type: Rc::new(DataType::Object(Object {
                        src: String::from("nested src"),
                        name: String::from("awesome foo part 2"),
                        properties: vec![ObjectProperty {
                            name: String::from("awesome property part 2"),
                            required: false,
                            data_type: Rc::new(DataType::Any),
                        }],
                    })),
                }],
            },
            Vec::new(),
        );

        assert_eq!(
            generator.types.get("correct src").map(|x| x.position),
            Some(0)
        );

        assert_eq!(
            generator.types.get("nested src").map(|x| x.position),
            Some(1)
        );
    }

    fn add_object(generator: &mut Generator) -> String {
        generator.add_object(
            &String::from(""),
            Rc::new(Root {
                file: Path::new("").to_path_buf(),
                data_type: Rc::new(DataType::Any),
                definitions: HashMap::new(),
            }),
            String::from("correct src"),
            &object_with_property(),
            Vec::new(),
        )
    }

    fn object_with_property() -> Object {
        Object {
            src: String::from("wrong src"),
            name: String::from("awesome foo"),
            properties: vec![ObjectProperty {
                name: String::from("awesome property"),
                required: false,
                data_type: Rc::new(DataType::Any),
            }],
        }
    }

    #[test]
    fn should_add_null_type() {
        let mut generator = Generator::new();

        assert_eq!(
            add_primitive_type(&mut generator, PrimitiveType::Null, true),
            String::from("Value")
        );
    }

    #[test]
    fn should_add_bool_type() {
        let mut generator = Generator::new();

        assert_eq!(
            add_primitive_type(&mut generator, PrimitiveType::Boolean, true),
            String::from("bool")
        );
    }

    #[test]
    fn should_add_integer_type() {
        let mut generator = Generator::new();

        assert_eq!(
            add_primitive_type(&mut generator, PrimitiveType::Integer, true),
            String::from("i64")
        );
    }

    #[test]
    fn should_add_number_type() {
        let mut generator = Generator::new();

        assert_eq!(
            add_primitive_type(&mut generator, PrimitiveType::Number, true),
            String::from("f64")
        );
    }

    #[test]
    fn should_add_string_type() {
        let mut generator = Generator::new();

        assert_eq!(
            add_primitive_type(&mut generator, PrimitiveType::String, true),
            String::from("String")
        );
    }

    #[test]
    fn should_add_optional_string_type() {
        let mut generator = Generator::new();

        assert_eq!(
            add_primitive_type(&mut generator, PrimitiveType::String, false),
            String::from("Option<String>")
        );
    }

    fn add_primitive_type(
        generator: &mut Generator,
        primitive_type: PrimitiveType,
        required: bool,
    ) -> String {
        add_type(generator, DataType::PrimitiveType(primitive_type), required)
    }

    #[test]
    fn should_add_array_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::Array(Rc::new(DataType::Any)),
            true,
        );

        assert_eq!(type_name, "Vec<Value>");
    }

    #[test]
    fn should_add_object_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::Object(object_with_property()),
            true,
        );

        assert_eq!(type_name, "AwesomeFoo");
    }

    #[test]
    fn should_add_optional_object_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::Object(object_with_property()),
            false,
        );

        assert_eq!(type_name, "Option<AwesomeFoo>");
    }

    #[test]
    fn should_add_map_type() {
        let mut generator = Generator::new();

        let type_name = add_type(&mut generator, DataType::Map(Rc::new(DataType::Any)), true);

        assert_eq!(type_name, "BTreeMap<String, Value>");
    }

    #[test]
    fn should_add_ref_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::Ref(Ref {
                ref_path: String::from("#/$defs/foo"),
            }),
            true,
        );

        assert_eq!(type_name, "AwesomeFoo");
    }

    #[test]
    fn should_add_optional_ref_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::Ref(Ref {
                ref_path: String::from("#/$defs/foo"),
            }),
            false,
        );

        assert_eq!(type_name, "Option<AwesomeFoo>");
    }

    #[test]
    fn should_add_one_of_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::OneOf(OneOf {
                types: vec![DataType::Any],
            }),
            true,
        );

        assert_eq!(type_name, "Value");
    }

    #[test]
    fn should_add_any_of_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::AnyOf(AnyOf {
                types: vec![DataType::Any],
            }),
            true,
        );

        assert_eq!(type_name, "Value");
    }

    #[test]
    fn should_add_all_of_type() {
        let mut generator = Generator::new();

        let type_name = add_type(
            &mut generator,
            DataType::AllOf(AllOf {
                types: vec![DataType::Any],
            }),
            true,
        );

        assert_eq!(type_name, "Value");
    }

    #[test]
    fn should_add_any_type() {
        let mut generator = Generator::new();

        let type_name = add_type(&mut generator, DataType::Any, true);

        assert_eq!(type_name, "Value");
    }

    #[test]
    fn should_detect_loops() {
        let file = "src/examples/generator/loop1.schema.json";

        let mut generator = Generator::new();
        generator.add_file(Path::new(file));

        let mut types: Vec<EntryWithPosition<Type>> = generator
            .types
            .into_iter()
            .map(|(_, value)| value)
            .collect();

        types.sort();

        let types: Vec<Type> = types.into_iter().map(|x| x.payload).collect();

        assert_eq!(
            types,
            vec![
                Type {
                    src: String::from("src/examples/generator/loop1.schema.json"),
                    name: String::from("Loop"),
                    properties: vec![Property {
                        name: String::from("a"),
                        serde_options: SerdeOptions { rename: None },
                        property_type: String::from("Option<B>")
                    }]
                },
                Type {
                    src: String::from("src/examples/generator/loop1.schema.json#/definitions/b"),
                    name: String::from("B"),
                    properties: vec![Property {
                        name: String::from("c"),
                        serde_options: SerdeOptions { rename: None },
                        property_type: String::from("Option<C>")
                    }]
                },
                Type {
                    src: String::from("src/examples/generator/loop2.schema.json#/definitions/c"),
                    name: String::from("C"),
                    properties: vec![Property {
                        name: String::from("b"),
                        serde_options: SerdeOptions { rename: None },
                        property_type: String::from("Option<Box<B>>")
                    }]
                }
            ]
        );
    }

    #[test]
    fn should_create_referenced_types_once() {
        let file = "src/examples/generator/reference.twice.schema.json";

        let mut generator = Generator::new();
        generator.add_file(Path::new(file));

        let mut types: Vec<EntryWithPosition<Type>> = generator
            .types
            .into_iter()
            .map(|(_, value)| value)
            .collect();

        types.sort();

        let types: Vec<Type> = types.into_iter().map(|x| x.payload).collect();

        assert_eq!(
            types,
            vec![
                Type {
                    src: String::from(file),
                    name: String::from("Twice"),
                    properties: vec![
                        Property {
                            name: String::from("a"),
                            serde_options: SerdeOptions { rename: None },
                            property_type: String::from("Option<C>")
                        },
                        Property {
                            name: String::from("b"),
                            serde_options: SerdeOptions { rename: None },
                            property_type: String::from("Option<C>")
                        }
                    ]
                },
                Type {
                    src: String::from(format!("{}#/definitions/c", file)),
                    name: String::from("C"),
                    properties: vec![Property {
                        name: String::from("foo"),
                        serde_options: SerdeOptions { rename: None },
                        property_type: String::from("Value")
                    }]
                }
            ]
        );
    }

    #[test]
    fn should_prevent_name_collisions() {
        let file = "src/examples/generator/name.collision.schema.json";

        let mut generator = Generator::new();
        generator.add_file(Path::new(file));

        let mut types: Vec<EntryWithPosition<Type>> = generator
            .types
            .into_iter()
            .map(|(_, value)| value)
            .collect();

        types.sort();

        let types: Vec<Type> = types.into_iter().map(|x| x.payload).collect();

        assert_eq!(
            types,
            vec![
                Type {
                    src: String::from(file),
                    name: String::from("Collision"),
                    properties: vec![
                        Property {
                            name: String::from("a"),
                            serde_options: SerdeOptions { rename: None },
                            property_type: String::from("Option<A>")
                        },
                        Property {
                            name: String::from("b"),
                            serde_options: SerdeOptions { rename: None },
                            property_type: String::from("Option<A1>")
                        },
                        Property {
                            name: String::from("c"),
                            serde_options: SerdeOptions { rename: None },
                            property_type: String::from("Option<A2>")
                        }
                    ]
                },
                Type {
                    src: String::from(format!("{}/properties/a", file)),
                    name: String::from("A"),
                    properties: vec![Property {
                        name: String::from("foo"),
                        serde_options: SerdeOptions { rename: None },
                        property_type: String::from("Value")
                    }]
                },
                Type {
                    src: String::from(format!("{}/properties/b", file)),
                    name: String::from("A1"),
                    properties: vec![Property {
                        name: String::from("foo"),
                        serde_options: SerdeOptions { rename: None },
                        property_type: String::from("Value")
                    }]
                },
                Type {
                    src: String::from(format!("{}/properties/c", file)),
                    name: String::from("A2"),
                    properties: vec![Property {
                        name: String::from("foo"),
                        serde_options: SerdeOptions { rename: None },
                        property_type: String::from("Value")
                    }]
                }
            ]
        );
    }

    fn add_type(generator: &mut Generator, data_type: DataType, required: bool) -> String {
        let mut definitions = HashMap::new();

        definitions.insert(
            String::from("foo"),
            Rc::new(DataType::Object(object_with_property())),
        );

        generator.add_type(
            &String::from(""),
            Rc::new(Root {
                file: Path::new("").to_path_buf(),
                data_type: Rc::new(DataType::Any),
                definitions,
            }),
            Some(String::from("")),
            &data_type,
            required,
            Vec::new(),
        )
    }
}
