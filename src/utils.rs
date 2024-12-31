//! Utility functions and macros used throughout the VMF parser.

/// A trait for converting a boolean value to a "0" or "1" string.
pub trait To01String {
    /// Converts the boolean value to a "0" or "1" string.
    ///
    /// # Returns
    ///
    /// "1" if `self` is true, "0" otherwise.
    fn to_01_string(self) -> String;
}

impl To01String for bool {
    fn to_01_string(self) -> String {
        if self {
            "1".to_string()
        } else {
            "0".to_string()
        }
    }
}

/// Tries to get a key from a `IndexMap`, returning a `VmfError` if the key is not found.
///
/// # Arguments
///
/// * `$map` - The `IndexMap` to get the key from.
/// * `$key` - The key to get.
///
/// # Returns
///
/// A `Result` containing the value associated with the key, or a `VmfError` if the key is not found.
macro_rules! get_key {
    ($map:expr, $key:expr) => {
        $map.get($key)
            .ok_or_else(|| VmfError::InvalidFormat(format!("{} key not found", $key)))
    };

    // A variant with a default value
    ($map:expr, $key:expr, $default:expr) => {
        $map.get($key).map(|v| v.to_owned()).unwrap_or($default)
    };
}

/// Tries to get a key from a `IndexMap` and parse it as a specific type,
/// returning a `VmfError` if the key is not found or the value cannot be parsed.
///
/// # Arguments
///
/// * `$map` - The `IndexMap` to get the key from.
/// * `$key` - The key to get.
/// * `$type` - The type to parse the value as.
///
/// # Returns
///
/// A `Result` containing the parsed value, or a `VmfError` if the key is not found or the value cannot be parsed.
macro_rules! parse_hs_key {
    ($map:expr, $key:expr, $type:ty) => {
        $map.get($key)
            .ok_or_else(|| VmfError::InvalidFormat(format!("{} key not found", $key)))
            .and_then(|value| {
                value.parse::<$type>().map_err(|e| {
                    VmfError::ParseInt(e, format!("{} (in key {})", $key, stringify!($type)))
                })
            })
    };
}

pub(crate) use get_key;
pub(crate) use parse_hs_key;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::VmfError;
    use indexmap::IndexMap;

    #[test]
    fn get_key_existing_key() {
        let mut map = IndexMap::new();
        map.insert("test_key".to_string(), "test_value".to_string());

        let value = get_key!(map, "test_key").unwrap();
        assert_eq!(value, "test_value");
    }

    #[test]
    fn get_key_missing_key() {
        let map = IndexMap::<String, String>::new();
        let result = get_key!(map, "test_key");
        assert!(matches!(result, Err(VmfError::InvalidFormat(_))));
    }

    #[test]
    fn get_key_with_default_existing_key() {
        let mut map = IndexMap::new();
        map.insert("test_key".to_string(), "test_value".to_string());

        let value = get_key!(map, "test_key", "default".to_string());
        assert_eq!(value, "test_value");
    }

    #[test]
    fn get_key_with_default_missing_key() {
        let map = IndexMap::<String, String>::new();
        let value = get_key!(map, "test_key", "default".to_string());
        assert_eq!(value, "default");
    }

    #[test]
    fn parse_hs_key_valid_value() {
        let mut map = IndexMap::new();
        map.insert("test_key".to_string(), "123".to_string());

        let value = parse_hs_key!(map, "test_key", i32).unwrap();
        assert_eq!(value, 123);
    }

    #[test]
    fn parse_hs_key_missing_key() {
        let map = IndexMap::<String, String>::new();
        let result = parse_hs_key!(map, "test_key", i32);
        assert!(matches!(result, Err(VmfError::InvalidFormat(_))));
    }

    #[test]
    fn parse_hs_key_invalid_value() {
        let mut map = IndexMap::new();
        map.insert("test_key".to_string(), "abc".to_string());

        let result = parse_hs_key!(map, "test_key", i32);
        assert!(matches!(result, Err(VmfError::ParseInt(_, _))));
    }

    #[test]
    fn to_01_string_true() {
        assert_eq!(true.to_01_string(), "1");
    }

    #[test]
    fn to_01_string_false() {
        assert_eq!(false.to_01_string(), "0");
    }
}
