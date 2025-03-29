#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use pretty_assertions::assert_eq;
    use vmf_forge::errors::VmfError;
    use vmf_forge::vmf::metadata::*;
    use vmf_forge::VmfBlock;
    use vmf_forge::VmfSerializable;

    // Tests for VersionInfo
    #[test]
    fn version_info_try_from_valid_block() {
        let mut key_values = IndexMap::new();
        key_values.insert("editorversion".to_string(), "400".to_string());
        key_values.insert("editorbuild".to_string(), "8000".to_string());
        key_values.insert("mapversion".to_string(), "1".to_string());
        key_values.insert("formatversion".to_string(), "100".to_string());
        key_values.insert("prefab".to_string(), "0".to_string());

        let block = VmfBlock {
            name: "versioninfo".to_string(),
            key_values,
            blocks: Vec::new(),
        };

        let version_info = VersionInfo::try_from(block).unwrap();

        assert_eq!(version_info.editor_version, 400);
        assert_eq!(version_info.editor_build, 8000);
        assert_eq!(version_info.map_version, 1);
        assert_eq!(version_info.format_version, 100);
        assert_eq!(version_info.prefab, false);
    }

    #[test]
    fn version_info_try_from_missing_key() {
        let mut key_values = IndexMap::new();
        key_values.insert("editorbuild".to_string(), "8000".to_string());
        key_values.insert("mapversion".to_string(), "1".to_string());
        key_values.insert("formatversion".to_string(), "100".to_string());
        key_values.insert("prefab".to_string(), "0".to_string());

        let block = VmfBlock {
            name: "versioninfo".to_string(),
            key_values,
            blocks: Vec::new(),
        };

        let result = VersionInfo::try_from(block);

        assert!(matches!(result, Err(VmfError::InvalidFormat(_))));
    }

    #[test]
    fn version_info_try_from_invalid_type() {
        let mut key_values = IndexMap::new();
        key_values.insert("editorversion".to_string(), "400".to_string());
        key_values.insert("editorbuild".to_string(), "abc".to_string());
        key_values.insert("mapversion".to_string(), "1".to_string());
        key_values.insert("formatversion".to_string(), "100".to_string());
        key_values.insert("prefab".to_string(), "0".to_string());

        let block = VmfBlock {
            name: "versioninfo".to_string(),
            key_values,
            blocks: Vec::new(),
        };

        let result = VersionInfo::try_from(block);

        assert!(matches!(result, Err(VmfError::ParseInt{ source: _, key: _ })));
    }

    #[test]
    fn version_info_to_vmf_string() {
        let version_info = VersionInfo {
            editor_version: 400,
            editor_build: 8000,
            map_version: 1,
            format_version: 100,
            prefab: false,
        };

        let expected = "\
            versioninfo\n\
            {\n\
            \t\"editorversion\" \"400\"\n\
            \t\"editorbuild\" \"8000\"\n\
            \t\"mapversion\" \"1\"\n\
            \t\"formatversion\" \"100\"\n\
            \t\"prefab\" \"0\"\n\
            }\n";

        assert_eq!(version_info.to_vmf_string(0), expected);
    }

    #[test]
    fn version_info_into_vmf_block() {
        let version_info = VersionInfo {
            editor_version: 400,
            editor_build: 8000,
            map_version: 1,
            format_version: 100,
            prefab: false,
        };

        let block: VmfBlock = version_info.into();

        assert_eq!(block.name, "versioninfo");
        assert_eq!(
            block.key_values.get("editorversion"),
            Some(&"400".to_string())
        );
        assert_eq!(
            block.key_values.get("editorbuild"),
            Some(&"8000".to_string())
        );
        assert_eq!(block.key_values.get("mapversion"), Some(&"1".to_string()));
        assert_eq!(
            block.key_values.get("formatversion"),
            Some(&"100".to_string())
        );
        assert_eq!(block.key_values.get("prefab"), Some(&"0".to_string()));
        assert!(block.blocks.is_empty());
    }
}
