#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use pretty_assertions::assert_eq;
    use vmf_forge::errors::VmfError;
    use vmf_forge::vmf::common::Editor;
    use vmf_forge::vmf::entities::*;
    use vmf_forge::VmfBlock;
    use vmf_forge::VmfSerializable;

    // Tests for Entity
    #[test]
    fn entity_try_from_valid_block() {
        let mut key_values = IndexMap::new();
        key_values.insert("classname".to_string(), "logic_relay".to_string());
        key_values.insert("targetname".to_string(), "test_relay".to_string());

        let mut block = VmfBlock {
            name: "entity".to_string(),
            key_values,
            blocks: Vec::new(),
        };

        let editor = VmfBlock {
            name: "editor".to_string(),
            key_values: {
                let mut map = IndexMap::new();
                map.insert("color".to_string(), "255 255 255".to_string());
                map
            },
            blocks: Vec::new(),
        };

        let solid1 = VmfBlock {
            name: "solid".to_string(),
            key_values: {
                let mut map = IndexMap::new();
                map.insert("id".to_string(), "1".to_string());
                map
            },
            blocks: vec![],
        };

        let hidden = VmfBlock {
            name: "hidden".to_string(),
            key_values: IndexMap::new(),
            blocks: vec![VmfBlock {
                name: "solid".to_string(),
                key_values: {
                    let mut map = IndexMap::new();
                    map.insert("id".to_string(), "2".to_string());
                    map
                },
                blocks: Vec::new(),
            }],
        };

        block.blocks.push(editor);
        block.blocks.push(solid1);
        block.blocks.push(hidden);

        let entity = Entity::try_from(block).unwrap();

        assert_eq!(entity.key_values.get("classname").unwrap(), "logic_relay");
        assert_eq!(entity.key_values.get("targetname").unwrap(), "test_relay");
        assert_eq!(entity.editor.color, "255 255 255");
        assert_eq!(entity.solids.as_ref().unwrap().len(), 2);
        assert_eq!(entity.solids.as_ref().unwrap()[0].id, 1);
        assert_eq!(entity.solids.as_ref().unwrap()[1].id, 2);
    }

    #[test]
    fn entity_try_from_invalid_type() {
        let mut key_values = IndexMap::new();
        key_values.insert("classname".to_string(), "logic_relay".to_string());
        key_values.insert("targetname".to_string(), "abc".to_string());

        let block = VmfBlock {
            name: "entity".to_string(),
            key_values,
            blocks: vec![VmfBlock {
                name: "solid".to_string(),
                key_values: {
                    let mut map = IndexMap::new();
                    map.insert("id".to_string(), "abc".to_string());
                    map
                },
                blocks: vec![],
            }],
        };

        let result = Entity::try_from(block);
        assert!(matches!(result, Err(VmfError::ParseInt(_, _))));
    }

    #[test] // todo: fuck u, IndexMap (unsorted)!
    fn entity_to_vmf_string() {
        let entity = Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "logic_relay".to_string());
                map.insert("targetname".to_string(), "test_relay".to_string());
                map
            },
            connections: None,
            solids: None,
            editor: Editor {
                color: "255 255 255".to_string(),
                ..Default::default()
            },
        };

        let expected = "\
        entity\n\
        {\n\
        \t\"classname\" \"logic_relay\"\n\
        \t\"targetname\" \"test_relay\"\n\
        \teditor\n\
        \t{\n\
        \t\t\"color\" \"255 255 255\"\n\
        \t\t\"visgroupshown\" \"0\"\n\
        \t\t\"visgroupautoshown\" \"0\"\n\
        \t}\n\
        }\n";

        assert_eq!(entity.to_vmf_string(0), expected);
    }

    #[test]
    fn entity_into_vmf_block() {
        let entity = Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "logic_relay".to_string());
                map.insert("targetname".to_string(), "test_relay".to_string());
                map
            },
            connections: None,
            solids: None,
            editor: Editor {
                color: "255 255 255".to_string(),
                ..Default::default()
            },
        };

        let block: VmfBlock = entity.into();

        assert_eq!(block.name, "entity");
        assert_eq!(
            block.key_values.get("classname"),
            Some(&"logic_relay".to_string())
        );
        assert_eq!(
            block.key_values.get("targetname"),
            Some(&"test_relay".to_string())
        );
        assert_eq!(block.blocks.len(), 1);
        assert_eq!(block.blocks[0].name, "editor");
        assert_eq!(
            block.blocks[0].key_values.get("color"),
            Some(&"255 255 255".to_string())
        );
    }

    // Tests for Entities
    #[test]
    fn test_entities_find_by_keyvalue() {
        let mut entities = Entities::default();
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "entity1".to_string());
                map.insert("key1".to_string(), "value1".to_string());
                map
            },
            ..Default::default()
        });
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "entity2".to_string());
                map.insert("key1".to_string(), "value2".to_string());
                map
            },
            ..Default::default()
        });
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "entity1".to_string());
                map.insert("key2".to_string(), "value3".to_string());
                map
            },
            ..Default::default()
        });

        let found_entities: Vec<&Entity> =
            entities.find_by_keyvalue("classname", "entity1").collect();
        assert_eq!(found_entities.len(), 2);

        let found_entities: Vec<&Entity> = entities.find_by_keyvalue("key1", "value1").collect();
        assert_eq!(found_entities.len(), 1);

        let found_entities: Vec<&Entity> = entities.find_by_keyvalue("key3", "value4").collect();
        assert_eq!(found_entities.len(), 0);
    }

    #[test]
    fn test_entities_find_by_keyvalue_mut() {
        let mut entities = Entities::default();
        let entity1 = Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "entity1".to_string());
                map.insert("key1".to_string(), "value1".to_string());
                map
            },
            ..Default::default()
        };
        let entity2 = Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "entity2".to_string());
                map.insert("key1".to_string(), "value2".to_string());
                map
            },
            ..Default::default()
        };
        entities.vec.push(entity1);
        entities.vec.push(entity2);

        let found_entities: Vec<_> = entities
            .find_by_keyvalue_mut("classname", "entity1")
            .collect();
        assert_eq!(found_entities[0].key_values.get("key1").unwrap(), "value1");
        assert_eq!(found_entities.len(), 1);

        assert_eq!(entities.vec[1].key_values.get("key1").unwrap(), "value2");
    }

    #[test]
    fn test_entities_find_by_classname() {
        let mut entities = Entities::default();
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "info_player_start".to_string());
                map
            },
            ..Default::default()
        });
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "entity2".to_string());
                map
            },
            ..Default::default()
        });

        let found_entities: Vec<&Entity> =
            entities.find_by_classname("info_player_start").collect();
        assert_eq!(found_entities.len(), 1);
    }

    #[test]
    fn test_entities_find_by_name() {
        let mut entities = Entities::default();
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("targetname".to_string(), "my_entity".to_string());
                map
            },
            ..Default::default()
        });
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("targetname".to_string(), "another_entity".to_string());
                map
            },
            ..Default::default()
        });

        let found_entities: Vec<&Entity> = entities.find_by_name("my_entity").collect();
        assert_eq!(found_entities.len(), 1);
    }

    #[test]
    fn test_entities_find_by_classname_mut() {
        let mut entities = Entities::default();
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("classname".to_string(), "info_player_start".to_string());
                map
            },
            ..Default::default()
        });

        let mut found_entities = entities.find_by_classname_mut("info_player_start");
        assert!(found_entities.next().is_some());
        assert!(found_entities.next().is_none());
    }

    #[test]
    fn test_entities_find_by_name_mut() {
        let mut entities = Entities::default();
        entities.vec.push(Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("targetname".to_string(), "my_entity".to_string());
                map
            },
            ..Default::default()
        });

        let mut found_entities = entities.find_by_name_mut("my_entity");
        assert!(found_entities.next().is_some());
        assert!(found_entities.next().is_none());
    }

    #[test]
    fn entity_to_vmf_string_with_connections() {
        let entity = Entity {
            key_values: {
                let mut map = IndexMap::new();
                map.insert("id".to_string(), "2810".to_string());
                map.insert("classname".to_string(), "logic_relay".to_string());
                map.insert(
                    "targetname".to_string(),
                    "button_unpressed_relay".to_string(),
                );
                map.insert("origin".to_string(), "304 416 64".to_string());
                map
            },
            connections: Some({
                vec![
                    (
                        "OnTrigger".to_string(),
                        "@exit_door instance:door_close_relay;Trigger  0 -1".to_string(),
                    ),
                    (
                        "OnTrigger".to_string(),
                        "door_checkmark Uncheck  0 -1".to_string(),
                    ),
                ]
            }),
            solids: None,
            editor: Editor {
                color: "220 30 220".to_string(),
                visgroup_shown: true,
                visgroup_auto_shown: true,
                logical_pos: Some("[0 -5268]".to_string()),
                ..Default::default()
            },
        };

        let expected = "\
        entity\n\
        {\n\
        \t\"id\" \"2810\"\n\
        \t\"classname\" \"logic_relay\"\n\
        \t\"targetname\" \"button_unpressed_relay\"\n\
        \t\"origin\" \"304 416 64\"\n\
        \tconnections\n\
        \t{\n\
        \t\t\"OnTrigger\" \"@exit_door instance:door_close_relay;Trigger  0 -1\"\n\
        \t\t\"OnTrigger\" \"door_checkmark Uncheck  0 -1\"\n\
        \t}\n\
        \teditor\n\
        \t{\n\
        \t\t\"color\" \"220 30 220\"\n\
        \t\t\"visgroupshown\" \"1\"\n\
        \t\t\"visgroupautoshown\" \"1\"\n\
        \t\t\"logicalpos\" \"[0 -5268]\"\n\
        \t}\n\
        }\n";

        assert_eq!(entity.to_vmf_string(0), expected);
    }
}
