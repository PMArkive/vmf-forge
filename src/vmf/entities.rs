//! This module provides structures for representing entities in a VMF file.

use crate::{
    errors::{VmfError, VmfResult},
    VmfBlock, VmfSerializable,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    vec,
};

use super::common::Editor;
use super::world::Solid;

/// Represents an entity in a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// The key-value pairs associated with this entity.
    pub key_values: IndexMap<String, String>,
    /// The output connections of this entity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<(String, String)>>,
    /// The solids associated with this entity, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solids: Option<Vec<Solid>>,
    /// The editor data for this entity.
    pub editor: Editor,
}

impl TryFrom<VmfBlock> for Entity {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        // Extract key-value pairs from the block
        let key_values = block.key_values;

        // Searches for nested blocks and extracts the necessary information
        let mut ent = Self {
            key_values,
            ..Default::default()
        };
        let mut solids = Vec::new();

        for block in block.blocks {
            match block.name.as_str() {
                "editor" => ent.editor = Editor::try_from(block)?,
                "connections" => ent.connections = process_connections(block.key_values),
                "solid" => solids.push(Solid::try_from(block)?),
                "hidden" => {
                    if let Some(hidden_block) = block.blocks.first() {
                        solids.push(Solid::try_from(hidden_block.to_owned())?)
                    }
                }
                _ => {
                    #[cfg(feature = "debug_assert_info")]
                    debug_assert!(
                        false,
                        "Unexpected block name: {}, id: {:?}",
                        block.name,
                        ent.key_values.get("id")
                    );
                }
            }
        }

        if !solids.is_empty() {
            ent.solids = Some(solids);
        }

        Ok(ent)
    }
}

impl From<Entity> for VmfBlock {
    fn from(val: Entity) -> Self {
        let editor = val.editor.into();

        VmfBlock {
            name: "entity".to_string(),
            key_values: val.key_values,
            blocks: vec![editor],
        }
    }
}

impl VmfSerializable for Entity {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::with_capacity(256);

        // Writes the main entity block
        output.push_str(&format!("{0}entity\n{0}{{\n", indent));

        // Adds key_values of the main block
        for (key, value) in &self.key_values {
            output.push_str(&format!("{}\t\"{}\" \"{}\"\n", indent, key, value));
        }

        // Adds connections block
        if let Some(connections) = &self.connections {
            output.push_str(&format!("{0}\tconnections\n{0}\t{{\n", indent));
            for (out, inp) in connections {
                output.push_str(&format!("{}\t\t\"{}\" \"{}\"\n", indent, out, inp));
            }
            output.push_str(&format!("{}\t}}\n", indent));
        }

        // Solids block
        if let Some(solids) = &self.solids {
            for solid in solids {
                output.push_str(&solid.to_vmf_string(indent_level + 1));
            }
        }

        // Editor block
        output.push_str(&self.editor.to_vmf_string(indent_level + 1));

        output.push_str(&format!("{}}}\n", indent));

        output
    }
}

/// Represents a collection of entities in a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Entities {
    /// The vector of entities.
    pub vec: Vec<Entity>,
}

impl Deref for Entities {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl DerefMut for Entities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl Entities {
    /// Returns an iterator over the entities that have the specified key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to search for.
    /// * `value` - The value to search for.
    pub fn find_by_keyvalue<'a>(
        &'a self,
        key: &'a str,
        value: &'a str,
    ) -> impl Iterator<Item = &'a Entity> + 'a {
        self.vec
            .iter()
            .filter(move |ent| ent.key_values.get(key).is_some_and(|v| v == value))
    }

    /// Returns an iterator over the entities that have the specified key-value pair, allowing modification.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to search for.
    /// * `value` - The value to search for.
    pub fn find_by_keyvalue_mut<'a>(
        &'a mut self,
        key: &'a str,
        value: &'a str,
    ) -> impl Iterator<Item = &'a mut Entity> + 'a {
        self.vec
            .iter_mut()
            .filter(move |ent| ent.key_values.get(key).is_some_and(|v| v == value))
    }

    /// Returns an iterator over the entities with the specified classname.
    ///
    /// # Arguments
    ///
    /// * `classname` - The classname to search for.
    pub fn find_by_classname<'a>(
        &'a self,
        classname: &'a str,
    ) -> impl Iterator<Item = &'a Entity> + 'a {
        self.find_by_keyvalue("classname", classname)
    }

    /// Returns an iterator over the entities with the specified targetname.
    ///
    /// # Arguments
    ///
    /// * `name` - The targetname to search for.
    pub fn find_by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Entity> + 'a {
        self.find_by_keyvalue("targetname", name)
    }

    /// Returns an iterator over the entities with the specified classname, allowing modification.
    ///
    /// # Arguments
    ///
    /// * `classname` - The classname to search for.
    pub fn find_by_classname_mut<'a>(
        &'a mut self,
        classname: &'a str,
    ) -> impl Iterator<Item = &'a mut Entity> + 'a {
        self.find_by_keyvalue_mut("classname", classname)
    }

    /// Returns an iterator over the entities with the specified targetname, allowing modification.
    ///
    /// # Arguments
    ///
    /// * `name` - The targetname to search for.
    pub fn find_by_name_mut<'a>(
        &'a mut self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a mut Entity> + 'a {
        self.find_by_keyvalue_mut("targetname", name)
    }

    /// Returns an iterator over the entities with the specified model.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to search for.
    pub fn find_by_model<'a>(&'a self, model: &'a str) -> impl Iterator<Item = &'a Entity> + 'a {
        self.find_by_keyvalue("model", model)
    }

    /// Returns an iterator over the entities with the specified model, allowing modification.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to search for.
    pub fn find_by_model_mut<'a>(
        &'a mut self,
        model: &'a str,
    ) -> impl Iterator<Item = &'a mut Entity> + 'a {
        self.find_by_keyvalue_mut("model", model)
    }
}

// utils func
fn process_connections(map: IndexMap<String, String>) -> Option<Vec<(String, String)>> {
    if map.is_empty() {
        return None;
    }

    let result = map
        .iter()
        .flat_map(|(key, value)| {
            value
                .split('\r')
                .map(move |part| (key.clone(), part.to_string()))
        })
        .collect();

    Some(result)
}
