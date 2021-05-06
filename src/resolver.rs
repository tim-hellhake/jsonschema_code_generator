/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::parser::{parse_from_file, DataType, Root};
use crate::ref_parser::{parse_ref, RefPath};

#[derive(PartialEq, Debug)]
pub struct ResolveResult {
    pub root: Rc<Root>,
    pub path: Option<String>,
    pub data_type: Rc<DataType>,
}

pub struct Resolver {
    cache: HashMap<String, Rc<Root>>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            cache: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, root: Rc<Root>, ref_path: String) -> ResolveResult {
        let RefPath { file, path } = parse_ref(ref_path.clone());

        let file = match file {
            Some(file) => match root.file.parent() {
                Some(base_path) => Some(Path::join(Path::new(base_path), Path::new(&file))),
                None => panic!("'{}' has no parent", root.file.display()),
            },
            None => None,
        };

        let root = match &file {
            Some(file) => match self.cache.get(&file.display().to_string()) {
                Some(root) => root.clone(),
                None => self.load(file),
            },
            None => root,
        };

        let data_type = match &path {
            Some(path) => Resolver::deref(path.clone(), &root.definitions),
            None => root.data_type.clone(),
        };

        ResolveResult {
            root,
            path,
            data_type,
        }
    }

    fn load(&mut self, file: &Path) -> Rc<Root> {
        let root = parse_from_file(file);
        let rc = Rc::new(root);
        self.cache.insert(file.display().to_string(), rc.clone());
        rc
    }

    fn deref(path: String, root_definitions: &HashMap<String, Rc<DataType>>) -> Rc<DataType> {
        let parts: Vec<&str> = path
            .split("/")
            .into_iter()
            .filter(|x| x.len() > 0)
            .collect();

        match parts.len() {
            0 => panic!("Cannot resolve empty ref {}", path),
            2 => {
                if parts[0] != "definitions" && parts[0] != "$defs" {
                    panic!("Ref path should begin with #/definitions or #/$defs")
                }

                match root_definitions.get(parts[1]) {
                    Some(data_type) => data_type.clone(),
                    None => {
                        panic!("No local definition for {} found", path);
                    }
                }
            }
            _ => panic!("Invalid ref {}", path),
        }
    }
}

#[cfg(test)]
mod resolver_tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::rc::Rc;

    use crate::parser::{DataType, Object, ObjectProperty, PrimitiveType, Root};
    use crate::resolver::{ResolveResult, Resolver};

    #[test]
    fn should_resolve_local_definition() {
        let mut resolver = Resolver::new();
        let referenced_value = Rc::new(DataType::Any);
        let mut definitions = HashMap::new();
        definitions.insert(String::from("foo"), referenced_value.clone());

        let root = Rc::new(Root {
            file: Path::new("does not exist").to_path_buf(),
            data_type: Rc::new(DataType::Any),
            definitions,
        });

        assert_eq!(
            resolver.resolve(root.clone(), String::from("#/definitions/foo")),
            ResolveResult {
                root,
                data_type: referenced_value,
                path: Some(String::from("/definitions/foo")),
            }
        );
    }

    #[test]
    fn should_resolve_file_definition() {
        let mut resolver = Resolver::new();
        let referenced_value = Rc::new(DataType::PrimitiveType(PrimitiveType::Integer));

        let root = Rc::new(Root {
            file: Path::new("src/examples/resolver/only-here-for-the-base-dir").to_path_buf(),
            data_type: Rc::new(DataType::Any),
            definitions: HashMap::new(),
        });

        let mut definitions = HashMap::new();
        definitions.insert(String::from("foo"), referenced_value.clone());

        let new_root = Rc::new(Root {
            file: Path::new("src/examples/resolver/definitions.json").to_path_buf(),
            data_type: Rc::new(create_root_object()),
            definitions,
        });

        assert_eq!(
            resolver.resolve(
                root.clone(),
                String::from("definitions.json#/definitions/foo"),
            ),
            ResolveResult {
                root: new_root,
                data_type: referenced_value,
                path: Some(String::from("/definitions/foo")),
            }
        );
    }

    #[test]
    fn should_resolve_file() {
        let mut resolver = Resolver::new();

        let root = Rc::new(Root {
            file: Path::new("src/examples/resolver/only-here-for-the-base-dir").to_path_buf(),
            data_type: Rc::new(DataType::Any),
            definitions: HashMap::new(),
        });

        let root_object = Rc::new(create_root_object());

        let mut definitions = HashMap::new();
        definitions.insert(
            String::from("foo"),
            Rc::new(DataType::PrimitiveType(PrimitiveType::Integer)),
        );

        let new_root = Rc::new(Root {
            file: Path::new("src/examples/resolver/definitions.json").to_path_buf(),
            data_type: root_object.clone(),
            definitions,
        });

        assert_eq!(
            resolver.resolve(root.clone(), String::from("definitions.json")),
            ResolveResult {
                root: new_root,
                data_type: root_object,
                path: None,
            }
        );
    }

    fn create_root_object() -> DataType {
        DataType::Object(Object {
            src: String::from("src/examples/resolver/definitions.json"),
            name: String::from("r00t"),
            properties: vec![ObjectProperty {
                name: String::from("foo"),
                required: false,
                data_type: Rc::new(DataType::PrimitiveType(PrimitiveType::String)),
            }],
        })
    }

    #[test]
    fn should_resolve_root_on_empty_path() {
        let mut resolver = Resolver::new();
        let root_type = Rc::new(DataType::Any);

        let root = Rc::new(Root {
            file: Path::new("does not exist").to_path_buf(),
            data_type: root_type.clone(),
            definitions: HashMap::new(),
        });

        assert_eq!(
            resolver.resolve(root.clone(), String::from("")),
            ResolveResult {
                root,
                data_type: root_type,
                path: None,
            }
        );
    }
}
