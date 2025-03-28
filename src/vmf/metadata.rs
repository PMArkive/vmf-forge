//! This module provides structures for representing metadata blocks in a VMF file, such as version info, visgroups, and view settings.

use derive_more::{Deref, DerefMut, IntoIterator};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::utils::{get_key, parse_hs_key, To01String};
use crate::{
    errors::{VmfError, VmfResult},
    VmfBlock, VmfSerializable,
};

/// Represents the version info of a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionInfo {
    /// The editor version.
    pub editor_version: i32,
    /// The editor build number.
    pub editor_build: i32,
    /// The map version.
    pub map_version: i32,
    /// The format version.
    pub format_version: i32,
    /// Whether the VMF is a prefab.
    pub prefab: bool,
}

impl TryFrom<VmfBlock> for VersionInfo {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        let kv = &block.key_values;
        Ok(Self {
            editor_version: parse_hs_key!(kv, "editorversion", i32)?,
            editor_build: parse_hs_key!(kv, "editorbuild", i32)?,
            map_version: parse_hs_key!(kv, "mapversion", i32)?,
            format_version: parse_hs_key!(kv, "formatversion", i32)?,
            prefab: get_key!(kv, "prefab")? == "1",
        })
    }
}

impl From<VersionInfo> for VmfBlock {
    fn from(val: VersionInfo) -> Self {
        let mut key_values = IndexMap::new();
        key_values.insert("editorversion".to_string(), val.editor_version.to_string());
        key_values.insert("editorbuild".to_string(), val.editor_build.to_string());
        key_values.insert("mapversion".to_string(), val.map_version.to_string());
        key_values.insert("formatversion".to_string(), val.format_version.to_string());
        key_values.insert("prefab".to_string(), val.prefab.to_01_string());

        VmfBlock {
            name: "versioninfo".to_string(),
            key_values,
            blocks: Vec::new(),
        }
    }
}

impl VmfSerializable for VersionInfo {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::with_capacity(256);

        output.push_str(&format!("{0}versioninfo\n{0}{{\n", indent));
        output.push_str(&format!(
            "{}\t\"editorversion\" \"{}\"\n",
            indent, self.editor_version
        ));
        output.push_str(&format!(
            "{}\t\"editorbuild\" \"{}\"\n",
            indent, self.editor_build
        ));
        output.push_str(&format!(
            "{}\t\"mapversion\" \"{}\"\n",
            indent, self.map_version
        ));
        output.push_str(&format!(
            "{}\t\"formatversion\" \"{}\"\n",
            indent, self.format_version
        ));
        output.push_str(&format!(
            "{}\t\"prefab\" \"{}\"\n",
            indent,
            self.prefab.to_01_string()
        ));

        output.push_str(&format!("{}}}\n", indent));
        output
    }
}

/// Represents a collection of VisGroups in a VMF file.
#[derive(
    Debug, Default, Clone, Serialize, Deserialize, PartialEq, Deref, DerefMut, IntoIterator,
)]
pub struct VisGroups {
    /// The list of VisGroups.
    #[deref]
    pub groups: Vec<VisGroup>,
}

impl TryFrom<VmfBlock> for VisGroups {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        let mut groups = Vec::with_capacity(12);
        for group in block.blocks {
            groups.push(VisGroup::try_from(group)?);
        }

        Ok(Self { groups })
    }
}

impl From<VisGroups> for VmfBlock {
    fn from(val: VisGroups) -> Self {
        let mut visgroups_block = VmfBlock {
            name: "visgroups".to_string(),
            key_values: IndexMap::new(),
            blocks: Vec::with_capacity(val.groups.len()),
        };

        for group in val.groups {
            visgroups_block.blocks.push(group.into())
        }

        visgroups_block
    }
}

impl VmfSerializable for VisGroups {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::with_capacity(128);

        output.push_str(&format!("{0}visgroups\n{0}{{\n", indent));

        if self.groups.is_empty() {
            output.push_str(&format!("{}}}\n", indent));
            return output;
        }

        for group in &self.groups {
            output.push_str(&group.to_vmf_string(indent_level));
        }

        output.push_str(&format!("{}}}\n", indent));
        output
    }
}

/// Represents a VisGroup in a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisGroup {
    /// The name of the VisGroup.
    pub name: String,
    /// The ID of the VisGroup.
    pub id: i32,
    /// The color of the VisGroup in the editor.
    pub color: String,
    /// The child VisGroups of this VisGroup, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<VisGroup>>,
}

impl TryFrom<VmfBlock> for VisGroup {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        let children = if block.blocks.is_empty() {
            None
        } else {
            Some(
                block
                    .blocks
                    .into_iter()
                    .map(VisGroup::try_from)
                    .collect::<VmfResult<Vec<_>>>()?,
            )
        };

        Ok(Self {
            name: get_key!(block.key_values, "name")?.to_owned(),
            id: parse_hs_key!(block.key_values, "visgroupid", i32)?,
            color: get_key!(block.key_values, "color")?.to_owned(),
            children,
        })
    }
}

impl From<VisGroup> for VmfBlock {
    fn from(val: VisGroup) -> Self {
        // Create a block for VisGroup
        let mut visgroup_block = VmfBlock {
            name: "visgroup".to_string(),
            key_values: IndexMap::new(),
            blocks: Vec::new(),
        };

        // Adds key-value pairs for VisGroup
        visgroup_block
            .key_values
            .insert("name".to_string(), val.name);
        visgroup_block
            .key_values
            .insert("visgroupid".to_string(), val.id.to_string());
        visgroup_block
            .key_values
            .insert("color".to_string(), val.color);

        // If the `VisGroup` has a child element, adds it as nested block
        if let Some(children) = val.children {
            for child in children {
                visgroup_block.blocks.push(child.into());
            }
        }

        visgroup_block
    }
}

impl VmfSerializable for VisGroup {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::with_capacity(64);

        output.push_str(&format!("{0}\tvisgroup\n\t{0}{{\n", indent));
        output.push_str(&format!("{}\t\t\"name\" \"{}\"\n", indent, self.name));
        output.push_str(&format!("{}\t\t\"visgroupid\" \"{}\"\n", indent, self.id));
        output.push_str(&format!("{}\t\t\"color\" \"{}\"\n", indent, self.color));

        // If there are child elements, adds them
        if let Some(ref children) = self.children {
            for child in children {
                output.push_str(&child.to_vmf_string(indent_level + 1));
            }
        }

        output.push_str(&format!("{}\t}}\n", indent));
        output
    }
}

/// Represents the view settings of a VMF file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewSettings {
    /// Whether snapping to the grid is enabled.
    pub snap_to_grid: bool,
    /// Whether the grid is shown in the editor.
    pub show_grid: bool,
    /// Whether the logical grid is shown in the editor.
    pub show_logical_grid: bool,
    /// The grid spacing.
    pub grid_spacing: u16,
    /// Whether the 3D grid is shown in the editor.
    pub show_3d_grid: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            snap_to_grid: true,
            show_grid: true,
            show_logical_grid: false,
            grid_spacing: 8,
            show_3d_grid: false,
        }
    }
}

impl TryFrom<VmfBlock> for ViewSettings {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        Ok(Self {
            snap_to_grid: get_key!(&block.key_values, "bSnapToGrid")? == "1",
            show_grid: get_key!(&block.key_values, "bShowGrid")? == "1",
            show_logical_grid: get_key!(&block.key_values, "bShowLogicalGrid")? == "1",
            grid_spacing: get_key!(&block.key_values, "nGridSpacing")?
                .parse()
                .unwrap_or(64),
            show_3d_grid: get_key!(&block.key_values, "bShow3DGrid", "0".into()) == "1",
        })
    }
}

impl From<ViewSettings> for VmfBlock {
    fn from(val: ViewSettings) -> Self {
        let mut key_values = IndexMap::new();
        key_values.insert("bSnapToGrid".to_string(), val.snap_to_grid.to_01_string());
        key_values.insert("bShowGrid".to_string(), val.show_grid.to_01_string());
        key_values.insert(
            "bShowLogicalGrid".to_string(),
            val.show_logical_grid.to_01_string(),
        );
        key_values.insert("nGridSpacing".to_string(), val.grid_spacing.to_string());
        key_values.insert("bShow3DGrid".to_string(), val.show_3d_grid.to_01_string());

        VmfBlock {
            name: "viewsettings".to_string(),
            key_values,
            blocks: Vec::new(),
        }
    }
}

impl VmfSerializable for ViewSettings {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::with_capacity(64);

        output.push_str(&format!("{0}viewsettings\n{0}{{\n", indent));
        output.push_str(&format!(
            "{}\t\"bSnapToGrid\" \"{}\"\n",
            indent,
            self.snap_to_grid.to_01_string()
        ));
        output.push_str(&format!(
            "{}\t\"bShowGrid\" \"{}\"\n",
            indent,
            self.show_grid.to_01_string()
        ));
        output.push_str(&format!(
            "{}\t\"bShowLogicalGrid\" \"{}\"\n",
            indent,
            self.show_logical_grid.to_01_string()
        ));
        output.push_str(&format!(
            "{}\t\"nGridSpacing\" \"{}\"\n",
            indent, self.grid_spacing
        ));
        output.push_str(&format!(
            "{}\t\"bShow3DGrid\" \"{}\"\n",
            indent,
            self.show_3d_grid.to_01_string()
        ));

        output.push_str(&format!("{}}}\n", indent));
        output
    }
}
