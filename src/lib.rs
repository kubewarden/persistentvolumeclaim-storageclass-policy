use std::collections::HashSet;

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1::{self as apicore, PersistentVolumeClaim};

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

    let storage_class_name = pvc
        .spec
        .as_ref()
        .and_then(|spec| spec.storage_class_name.clone())
        .unwrap_or_default();

    if validation_request
        .settings
        .allowed_storage_classes
        .is_some()
    {
        return validate_allowed_classes(
            pvc,
            storage_class_name,
            validation_request.settings.allowed_storage_classes.unwrap(),
            validation_request.settings.fallback_storage_class,
        );
    }
    validate_denied_classes(
        pvc,
        storage_class_name,
        validation_request.settings.denied_storage_classes.unwrap(),
        validation_request.settings.fallback_storage_class,
    )
}

// This function is the common logic to either allow or deny list a storage class.
// If a fallback storage class is set, the request will be mutated to use it.
// If not, the request will be rejected.
fn mutate_or_reject(
    pvc: PersistentVolumeClaim,
    storage_class_name: String,
    fallback_storage_class: Option<String>,
) -> CallResult {
    if fallback_storage_class.is_some() {
        let mutated_pvc = mutate_request(pvc, fallback_storage_class);
        let json_value = serde_json::to_value(&mutated_pvc)?;
        return kubewarden::mutate_request(json_value);
    }
    kubewarden::reject_request(
        Some(format!(
            "storage class \"{}\" is not allowed",
            storage_class_name,
        )),
        None,
        None,
        None,
    )
}

fn validate_allowed_classes(
    pvc: PersistentVolumeClaim,
    storage_class_name: String,
    allowed_storage_classes: HashSet<String>,
    fallback_storage_class: Option<String>,
) -> CallResult {
    if allowed_storage_classes.contains(&storage_class_name) {
        return kubewarden::accept_request();
    }
    mutate_or_reject(pvc, storage_class_name, fallback_storage_class)
}

fn validate_denied_classes(
    pvc: PersistentVolumeClaim,
    storage_class_name: String,
    denied_storage_classes: HashSet<String>,
    fallback_storage_class: Option<String>,
) -> CallResult {
    if denied_storage_classes.contains(&storage_class_name) {
        return mutate_or_reject(pvc, storage_class_name, fallback_storage_class);
    }
    kubewarden::accept_request()
}

fn mutate_request(
    pvc: apicore::PersistentVolumeClaim,
    fallback_storage_class: Option<String>,
) -> apicore::PersistentVolumeClaim {
    apicore::PersistentVolumeClaim {
        spec: Some(apicore::PersistentVolumeClaimSpec {
            storage_class_name: fallback_storage_class.clone(),
            ..pvc.spec.unwrap_or_default()
        }),
        ..pvc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
    use rstest::rstest;

    #[rstest]
    #[case::pvc_class_in_denined_list_should_reject(vec![], vec!["slow", "standard"], None, "slow", false)]
    #[case::pvc_class_not_in_denied_list_should_accept(vec![], vec!["slow", "standard"], None, "fast", true)]
    #[case::pvc_class_should_fallback_when_using_class_from_denined_list(vec![], vec!["slow", "standard"], Some("fast"), "slow", true)]
    #[case::pvc_class_not_in_allowed_list_should_reject(vec!["slow", "standard"], vec![], None, "fast", false)]
    #[case::pvc_class_in_allowed_list_should_accept(vec!["slow", "standard"], vec![], None, "slow", true)]
    #[case::pvc_class_not_in_allowed_list_should_fallback(vec!["slow", "standard"], vec![], Some("slow"), "fast", true)]
    fn validation_tests(
        #[case] allowed_storage_classes_list: Vec<&str>,
        #[case] denied_storage_classes_list: Vec<&str>,
        #[case] fallback_storage_class: Option<&str>,
        #[case] storage_class_name: &str,
        #[case] should_accept: bool,
    ) {
        use kubewarden::response::ValidationResponse;

        let denied_storage_classes: HashSet<String> = denied_storage_classes_list
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let allowed_storage_classes: HashSet<String> = allowed_storage_classes_list
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let pvc = apicore::PersistentVolumeClaim {
            metadata: metav1::ObjectMeta {
                name: Some("pvc".to_string()),
                ..Default::default()
            },
            spec: Some(apicore::PersistentVolumeClaimSpec {
                storage_class_name: Some(storage_class_name.to_owned()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = if !allowed_storage_classes.is_empty() {
            validate_allowed_classes(
                pvc,
                storage_class_name.to_owned(),
                allowed_storage_classes,
                fallback_storage_class.map(|s| s.to_owned()),
            )
        } else {
            validate_denied_classes(
                pvc,
                storage_class_name.to_owned(),
                denied_storage_classes,
                fallback_storage_class.map(|s| s.to_owned()),
            )
        }
        .expect("CallResult should be Ok");

        let response: ValidationResponse =
            serde_json::from_slice(result.as_slice()).expect("Response should be valid JSON");
        assert!(should_accept == response.accepted);
        if should_accept {
            assert_eq!(
                fallback_storage_class.is_none(),
                response.mutated_object.is_none(),
            );
            if fallback_storage_class.is_some() {
                let mutated_pvc = serde_json::from_value::<apicore::PersistentVolumeClaim>(
                    response.mutated_object.expect("Missing mutated object"),
                )
                .expect("Mutated object should be a valid PVC");
                assert_eq!(
                    mutated_pvc
                        .spec
                        .expect("Missing spec")
                        .storage_class_name
                        .expect("Missing storage class name"),
                    fallback_storage_class.expect("Missing fallback storage class")
                );
            }
        } else {
            assert_eq!(
                response.message.expect("Missing error message"),
                format!("storage class \"{}\" is not allowed", storage_class_name)
            );
        }
    }
}
