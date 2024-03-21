use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    pub(crate) denied_storage_classes: HashSet<String>,
    pub(crate) fallback_storage_class: Option<String>,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        if self.denied_storage_classes.is_empty() {
            return Err("deniedStorageClasses cannot be empty".to_string());
        }
        if self
            .denied_storage_classes
            .contains(&self.fallback_storage_class.clone().unwrap_or_default())
        {
            return Err("fallbackStorageClass cannot be in deniedStorageClasses".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::settings::Validatable;

    #[test]
    fn validate_settings_with_fallback_inside_denied_storage_class_list() {
        let settings = Settings {
            denied_storage_classes: HashSet::from(["foo".to_string(), "bar".to_string()]),
            fallback_storage_class: Some("bar".to_string()),
        };
        assert!(settings.validate().is_err());
        assert_eq!(
            settings.validate().unwrap_err(),
            "fallbackStorageClass cannot be in deniedStorageClasses".to_string()
        );
    }

    #[test]
    fn validate_empty_settings() {
        let settings = Settings::default();
        assert!(settings.validate().is_err());
        assert_eq!(
            settings.validate().unwrap_err(),
            "deniedStorageClasses cannot be empty".to_string()
        );
    }

    #[test]
    fn validate_settings_with_empty_storage_classes_list() {
        let settings = Settings {
            denied_storage_classes: HashSet::new(),
            fallback_storage_class: None,
        };
        assert!(settings.validate().is_err());
        assert_eq!(
            settings.validate().unwrap_err(),
            "deniedStorageClasses cannot be empty".to_string()
        );
    }

    #[test]
    fn validate_settings() {
        let settings = Settings {
            denied_storage_classes: HashSet::from(["foo".to_string(), "bar".to_string()]),
            fallback_storage_class: None,
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_settings_with_fallback() {
        let settings = Settings {
            denied_storage_classes: HashSet::from(["foo".to_string(), "bar".to_string()]),
            fallback_storage_class: Some("baz".to_string()),
        };
        assert!(settings.validate().is_ok());
    }
}
