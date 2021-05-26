/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[derive(Eq, PartialEq, Debug)]
pub struct GeneratedType {
    pub src: String,
    pub name: String,
    pub properties: Vec<GeneratedProperty>,
}

impl GeneratedType {
    pub fn serialize(&self) -> String {
        let mut result = String::from("");

        result.push_str(&format!("#[doc = \"Generated from {}\"]\n", self.src));
        result.push_str("#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]\n");
        result.push_str(&format!("struct {} {{\n", self.name));

        let properties: Vec<String> = self.properties.iter().map(|x| x.serialize()).collect();
        result.push_str(&properties.join(""));

        result.push_str("}");

        result
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct GeneratedProperty {
    pub name: String,
    pub property_type: String,
    pub serde_options: SerdeOptions,
}

impl GeneratedProperty {
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
pub struct SerdeOptions {
    pub rename: Option<String>,
}
