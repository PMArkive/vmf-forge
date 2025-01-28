//! This module provides structures for representing region-specific data in a VMF file, such as cameras and cordons.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::utils::{get_key, parse_hs_key, To01String};
use crate::{
    errors::{VmfError, VmfResult},
    VmfBlock, VmfSerializable,
};

/// Represents the camera data in a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cameras {
    /// The index of the active camera.
    pub active: i8,
    /// The list of cameras.
    pub cams: Vec<Camera>,
}

impl TryFrom<VmfBlock> for Cameras {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        let mut cams = Vec::with_capacity(12);
        for group in block.blocks {
            cams.push(Camera::try_from(group)?);
        }

        Ok(Self {
            active: parse_hs_key!(&block.key_values, "activecamera", i8)?,
            cams,
        })
    }
}

impl From<Cameras> for VmfBlock {
    fn from(val: Cameras) -> Self {
        let mut blocks = Vec::with_capacity(val.cams.len());

        for cam in val.cams {
            blocks.push(cam.into());
        }

        let mut key_values = IndexMap::new();
        key_values.insert("active".to_string(), val.active.to_string());

        VmfBlock {
            name: "cameras".to_string(),
            key_values,
            blocks,
        }
    }
}

impl VmfSerializable for Cameras {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent: String = "\t".repeat(indent_level);
        let mut output = String::with_capacity(64);

        output.push_str(&format!("{0}cameras\n{0}{{\n", indent));
        output.push_str(&format!(
            "{}\t\"activecamera\" \"{}\"\n",
            indent, self.active
        ));

        for cam in &self.cams {
            output.push_str(&format!("{0}\tcamera\n{0}\t{{\n", indent));
            output.push_str(&format!(
                "{}\t\t\"position\" \"{}\"\n",
                indent, cam.position
            ));
            output.push_str(&format!("{}\t\t\"look\" \"{}\"\n", indent, cam.look));
            output.push_str(&format!("{}\t}}\n", indent));
        }

        output.push_str(&format!("{}}}\n", indent));
        output
    }
}

/// Represents a single camera in a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Camera {
    /// The position of the camera in the VMF coordinate system.
    pub position: String, // vertex
    /// The point at which the camera is looking, in the VMF coordinate system.
    pub look: String, // vertex
}

impl TryFrom<VmfBlock> for Camera {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        Ok(Self {
            position: get_key!(&block.key_values, "position")?.to_owned(),
            look: get_key!(&block.key_values, "look")?.to_owned(),
        })
    }
}

impl From<Camera> for VmfBlock {
    fn from(val: Camera) -> Self {
        let mut key_values = IndexMap::new();
        key_values.insert("position".to_string(), val.position);
        key_values.insert("look".to_string(), val.look);

        VmfBlock {
            name: "camera".to_string(),
            key_values,
            ..Default::default()
        }
    }
}

/// Represents the cordons data in a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cordons {
    /// The index of the active cordon.
    pub active: i8,
    /// The list of cordons.
    pub cordons: Vec<Cordon>,
}

impl TryFrom<VmfBlock> for Cordons {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        let mut cordons = Vec::with_capacity(12);
        for group in block.blocks {
            cordons.push(Cordon::try_from(group)?);
        }

        Ok(Self {
            active: parse_hs_key!(&block.key_values, "active", i8)?,
            cordons,
        })
    }
}

impl From<Cordons> for VmfBlock {
    fn from(val: Cordons) -> Self {
        let mut blocks = Vec::new();

        // Converts each  Cordon to a VmfBlock and adds it to the `blocks` vector
        for cordon in val.cordons {
            blocks.push(cordon.into());
        }

        // Creates a VmfBlock for Cordons
        let mut key_values = IndexMap::new();
        key_values.insert("active".to_string(), val.active.to_string());

        VmfBlock {
            name: "cordons".to_string(),
            key_values,
            blocks,
        }
    }
}

impl VmfSerializable for Cordons {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        let mut output = String::with_capacity(256);

        // Start of Cordons block
        output.push_str(&format!("{0}cordons\n{0}{{\n", indent));
        output.push_str(&format!("{}\t\"active\" \"{}\"\n", indent, self.active));

        // Iterates through all Cordons and adds their string representation
        for cordon in &self.cordons {
            output.push_str(&cordon.to_vmf_string(indent_level + 1));
        }

        output.push_str(&format!("{}}}\n", indent));

        output
    }
}

/// Represents a single cordon in a VMF file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cordon {
    /// The name of the cordon.
    pub name: String,
    /// Whether the cordon is active.
    pub active: bool,
    /// The minimum point of the cordon's bounding box.
    pub min: String, // vertex
    /// The maximum point of the cordon's bounding box.
    pub max: String, // vertex
}

impl TryFrom<VmfBlock> for Cordon {
    type Error = VmfError;

    fn try_from(block: VmfBlock) -> VmfResult<Self> {
        let (min, max) = block
            .blocks
            .first()
            .ok_or_else(|| VmfError::InvalidFormat("Missing 'box' block in Cordon".to_string()))
            .and_then(|sub_block| {
                Ok((
                    get_key!(&sub_block.key_values, "mins")?,
                    get_key!(&sub_block.key_values, "maxs")?,
                ))
            })
            .or_else(|_| {
                Ok::<(_, _), VmfError>((
                    get_key!(&block.key_values, "mins")?,
                    get_key!(&block.key_values, "maxs")?,
                ))
            })?;

        Ok(Self {
            name: get_key!(&block.key_values, "name")?.to_owned(),
            active: get_key!(&block.key_values, "active")? == "1",
            min: min.to_owned(),
            max: max.to_owned(),
        })
    }
}

impl From<Cordon> for VmfBlock {
    fn from(val: Cordon) -> Self {
        // Creates key_values for Cordon
        let mut key_values = IndexMap::new();
        key_values.insert("name".to_string(), val.name);
        key_values.insert("active".to_string(), val.active.to_01_string());

        // Creates a block for the box with `mins/maxs`
        let mut box_block_key_values = IndexMap::new();
        box_block_key_values.insert("mins".to_string(), val.min);
        box_block_key_values.insert("maxs".to_string(), val.max);

        // Creates a VmfBlock for the box
        let box_block = VmfBlock {
            name: "box".to_string(),
            key_values: box_block_key_values,
            blocks: vec![],
        };

        // Creates the main VmfBlock for Cordon
        VmfBlock {
            name: "cordon".to_string(),
            key_values,
            blocks: vec![box_block],
        }
    }
}

impl VmfSerializable for Cordon {
    fn to_vmf_string(&self, indent_level: usize) -> String {
        let indent: String = "\t".repeat(indent_level);
        let mut output = String::with_capacity(64);

        // Start of Cordon block
        output.push_str(&format!("{0}cordon\n{0}{{\n", indent));
        output.push_str(&format!("{}\t\"name\" \"{}\"\n", indent, self.name));
        output.push_str(&format!(
            "{}\t\"active\" \"{}\"\n",
            indent,
            self.active.to_01_string()
        ));

        // Adds a nested block with coordinates
        output.push_str(&format!("{0}\tbox\n{}\t{{\n", indent));
        output.push_str(&format!("{}\t\t\"mins\" \"{}\"\n", indent, self.min));
        output.push_str(&format!("{}\t\t\"maxs\" \"{}\"\n", indent, self.max));
        output.push_str(&format!("{}\t}}\n", indent)); // end of `box``

        // End of Cordon block
        output.push_str(&format!("{}}}\n", indent));

        output
    }
}
