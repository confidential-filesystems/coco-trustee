// Copyright (c) 2023 by Alibaba.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

//! KBS client SDK.

use std::fmt::Debug;
use anyhow::{anyhow, bail, Result};
use as_types::SetPolicyInput;
use base64::engine::general_purpose::{STANDARD};
use base64::Engine;
use jwt_simple::prelude::{Claims, Duration, Ed25519KeyPair, EdDSAKeyPairLike};
use kbs_protocol::evidence_provider::NativeEvidenceProvider;
use kbs_protocol::token_provider::TestTokenProvider;
use kbs_protocol::KbsClientBuilder;
use kbs_protocol::KbsClientCapabilities;
use kbs_types::{TeePubKey};
use serde::{Serialize, Deserialize};

const KBS_URL_PREFIX: &str = "kbs/v0";

/// Attestation and get a result token signed by attestation service
/// Input parameters:
/// - url: KBS server root URL.
/// - [tee_pubkey_pem]: Public key (PEM format) of the RSA key pair generated in TEE.
///     This public key will be contained in attestation results token.
/// - kbs_root_certs_pem: Custom HTTPS root certificate of KBS server. It can be left blank.
pub async fn attestation(
    url: &str,
    tee_key_pem: Option<String>,
    kbs_root_certs_pem: Vec<String>,
    extra_credential_json: String,
) -> Result<String> {
    let evidence_provider = Box::new(NativeEvidenceProvider::new()?);
    let mut client_builder = KbsClientBuilder::with_evidence_provider(evidence_provider, url);
    if let Some(key) = tee_key_pem {
        client_builder = client_builder.set_tee_key(&key)
    }
    for cert in kbs_root_certs_pem {
        client_builder = client_builder.add_kbs_cert(&cert)
    }
    let mut client = client_builder.build()?;
    let extra_credential = attester::extra_credential::ExtraCredential::from_string(&extra_credential_json)
        .map_err(|e| anyhow!("{}: {:?}", "fail to parse extra_credential", e))?;
    let (token, _) = client.get_token(&extra_credential).await?;

    Ok(token.content)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Evidence {
    #[serde(rename = "tee-pubkey")]
    pub tee_pubkey: TeePubKey,
    #[serde(rename = "evidence")]
    pub evidence: String,
}

pub async fn kbs_evidence(
    url: &str,
    kbs_root_certs_pem: Vec<String>,
    challenge: &str,
) -> Result<Evidence> {
    let http_client = build_http_client(kbs_root_certs_pem)?;
    let get_evidence_url = format!("{}/{KBS_URL_PREFIX}/evidence?challenge={}", url, challenge);
    let get_evidence_response = http_client
        .get(get_evidence_url)
        .send()
        .await?;

    match get_evidence_response.status() {
        reqwest::StatusCode::OK => {
            //let rsp = get_evidence_response.text().await?;
            let cookies = get_evidence_response.cookies();
            for cookie in cookies.into_iter() {
                println!("    kbs_evidence(): cookie = {:?}", cookie);
            }
            let rsp = get_evidence_response.json::<Evidence>().await?;
            Ok(rsp)
        },
        _ => {
            bail!("Request Failed, Response: {:?}", get_evidence_response.text().await?)
        }
    }
}

/// Get secret resources with attestation results token
/// Input parameters:
/// - url: KBS server root URL.
/// - path: Resource path, format must be `<top>/<middle>/<tail>`, e.g. `alice/key/example`.
/// - tee_key_pem: TEE private key file path (PEM format). This key must consistent with the public key in `token` claims.
/// - token: Attestation Results Token file path.
/// - kbs_root_certs_pem: Custom HTTPS root certificate of KBS server. It can be left blank.
pub async fn get_resource_with_token(
    url: &str,
    path: &str,
    tee_key_pem: String,
    token: String,
    kbs_root_certs_pem: Vec<String>,
    extra_credential_json: String,
) -> Result<Vec<u8>> {
    let token_provider = Box::<TestTokenProvider>::default();
    let mut client_builder =
        KbsClientBuilder::with_token_provider(token_provider, url).set_token(&token);
    client_builder = client_builder.set_tee_key(&tee_key_pem);

    for cert in kbs_root_certs_pem {
        client_builder = client_builder.add_kbs_cert(&cert)
    }
    let mut client = client_builder.build()?;

    let resource_kbs_uri = format!("kbs:///{path}");
    let extra_credential = attester::extra_credential::ExtraCredential::from_string(&extra_credential_json)
        .map_err(|e| anyhow!("{}: {:?}", "fail to parse extra_credential", e))?;
    let resource_bytes = client
        .get_resource(serde_json::from_str(&format!("\"{resource_kbs_uri}\""))?, &extra_credential)
        .await?;
    Ok(resource_bytes)
}

/// Get secret resources with attestation
/// Input parameters:
/// - url: KBS server root URL.
/// - path: Resource path, format must be `<top>/<middle>/<tail>`, e.g. `alice/key/example`.
/// - [tee_pubkey_pem]: Public key (PEM format) of the RSA key pair generated in TEE.
/// - kbs_root_certs_pem: Custom HTTPS root certificate of KBS server. It can be left blank.
pub async fn get_resource_with_attestation(
    url: &str,
    path: &str,
    tee_key_pem: Option<String>,
    kbs_root_certs_pem: Vec<String>,
    extra_credential_json: String,
) -> Result<Vec<u8>> {
    let evidence_provider = Box::new(NativeEvidenceProvider::new()?);
    let mut client_builder = KbsClientBuilder::with_evidence_provider(evidence_provider, url);
    if let Some(key) = tee_key_pem {
        client_builder = client_builder.set_tee_key(&key);
    }

    for cert in kbs_root_certs_pem {
        client_builder = client_builder.add_kbs_cert(&cert)
    }
    let mut client = client_builder.build()?;

    let resource_kbs_uri = format!("kbs:///{path}");
    let extra_credential = attester::extra_credential::ExtraCredential::from_string(&extra_credential_json)
        .map_err(|e| anyhow!("{}: {:?}", "fail to parse extra_credential", e))?;
    let resource_bytes = client
        .get_resource(serde_json::from_str(&format!("\"{resource_kbs_uri}\""))?, &extra_credential)
        .await?;
    Ok(resource_bytes)
}

/// Set attestation policy
/// Input parameters:
/// - url: KBS server root URL.
/// - auth_key: KBS owner's authenticate private key (PEM string).
/// - policy_bytes: Policy file content in `Vec<u8>`.
/// - [policy_type]: Policy type. Default value is "rego".
/// - [policy_id]: Policy ID. Default value is "default".
/// - kbs_root_certs_pem: Custom HTTPS root certificate of KBS server. It can be left blank.
pub async fn set_attestation_policy(
    url: &str,
    auth_key: String,
    policy_bytes: Vec<u8>,
    policy_type: Option<String>,
    policy_id: Option<String>,
    kbs_root_certs_pem: Vec<String>,
) -> Result<()> {
    let auth_private_key = Ed25519KeyPair::from_pem(&auth_key)?;
    let claims = Claims::create(Duration::from_hours(2));
    let token = auth_private_key.sign(claims)?;

    let http_client = build_http_client(kbs_root_certs_pem)?;

    let set_policy_url = format!("{}/{KBS_URL_PREFIX}/attestation-policy", url);
    let post_input = SetPolicyInput {
        r#type: policy_type.unwrap_or("rego".to_string()),
        policy_id: policy_id.unwrap_or("default".to_string()),
        policy: STANDARD.encode(policy_bytes.clone()),
    };

    let res = http_client
        .post(set_policy_url)
        .header("Content-Type", "application/json")
        .bearer_auth(token.clone())
        .json::<SetPolicyInput>(&post_input)
        .send()
        .await?;

    match res.status() {
        reqwest::StatusCode::OK => Ok(()),
        _ => {
            bail!("Request Failed, Response: {:?}", res.text().await?)
        }
    }
}

#[derive(Clone, Serialize)]
struct ResourcePolicyData {
    pub policy: String,
}

/// Set resource policy
/// Input parameters:
/// - url: KBS server root URL.
/// - auth_key: KBS owner's authenticate private key (PEM string).
/// - policy_bytes: Policy file content in `Vec<u8>`.
/// - kbs_root_certs_pem: Custom HTTPS root certificate of KBS server. It can be left blank.
pub async fn set_resource_policy(
    url: &str,
    auth_key: String,
    policy_bytes: Vec<u8>,
    kbs_root_certs_pem: Vec<String>,
) -> Result<()> {
    let auth_private_key = Ed25519KeyPair::from_pem(&auth_key)?;
    let claims = Claims::create(Duration::from_hours(2));
    let token = auth_private_key.sign(claims)?;

    let http_client = build_http_client(kbs_root_certs_pem)?;

    let set_policy_url = format!("{}/{KBS_URL_PREFIX}/resource-policy", url);
    let post_input = ResourcePolicyData {
        policy: STANDARD.encode(policy_bytes.clone()),
    };

    let res = http_client
        .post(set_policy_url)
        .header("Content-Type", "application/json")
        .bearer_auth(token.clone())
        .json::<ResourcePolicyData>(&post_input)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        bail!("Request Failed, Response: {:?}", res.text().await?);
    }
    Ok(())
}

/// Set secret resource to KBS.
/// Input parameters:
/// - url: KBS server root URL.
/// - auth_key: KBS owner's authenticate private key (PEM string).
/// - resource_bytes: Resource data in `Vec<u8>`
/// - path: Resource path, format must be `<top>/<middle>/<tail>`, e.g. `alice/key/example`.
/// - kbs_root_certs_pem: Custom HTTPS root certificate of KBS server. It can be left blank.
pub async fn set_resource(
    url: &str,
    challenge: &str,
    auth_key: String,
    resource_bytes: Vec<u8>,
    path: &str,
    kbs_root_certs_pem: Vec<String>,
) -> Result<()> {
    let auth_private_key = Ed25519KeyPair::from_pem(&auth_key)?;
    let claims = Claims::create(Duration::from_hours(2));
    let token = auth_private_key.sign(claims)?;

    let http_client = build_http_client(kbs_root_certs_pem)?;

    // get evidence
    let get_evidence_url = format!("{}/{KBS_URL_PREFIX}/evidence?challenge={}", url, challenge);
    let get_evidence_response = http_client
        .get(get_evidence_url)
        .send()
        .await?;

    match get_evidence_response.status() {
        reqwest::StatusCode::OK => {
        
        },
        _ => {
            bail!("Request Failed, Response: {:?}", get_evidence_response.text().await?);
        }
    }

    let cookies = get_evidence_response.cookies();
    let mut last_cookie = "".to_string();
    for cookie in cookies.into_iter() {
        println!("set_resource(): kbs_evidence() -> cookie = {:?}", cookie);
        println!("set_resource(): kbs_evidence() -> cookie.name() = {:?}, cookie.value() = {:?}",
                 cookie.name(), cookie.value());
        //last_cookie = cookie.value().clone();
        last_cookie = format!("{}={}", cookie.name(), cookie.value());
    }
    println!("set_resource(): kbs_evidence() -> last_cookie = {:?}", last_cookie);

    //let rsp = get_evidence_response.text().await?;
    let evidence = get_evidence_response.json::<Evidence>().await?;
    println!("set_resource(): kbs_evidence() -> evidence = {:?}", evidence);

    let jwe = api_server::http::jwe(evidence.tee_pubkey, resource_bytes)?;
    let resource_bytes_ciphertext = serde_json::to_vec(&jwe)?;

    //
    let resource_url = format!("{}/{KBS_URL_PREFIX}/resource/{}", url, path);
    let res = http_client
        .post(resource_url)
        .header("Content-Type", "application/octet-stream")
        .header("Cookie", last_cookie)
        .bearer_auth(token)
        .body(resource_bytes_ciphertext.clone())
        .send()
        .await?;
    match res.status() {
        reqwest::StatusCode::OK => Ok(()),
        _ => {
            bail!("Request Failed, Response: {:?}", res.text().await?)
        }
    }
}

fn build_http_client(kbs_root_certs_pem: Vec<String>) -> Result<reqwest::Client> {
    let mut client_builder =
        reqwest::Client::builder().user_agent(format!("kbs-client/{}", env!("CARGO_PKG_VERSION")));

    for custom_root_cert in kbs_root_certs_pem.iter() {
        let cert = reqwest::Certificate::from_pem(custom_root_cert.as_bytes())?;
        client_builder = client_builder.add_root_certificate(cert);
    }

    client_builder
        .build()
        .map_err(|e| anyhow!("Build KBS http client failed: {:?}", e))
}
