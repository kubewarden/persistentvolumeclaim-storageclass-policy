use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    pub denied_storage_classes: Option<HashSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_storage_class: Option<String>,
    pub allowed_storage_classes: Option<HashSet<String>>,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        if self.denied_storage_classes.is_none() && self.allowed_storage_classes.is_none() {
            return Err(
                "One of deniedStorageClasses or allowedStorageClasses must be set".to_string(),
            );
        }
        if self.denied_storage_classes.is_some() && self.allowed_storage_classes.is_some() {
            return Err(
                "Only one of deniedStorageClasses or allowedStorageClasses can be set".to_string(),
            );
        }
        if self.denied_storage_classes.is_some()
            && self.denied_storage_classes.as_ref().unwrap().is_empty()
        {
            return Err("deniedStorageClasses cannot be empty".to_string());
        }
        if self.denied_storage_classes.is_some()
            && self.fallback_storage_class.is_some()
            && self
                .denied_storage_classes
                .as_ref()
                .unwrap()
                .contains(self.fallback_storage_class.as_ref().unwrap())
        {
            return Err("fallbackStorageClass cannot be in deniedStorageClasses".to_string());
        }
        if self.allowed_storage_classes.is_some()
            && self.allowed_storage_classes.as_ref().unwrap().is_empty()
        {
            return Err("allowedStorageClasses cannot be empty".to_string());
        }
        if self.allowed_storage_classes.is_some()
            && self.fallback_storage_class.is_some()
            && !self
                .allowed_storage_classes
                .as_ref()
                .unwrap()
                .contains(self.fallback_storage_class.as_ref().unwrap())
        {
            return Err("fallbackStorageClass must be in allowedStorageClasses".to_string());
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
            denied_storage_classes: Some(HashSet::from(["foo".to_string(), "bar".to_string()])),
            fallback_storage_class: Some("bar".to_string()),
            ..Default::default()
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
            "One of deniedStorageClasses or allowedStorageClasses must be set".to_string()
        );
    }

    #[test]
    fn validate_settings_with_empty_storage_classes_list() {
        let settings = Settings {
            denied_storage_classes: Some(HashSet::new()),
            fallback_storage_class: None,
            ..Default::default()
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
            denied_storage_classes: Some(HashSet::from(["foo".to_string(), "bar".to_string()])),
            fallback_storage_class: None,
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_settings_with_fallback() {
        let settings = Settings {
            denied_storage_classes: Some(HashSet::from(["foo".to_string(), "bar".to_string()])),
            fallback_storage_class: Some("baz".to_string()),
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_settings_with_empty_allowed_storage_classes_list() {
        let settings = Settings {
            allowed_storage_classes: Some(HashSet::new()),
            ..Default::default()
        };
        assert_eq!(
            settings.validate().expect_err("Expected error"),
            "allowedStorageClasses cannot be empty".to_string()
        );
    }

    #[test]
    fn validate_non_empty_allowed_storage_classes_list_settings() {
        let settings = Settings {
            allowed_storage_classes: Some(HashSet::from(["foo".to_string(), "bar".to_string()])),
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_fallback_should_be_defined_in_allowed_list() {
        let settings = Settings {
            allowed_storage_classes: Some(HashSet::from(["foo".to_string(), "bar".to_string()])),
            fallback_storage_class: Some("baz".to_string()),
            ..Default::default()
        };
        assert_eq!(
            settings.validate().expect_err("Expected error"),
            "fallbackStorageClass must be in allowedStorageClasses".to_string()
        );
    }

    #[test]
    fn validate_fallback_should_be_defined_in_allowed_list2() {
        let settings = Settings {
            allowed_storage_classes: Some(HashSet::from(["foo".to_string(), "bar".to_string()])),
            fallback_storage_class: Some("foo".to_string()),
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }
}
