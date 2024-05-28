// Copyright (c) 2023 by Alibaba.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::process::Command;

#[cfg(feature = "as")]
/// POST /attestation-policy
pub(crate) async fn attestation_policy(
    request: HttpRequest,
    input: web::Json<as_types::SetPolicyInput>,
    user_pub_key: web::Data<Option<Ed25519PublicKey>>,
    insecure: web::Data<bool>,
    attestation_service: web::Data<AttestationService>,
) -> Result<HttpResponse> {
    if !insecure.get_ref() {
        let user_pub_key = user_pub_key
            .as_ref()
            .as_ref()
            .ok_or(Error::UserPublicKeyNotProvided)?;

        validate_auth(&request, user_pub_key).map_err(|e| {
            Error::FailedAuthentication(format!("Requester is not an authorized user: {e}"))
        })?;
    }

    attestation_service
        .0
        .lock()
        .await
        .set_policy(input.into_inner())
        .await
        .map_err(|e| Error::PolicyEndpoint(format!("Set policy error {e}")))?;

    Ok(HttpResponse::Ok().finish())
}

#[cfg(feature = "policy")]
/// POST /resource-policy
pub(crate) async fn resource_policy(
    request: HttpRequest,
    input: web::Json<serde_json::Value>,
    user_pub_key: web::Data<Option<Ed25519PublicKey>>,
    insecure: web::Data<bool>,
    policy_engine: web::Data<PolicyEngine>,
) -> Result<HttpResponse> {
    if !insecure.get_ref() {
        let user_pub_key = user_pub_key
            .as_ref()
            .as_ref()
            .ok_or(Error::UserPublicKeyNotProvided)?;

        validate_auth(&request, user_pub_key).map_err(|e| {
            Error::FailedAuthentication(format!("Requester is not an authorized user: {e}"))
        })?;
    }

    policy_engine
        .0
        .lock()
        .await
        .set_policy(
            input.into_inner()["policy"]
                .as_str()
                .ok_or(Error::PolicyEndpoint(
                    "Get policy from request failed".to_string(),
                ))?
                .to_string(),
        )
        .await
        .map_err(|e| Error::PolicyEndpoint(format!("Set policy error {e}")))?;

    Ok(HttpResponse::Ok().finish())
}

#[cfg(feature = "resource")]
/// POST /resource/{repository}/{type}/{tag}
/// POST /resource/{type}/{tag}
///
/// TODO: Although this endpoint is authenticated through a JSON Web Token (JWT),
/// only identified users should be able to get a JWT and access it.
/// At the moment user identification is not supported, and the KBS CLI
/// `--user-public-key` defines the authorized user for that endpoint. In other words,
/// any JWT signed with the user's private key will be authenticated.
/// JWT generation and user identification is unimplemented for now, and thus this
/// endpoint is insecure and is only meant for testing purposes.
pub(crate) async fn set_resource(
    request: HttpRequest,
    data: web::Bytes,
    user_pub_key: web::Data<Option<Ed25519PublicKey>>,
    insecure: web::Data<bool>,
    repository: web::Data<Arc<RwLock<dyn Repository + Send + Sync>>>,
) -> Result<HttpResponse> {
    let resource_description = ResourceDesc {
        repository_name: request
            .match_info()
            .get("repository")
            .unwrap_or("default")
            .to_string(),
        resource_type: request
            .match_info()
            .get("type")
            .ok_or_else(|| Error::InvalidRequest(String::from("no `type` in url")))?
            .to_string(),
        resource_tag: request
            .match_info()
            .get("tag")
            .ok_or_else(|| Error::InvalidRequest(String::from("no `tag` in url")))?
            .to_string(),
    };

    let is_cfs_resource = resource_description.resource_type == "seeds"
        && resource_description.resource_tag == "seeds";
    if !insecure.get_ref() {
        if is_cfs_resource {
            // validate seeds
            let cfsi = attestation_service::cfs::Cfs::new("".to_string())
                .map_err(|e| Error::SetSecretFailed(format!("init cfs error: {e}")))?;
            let verify_res = cfsi.verify_seeds(String::from_utf8_lossy(data.as_ref()).into_owned())
                .map_err(|e| Error::SetSecretFailed(format!("{} seeds are invalid: {e}", resource_description.repository_name)))?;
            log::info!("confilesystem - cfsi.verify_seeds() -> verify_res = {:?}", verify_res);
            /*
            let output = Command::new("cfs-resource")
                .arg("verify")
                .arg("-s")
                .arg(String::from_utf8_lossy(data.as_ref()).into_owned())
                .output()
                .expect("failed to execute process");
            if !output.status.success() {
                return Err(Error::SetSecretFailed(format!("{} seeds are invalid", resource_description.repository_name)));
            }
            */
        } else {
            let user_pub_key = user_pub_key
                .as_ref()
                .as_ref()
                .ok_or(Error::UserPublicKeyNotProvided)?;

            validate_auth(&request, user_pub_key).map_err(|e| {
                Error::FailedAuthentication(format!("Requester is not an authorized user: {e}"))
            })?;
        }
    }

    set_secret_resource(&repository, resource_description, data.as_ref())
        .await
        .map_err(|e| Error::SetSecretFailed(format!("{e}")))?;
    Ok(HttpResponse::Ok().content_type("application/json").body(""))
}
