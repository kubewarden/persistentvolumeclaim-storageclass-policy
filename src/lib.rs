use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1 as apicore;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

#[no_mangle]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    let pvc = match serde_json::from_value::<apicore::PersistentVolumeClaim>(
        validation_request.request.object,
    ) {
        Ok(pvc) => pvc,
        Err(_) => {
            // Not PVC, so we don't need to validate it
            return kubewarden::accept_request();
        }
    };
    let (use_denied_class, storage_class_name) =
        uses_denied_storage_class(&pvc, &validation_request.settings);
    if use_denied_class {
        if validation_request.settings.fallback_storage_class.is_some() {
            return mutate_request(pvc, &validation_request.settings);
        }
        return kubewarden::reject_request(
            Some(format!(
                "storage class \"{}\" is not allowed",
                storage_class_name.unwrap_or_default()
            )),
            None,
            None,
            None,
        );
    }
    kubewarden::accept_request()
}

fn mutate_request(pvc: apicore::PersistentVolumeClaim, settings: &Settings) -> CallResult {
    let mutated_pvc = serde_json::to_value(apicore::PersistentVolumeClaim {
        spec: Some(apicore::PersistentVolumeClaimSpec {
            storage_class_name: settings.fallback_storage_class.clone(),
            ..pvc.spec.unwrap_or_default()
        }),
        ..pvc
    })?;
    kubewarden::mutate_request(mutated_pvc)
}

fn uses_denied_storage_class(
    pvc: &apicore::PersistentVolumeClaim,
    settings: &Settings,
) -> (bool, Option<String>) {
    let storage_class_name = pvc
        .spec
        .as_ref()
        .and_then(|spec| spec.storage_class_name.clone())
        .unwrap_or_default();

    if settings
        .denied_storage_classes
        .contains(&storage_class_name)
    {
        return (true, Some(storage_class_name));
    }
    (false, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
    use kubewarden::response::ValidationResponse;
    use rstest::rstest;
    use std::collections::HashSet;

    #[rstest]
    #[case("valid-storage-class", vec![], None)]
    #[case("valid-storage-class", vec!["invalid-storage-class-name".to_owned()], None)]
    #[case("invalid-storage-class", vec!["invalid-storage-class-name".to_owned()], None)]
    fn validate_storage_class_name(
        #[case] storage_class_name: &str,
        #[case] denied_storage_classes_list: Vec<String>,
        #[case] fallback_storage_class: Option<String>,
    ) {
        let settings = Settings {
            denied_storage_classes: HashSet::from_iter(denied_storage_classes_list.iter().cloned()),
            fallback_storage_class,
        };
        let pvc = apicore::PersistentVolumeClaim {
            metadata: metav1::ObjectMeta {
                name: Some("valid-pvc".to_string()),
                ..Default::default()
            },
            spec: Some(apicore::PersistentVolumeClaimSpec {
                storage_class_name: Some(storage_class_name.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (use_denied_class, returned_storage_class_name) =
            uses_denied_storage_class(&pvc, &settings);

        if denied_storage_classes_list.contains(&storage_class_name.to_owned()) {
            assert!(use_denied_class);
            assert_eq!(returned_storage_class_name.unwrap(), storage_class_name);
        } else {
            assert!(!use_denied_class);
            assert_eq!(returned_storage_class_name, None);
        }
    }

    #[test]
    fn validate_request_mutation() {
        let settings = Settings {
            denied_storage_classes: HashSet::new(), //this field is not used in this test
            fallback_storage_class: Some("fallback-storage-class".to_string()),
        };
        let pvc = apicore::PersistentVolumeClaim {
            metadata: metav1::ObjectMeta {
                name: Some("pvc".to_string()),
                ..Default::default()
            },
            spec: Some(apicore::PersistentVolumeClaimSpec {
                storage_class_name: Some("invalid-storage-class".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let response = mutate_request(pvc, &settings);
        let response_object: ValidationResponse =
            serde_json::from_slice(&response.unwrap()).unwrap();
        let mutated_pvc = serde_json::from_value::<apicore::PersistentVolumeClaim>(
            response_object.mutated_object.unwrap(),
        )
        .unwrap();
        assert_eq!(
            mutated_pvc.spec.unwrap().storage_class_name.unwrap(),
            settings.fallback_storage_class.unwrap()
        );
    }
}
