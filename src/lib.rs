//! A library for parsing and manipulating Valve Map Format (VMF) files.
//!
//! This library provides functionality to parse VMF files used in Source Engine games
//! into Rust data structures, modify the data, and serialize it back into a VMF file.
//!
//! # Example
//!
//! ```
//! use vmf_forge::prelude::*;
//! use std::fs::File;
//!
//! fn main() -> Result<(), VmfError> {
//!     let mut file = File::open("your_map.vmf")?;
//!     let vmf_file = VmfFile::parse_file(&mut file)?;
//!
//!     println!("Map Version: {}", vmf_file.versioninfo.map_version);
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub mod parser;
pub(crate) mod utils;
pub mod vmf;

pub mod errors;
pub mod prelude;

pub use errors::{VmfError, VmfResult};
pub use vmf::entities::{Entities, Entity};
pub use vmf::metadata::{VersionInfo, ViewSettings, VisGroups};
pub use vmf::regions::{Cameras, Cordons};
pub use vmf::world::World;

/// A trait for types that can be serialized into a VMF string representation.
pub trait VmfSerializable {
    /// Serializes the object into a VMF string.
    ///
    /// # Arguments
    ///
    /// * `indent_level` - The indentation level to use for formatting.
    ///
    /// # Returns
    ///
    /// A string representation of the object in VMF format.
    fn to_vmf_string(&self, indent_level: usize) -> String;
}

/// Represents a parsed VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VmfFile {
    /// The path to the VMF file, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The version info of the VMF file.
    pub versioninfo: VersionInfo,
    /// The visgroups in the VMF file.
    pub visgroups: VisGroups,
    /// The view settings in the VMF file.
    pub viewsettings: ViewSettings,
    /// The world data in the VMF file.
    pub world: World,
    /// The entities in the VMF file.
    pub entities: Entities,
    /// The hidden entities in the VMF file.
    pub hiddens: Entities,
    /// The camera data in the VMF file.
    pub cameras: Cameras,
    /// The cordon data in the VMF file.
    pub cordons: Cordons,
}

impl VmfFile {
    /// Parses a VMF file from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - The string content of the VMF file.
    ///
    /// # Returns
    ///
    /// A `VmfResult` containing the parsed `VmfFile` or a `VmfError` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use vmf_forge::VmfFile;
    ///
    /// let vmf_content = r#"
    /// versioninfo
    /// {
    ///     "editorversion" "400"
    ///     "editorbuild" "8000"
    ///     "mapversion" "1"
    ///     "formatversion" "100"
    ///     "prefab" "0"
    /// }
    /// "#;
    ///
    /// let vmf_file = VmfFile::parse(vmf_content);
    /// assert!(vmf_file.is_ok());
    /// ```
    pub fn parse(content: &str) -> VmfResult<Self> {
        Ok(parser::parse_vmf(content)?)
    }

    /// Parses a VMF file from a `File`.
    ///
    /// # Arguments
    ///
    /// * `file` - The `File` to read from.
    ///
    /// # Returns
    ///
    /// A `VmfResult` containing the parsed `VmfFile` or a `VmfError` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vmf_forge::VmfFile;
    /// use std::fs::File;
    ///
    /// let mut file = File::open("your_map.vmf").unwrap();
    /// let vmf_file = VmfFile::parse_file(&mut file);
    /// assert!(vmf_file.is_ok());
    /// ```
    pub fn parse_file(file: &mut File) -> VmfResult<Self> {
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let content = String::from_utf8_lossy(&content);

        VmfFile::parse(&content)
    }

    /// Opens and parses a VMF file from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the VMF file.
    ///
    /// # Returns
    ///
    /// A `VmfResult` containing the parsed `VmfFile` or a `VmfError` if an error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vmf_forge::VmfFile;
    ///
    /// let vmf_file = VmfFile::open("your_map.vmf");
    /// assert!(vmf_file.is_ok());
    /// ```
    pub fn open(path: impl AsRef<Path>) -> VmfResult<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let content = String::from_utf8_lossy(&content);

        let mut vmf_file = VmfFile::parse(&content)?;
        vmf_file.path = Some(path_str);
        Ok(vmf_file)
    }

    /// Saves the `VmfFile` to a file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save the VMF file to.
    ///
    /// # Returns
    ///
    /// A `VmfResult` indicating success or a `VmfError` if an error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vmf_forge::VmfFile;
    ///
    /// let vmf_file = VmfFile::open("your_map.vmf").unwrap();
    /// let result = vmf_file.save("new_map.vmf");
    /// assert!(result.is_ok());
    /// ```
    pub fn save(&self, path: impl AsRef<Path>) -> VmfResult<()> {
        let mut file = File::create(path)?;
        file.write_all(self.to_vmf_string().as_bytes())?;
        Ok(())
    }

    /// Converts the `VmfFile` to a string in VMF format.
    ///
    /// # Returns
    ///
    /// A string representing the `VmfFile` in VMF format.
    pub fn to_vmf_string(&self) -> String {
        let mut output = String::new();

        // metadatas
        output.push_str(&self.versioninfo.to_vmf_string(0));
        output.push_str(&self.visgroups.to_vmf_string(0));
        output.push_str(&self.viewsettings.to_vmf_string(0));
        output.push_str(&self.world.to_vmf_string(0));

        // entities
        for entity in &*self.entities {
            output.push_str(&entity.to_vmf_string(0));
        }

        for entity in &*self.hiddens {
            output.push_str("hidden\n{\n");
            output.push_str(&entity.to_vmf_string(1));
            output.push_str("}\n");
        }

        // regions
        output.push_str(&self.cameras.to_vmf_string(0));
        output.push_str(&self.cordons.to_vmf_string(0));

        output
    }
}

/// Represents a block in a VMF file, which can contain key-value pairs and other blocks.
#[derive(Debug, Default, Clone)]
pub struct VmfBlock {
    /// The name of the block.
    pub name: String,
    /// The key-value pairs in the block.
    pub key_values: IndexMap<String, String>,
    /// The child blocks contained within this block.
    pub blocks: Vec<VmfBlock>,
}

impl VmfBlock {
    /// Serializes the `VmfBlock` into a string with the specified indentation level.
    ///
    /// # Arguments
    ///
    /// * `indent_level` - The indentation level to use for formatting.
    ///
    /// # Returns
    ///
    /// A string representation of the `VmfBlock` in VMF format.
    pub fn serialize(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::new();

        // Opens the block with its name
        output.push_str(&format!("{}{}\n", indent, self.name));
        output.push_str(&format!("{}{{\n", indent));

        // Adds all key-value pairs with the required indent
        for (key, value) in &self.key_values {
            output.push_str(&format!("{}\t\"{}\" \"{}\"\n", indent, key, value));
        }

        // Adds nested blocks with an increased indentation level
        for block in &self.blocks {
            output.push_str(&block.serialize(indent_level + 1));
        }

        // Closes the block
        output.push_str(&format!("{}}}\n", indent));

        output
    }
}
