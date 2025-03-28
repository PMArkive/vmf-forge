//! This module provides the VMF parser implementation using the `pest` parsing library.

use indexmap::IndexMap;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::errors::{VmfError, VmfResult};

use crate::vmf::regions::{Cordon, Cordons};
use crate::{Cameras, Entity, VersionInfo, ViewSettings, VisGroups, VmfBlock, VmfFile, World};

/// The VMF parser.
#[derive(Parser)]
#[grammar = "vmf.pest"]
struct VmfParser;

/// Parses a VMF string into a `VmfFile` struct.
///
/// # Arguments
///
/// * `input` - The VMF string to parse.
///
/// # Returns
///
/// A `VmfResult` containing the parsed `VmfFile` or a `VmfError` if parsing fails.
pub fn parse_vmf(input: &str) -> VmfResult<VmfFile> {
    let parsed = VmfParser::parse(Rule::file, input)
        .map_err(|e| VmfError::Parse(Box::new(e)))?
        .next()
        .unwrap(); // ok_or_else(|| VmfError::InvalidFormat("Input string did not contain a valid VMF file structure.".to_string()))?
    let mut vmf_file = VmfFile::default();

    for pair in parsed.into_inner() {
        if pair.as_rule() == Rule::block {
            let block: VmfBlock = parse_block(pair)?;

            match block.name.to_lowercase().as_str() {
                // -- metadatas
                "versioninfo" => vmf_file.versioninfo = VersionInfo::try_from(block)?,
                "visgroups" => vmf_file.visgroups = VisGroups::try_from(block)?,
                "viewsettings" => vmf_file.viewsettings = ViewSettings::try_from(block)?,

                // world
                "world" => vmf_file.world = World::try_from(block)?,

                // -- entities
                "entity" => vmf_file.entities.push(Entity::try_from(block)?),
                "hidden" => {
                    if let Some(hidden_block) = block.blocks.first() {
                        let mut ent = Entity::try_from(hidden_block.to_owned())?;
                        ent.is_hidden = true;
                        vmf_file.hiddens.push(ent)
                    }
                }

                // -- regions
                "cameras" => vmf_file.cameras = Cameras::try_from(block)?,
                "cordons" => vmf_file.cordons = Cordons::try_from(block)?,
                // for old version of VMF
                "cordon" => vmf_file.cordons.push(Cordon::try_from(block)?),
                // ....
                _ => {
                    #[cfg(feature = "debug_assert_info")]
                    debug_assert!(false, "Unexpected block name: {}", block.name);
                }
            }
        }
    }

    Ok(vmf_file)
}

/// Parses a `Pair` representing a VMF block into a `VmfBlock` struct.
///
/// # Arguments
///
/// * `pair` - The `Pair` representing the VMF block.
///
/// # Returns
///
/// A `VmfResult` containing the parsed `VmfBlock` or a `VmfError` if parsing fails.
fn parse_block(pair: Pair<Rule>) -> VmfResult<VmfBlock> {
    let mut inner = pair.into_inner();
    let block_name_pair = inner
        .next()
        .ok_or(VmfError::InvalidFormat("block name not found".to_string()))?;

    let name = block_name_pair.as_str().to_string();

    let mut key_values = IndexMap::new();
    let mut blocks = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::key_value => {
                let mut kv_inner = item.into_inner();
                let key = strip_quotes(
                    kv_inner
                        .next()
                        .ok_or(VmfError::InvalidFormat("key not found".to_string()))?
                        .as_str(),
                );
                let value = strip_quotes(
                    kv_inner
                        .next()
                        .ok_or(VmfError::InvalidFormat("value not found".to_string()))?
                        .as_str(),
                );

                key_values
                    .entry(key)
                    .and_modify(|existing_value: &mut String| {
                        existing_value.push('\r');
                        existing_value.push_str(&value);
                    })
                    .or_insert(value);
            }
            Rule::block => {
                blocks.push(parse_block(item)?);
            }
            _ => {}
        }
    }

    Ok(VmfBlock {
        name,
        key_values,
        blocks,
    })
}

/// Removes the leading and trailing quotes from a string.
///
/// # Arguments
///
/// * `s` - The string to strip quotes from.
///
/// # Returns
///
/// The string with quotes removed.
fn strip_quotes(s: &str) -> String {
    s.trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_block_valid_block() {
        let input = "entity { \"classname\" \"logic_relay\" }";
        let mut parsed = VmfParser::parse(Rule::block, input).unwrap();
        let block = parse_block(parsed.next().unwrap()).unwrap();

        assert_eq!(block.name, "entity");
        assert_eq!(
            block.key_values.get("classname"),
            Some(&"logic_relay".to_string())
        );
        assert!(block.blocks.is_empty());
    }

    #[test]
    fn parse_block_nested_blocks() {
        let input = "entity { \"classname\" \"logic_relay\" solid { \"id\" \"1\" } }";
        let mut parsed = VmfParser::parse(Rule::block, input).unwrap();
        let block = parse_block(parsed.next().unwrap()).unwrap();

        assert_eq!(block.name, "entity");
        assert_eq!(
            block.key_values.get("classname"),
            Some(&"logic_relay".to_string())
        );
        assert_eq!(block.blocks.len(), 1);
        assert_eq!(block.blocks[0].name, "solid");
        assert_eq!(block.blocks[0].key_values.get("id"), Some(&"1".to_string()));
    }

    #[test]
    fn parse_block_empty_block() {
        let input = "entity { }";
        let mut parsed = VmfParser::parse(Rule::block, input).unwrap();
        let block = parse_block(parsed.next().unwrap()).unwrap();

        assert_eq!(block.name, "entity");
        assert!(block.key_values.is_empty());
        assert!(block.blocks.is_empty());
    }
}
