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
    use rstest::rstest;

    #[rstest]
    #[case::fallback_cannot_be_in_denied_list(
        None,
        Some(HashSet::from(["foo", "bar"])), 
        Some("bar"), 
        Some("fallbackStorageClass cannot be in deniedStorageClasses"))]
    #[case::fallback_not_in_denied_list_is_allowed(
        None,
        Some(HashSet::from(["foo", "bar"])), 
        Some("baz"), 
        None)]
    #[case::denined_list_cannot_be_empty(
        None,
        Some(HashSet::new()),
        None,
        Some("deniedStorageClasses cannot be empty")
    )]
    #[case::denined_list_must_has_some_item(
        None,
        Some(HashSet::from(["foo", "bar"])), 
        None,
        None)]
    #[case::alloed_list_cannot_be_empty(
        Some(HashSet::new()),
        None,
        None,
        Some("allowedStorageClasses cannot be empty")
    )]
    #[case::allowed_list_must_has_some_item(
        Some(HashSet::from(["foo", "bar"])),
        None,
        None,
        None,
    )]
    #[case::fallback_class_must_be_in_allowed_list(
        Some(HashSet::from(["foo", "bar"])),
        None,
        Some("baz"), 
        Some("fallbackStorageClass must be in allowedStorageClasses"))]
    #[case::fallback_class_in_allowed_list_must_be_ok(
        Some(HashSet::from(["foo", "bar"])),
        None,
        Some("foo"), 
        None)]
    #[case::empty_settings_are_not_allowed(
        None,
        None,
        None,
        Some("One of deniedStorageClasses or allowedStorageClasses must be set")
    )]
    #[case::empty_settings_are_not_allowed(
        Some(HashSet::from(["foo", "bar"])),
        Some(HashSet::from(["foo", "bar"])),
        None,
        Some("Only one of deniedStorageClasses or allowedStorageClasses can be set")
    )]
    fn settings_validation(
        #[case] allowed_storage_classes: Option<HashSet<&str>>,
        #[case] denied_storage_classes: Option<HashSet<&str>>,
        #[case] fallback_storage_class: Option<&str>,
        #[case] expected_error: Option<&str>,
    ) {
        let settings = Settings {
            denied_storage_classes: denied_storage_classes
                .to_owned()
                .map(|set| set.into_iter().map(String::from).collect()),
            allowed_storage_classes: allowed_storage_classes
                .to_owned()
                .map(|set| set.into_iter().map(String::from).collect()),
            fallback_storage_class: fallback_storage_class.map(String::from),
        };
        let validation_result = settings.validate();
        if let Some(expected_error) = expected_error {
            assert_eq!(
                validation_result.expect_err("Missing validation error"),
                expected_error
            );
        } else {
            assert!(validation_result.is_ok());
        }
    }
}
