//! This module provides common structures and functions used across the VMF parser.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::utils::{get_key, parse_hs_key};
use crate::{
    errors::{VmfError, VmfResult},
    utils::To01String,
    VmfBlock, VmfSerializable,
};

/// Represents the editor data of a VMF entity or solid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Editor {
    /// The color of the entity in the editor, in "R G B" format.
    pub color: String,
    /// The ID of the visgroup this entity is in, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visgroup_id: Option<i32>,
    /// The ID of the group this entity is in, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i32>,
    /// Whether the entity is shown in the visgroup.
    pub visgroup_shown: bool,
    /// Whether the entity should automatically be shown in the visgroup.
    pub visgroup_auto_shown: bool,
    /// Comments associated with the entity, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// The logical position of the entity in the editor, in "[x y]" format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_pos: Option<String>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            color: "255 255 255".to_string(),
            visgroup_id: None,
            group_id: None,
            visgroup_shown: false,
            visgroup_auto_shown: false,
            comments: None,
            logical_pos: None,
        }
    }
}

impl TryFrom<VmfBlock> for Editor {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        let kv = &block.key_values;

        Ok(Self {
            color: get_key!(kv, "color", "255 255 255".to_string()).to_owned(),
            visgroup_id: parse_hs_key!(kv, "visgroupid", i32).ok(),
            group_id: parse_hs_key!(kv, "groupid", i32).ok(),
            visgroup_shown: get_key!(kv, "visgroupshown", "_".to_string()) == "1",
            visgroup_auto_shown: get_key!(kv, "visgroupautoshown", "_".to_string()) == "1",
            comments: kv.get("comments").cloned(),
            logical_pos: kv.get("logicalpos").cloned(),
        })
    }
}

impl From<Editor> for VmfBlock {
    fn from(val: Editor) -> VmfBlock {
        let mut key_values = IndexMap::new();
        key_values.insert("color".to_string(), val.color);
        if let Some(visgroup_id) = val.visgroup_id {
            key_values.insert("visgroupid".to_string(), visgroup_id.to_string());
        }
        if let Some(group_id) = val.group_id {
            key_values.insert("groupid".to_string(), group_id.to_string());
        }
        key_values.insert(
            "visgroupshown".to_string(),
            val.visgroup_shown.to_01_string(),
        );
        key_values.insert(
            "visgroupautoshown".to_string(),
            val.visgroup_auto_shown.to_01_string(),
        );
        if let Some(comments) = val.comments {
            key_values.insert("comments".to_string(), comments);
        }
        if let Some(logical_pos) = val.logical_pos {
            key_values.insert("logicalpos".to_string(), logical_pos);
        }

        VmfBlock {
            name: "editor".to_string(),
            key_values,
            blocks: Vec::new(),
        }
    }
}

impl VmfSerializable for Editor {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::with_capacity(128);

        output.push_str(&format!("{0}editor\n{0}{{\n", indent));
        output.push_str(&format!("{}\t\"color\" \"{}\"\n", indent, self.color));
        if let Some(visgroup_id) = self.visgroup_id {
            output.push_str(&format!("{}\t\"visgroupid\" \"{}\"\n", indent, visgroup_id));
        }
        if let Some(group_id) = self.group_id {
            output.push_str(&format!("{}\t\"groupid\" \"{}\"\n", indent, group_id));
        }
        output.push_str(&format!(
            "{}\t\"visgroupshown\" \"{}\"\n",
            indent,
            self.visgroup_shown.to_01_string()
        ));
        output.push_str(&format!(
            "{}\t\"visgroupautoshown\" \"{}\"\n",
            indent,
            self.visgroup_auto_shown.to_01_string()
        ));
        if let Some(comments) = &self.comments {
            output.push_str(&format!("{}\t\"comments\" \"{}\"\n", indent, comments));
        }
        if let Some(logical_pos) = &self.logical_pos {
            output.push_str(&format!("{}\t\"logicalpos\" \"{}\"\n", indent, logical_pos));
        }

        output.push_str(&format!("{}}}\n", indent));
        output
    }
}
