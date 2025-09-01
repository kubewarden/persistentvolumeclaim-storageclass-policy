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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use k8s_openapi::apimachinery::pkg::apis::meta::v1 as metav1;
//     use kubewarden::response::ValidationResponse;
//
//     // #[test]
//     // fn validate_request_mutation() {
//     //     let fallback_storage_class = "fallback-storage-class".to_string();
//     //     let pvc = apicore::PersistentVolumeClaim {
//     //         metadata: metav1::ObjectMeta {
//     //             name: Some("pvc".to_string()),
//     //             ..Default::default()
//     //         },
//     //         spec: Some(apicore::PersistentVolumeClaimSpec {
//     //             storage_class_name: Some("invalid-storage-class".to_string()),
//     //             ..Default::default()
//     //         }),
//     //         ..Default::default()
//     //     };
//     //     let response = mutate_request(pvc, &Some(fallback_storage_class.clone()));
//     //     let response_object: ValidationResponse =
//     //         serde_json::from_slice(&response.unwrap()).unwrap();
//     //     let mutated_pvc = serde_json::from_value::<apicore::PersistentVolumeClaim>(
//     //         response_object.mutated_object.unwrap(),
//     //     )
//     //     .unwrap();
//     //     assert_eq!(
//     //         mutated_pvc.spec.unwrap().storage_class_name.unwrap(),
//     //         fallback_storage_class
//     //     );
//     // }
// }
