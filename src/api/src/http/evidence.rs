use std::collections::HashMap;
use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse, web};
use anyhow::{anyhow, bail};
use log::info;
use serde::{Serialize, Deserialize};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use uuid::Uuid;
use kbs_protocol::keypair::TeeKeyPair;
use kbs_types::TeePubKey;
use crate::http::Error;
use crate::session::{Session, SessionMap};

#[derive(Serialize, Deserialize, Debug)]
pub struct Evidence {
    #[serde(rename = "tee-pubkey")]
    pub tee_pubkey: TeePubKey,
    #[serde(rename = "evidence")]
    pub evidence: String,
}

/// GET /evidence?challenge={}
pub(crate) async fn get_evidence(
    request: HttpRequest,
    map: web::Data<SessionMap<'_>>,
    agent_service_url: web::Data<String>,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - get_evidence(): request = {:?}", request);
    info!("confilesystem - get_evidence(): agent_service_url = {:?}", agent_service_url);
    {
        //key_test();
    }

    let tee_key = TeeKeyPair::new()
        .map_err(|e| Error::EvidenceIssueFailed(format!("New TeeKeyPair failed {e}")))?;
    let tee_pubkey = tee_key.export_pubkey()
        .map_err(|e| Error::EvidenceIssueFailed(format!("Export TeePubKey failed {e}")))?;
    let tee_pubkey_str = serde_json::to_string(&tee_pubkey)
        .map_err(|e| Error::EvidenceIssueFailed(format!("Json TeePubKey failed {e}")))?;

    let mut session = Session::default(30);
    session.set_authenticated();
    session.set_tee_key(tee_key);
    info!("confilesystem 1- get_evidence(): session.cookie().to_string() = {:?}", session.cookie().to_string());
    map.sessions
        .write()
        .await
        .insert(session.id().to_string(), Arc::new(Mutex::new(session.clone())));
    info!("confilesystem 2- get_evidence(): ");

    // get evidence from kata-agent api-server-rest
    let params: HashMap<String, String> = request
        .uri()
        .query()
        .map(|v| form_urlencoded::parse(v.as_bytes()).into_owned().collect())
        .unwrap_or_default();
    info!("confilesystem - get_evidence(): params = {:?}", params);
    let challenge = params.get("challenge")
        .ok_or_else(|| Error::InvalidRequest(String::from("no `challenge` in url")))?
        .to_string();
    let evidence = get_evidence_from_aa(agent_service_url.as_str(), &challenge, &tee_pubkey_str)
        .await
        .map_err(|e| Error::EvidenceIssueFailed(format!("Get evidence form AA failed {e}")))?;
    info!("confilesystem - get_evidence(): get_evidence_from_aa() -> evidence = {:?}", evidence);

    // return evidence and tee_pubkey
    let evidence = Evidence {
        tee_pubkey,
        evidence,
    };
    let evidence_rsp = serde_json::to_string(&evidence)
        .map_err(|e| Error::EvidenceIssueFailed(format!("Serialize evidence failed {e}")))?;
    Ok(HttpResponse::Ok()
        .cookie(session.cookie())
        .content_type("application/json")
        .body(evidence_rsp))
}

async fn get_evidence_from_aa(agent_service_url: &str, challenge: &str, tee_pubkey: &str) -> anyhow::Result<String> {
    info!("confilesystem - get_evidence_from_aa(): agent_service_url = {:?}", agent_service_url);
    info!("confilesystem - get_evidence_from_aa(): challenge = {:?}", challenge);
    info!("confilesystem - get_evidence_from_aa(): tee_pubkey = {:?}", tee_pubkey);

    let runtime_data = get_runtime_data(challenge, tee_pubkey);
    info!("confilesystem - get_evidence_from_aa(): runtime_data = {:?}", runtime_data);
    let http_client = build_http_client(vec![])?;
    // agent_service_url: http://127.0.0.1:8006
    // http://127.0.0.1:8006/aa/evidence_extra?runtime_data=123456
    let get_evidence_url = format!("{}/aa/evidence_extra?runtime_data={}", agent_service_url, runtime_data);
    info!("confilesystem - get_evidence_from_aa(): get_evidence_url = {:?}", get_evidence_url);
    let extra_credential = attester::extra_credential::ExtraCredential::new(
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "controller".to_string(),
        "".to_string(),
    );
    let res = http_client
        .get(get_evidence_url)
        .header("Content-Type", "application/json")
        .json::<attester::extra_credential::ExtraCredential>(&extra_credential)
        .send()
        .await?;
    if res.status() != reqwest::StatusCode::OK {
        bail!("Request Failed, Response: {:?}", res.text().await?);
    }

    let evidence = res.text().await?;
    //let evidence = "aa-evidence-rsp".to_string();
    info!("confilesystem - get_evidence_from_aa(): evidence = {:?}", evidence);
    Ok(evidence)
}

fn get_runtime_data(challenge: &str, tee_pubkey: &str) -> String {
    //TODO
    let challenge_tee_pubkey = format!("{challenge}{tee_pubkey}");
    let runtime_data = attester::emulate::get_hash_48bites(&challenge_tee_pubkey);
    //let runtime_data_str = String::from_utf8(runtime_data.to_vec()).unwrap();
    //let runtime_data_str = String::from_utf8_lossy(runtime_data.as_slice()).into_owned();
    let runtime_data_str = hex::encode(runtime_data.as_slice());
    let runtime_data_got = hex::decode(runtime_data_str.clone()).unwrap();
    //let runtime_data_got = runtime_data_str.clone().into_bytes();
    info!("confilesystem - get_runtime_data(): runtime_data = {:?}", runtime_data);
    info!("confilesystem - get_runtime_data(): runtime_data_str = {:?}", runtime_data_str);
    info!("confilesystem - get_runtime_data(): runtime_data_got = {:?}", runtime_data_got);
    return runtime_data_str;
}

fn build_http_client(aa_root_certs_pem: Vec<String>) -> anyhow::Result<reqwest::Client> {
    let mut client_builder =
        reqwest::Client::builder().user_agent(format!("aa-client/{}", env!("CARGO_PKG_VERSION")));

    for custom_root_cert in aa_root_certs_pem.iter() {
        let cert = reqwest::Certificate::from_pem(custom_root_cert.as_bytes())?;
        client_builder = client_builder.add_root_certificate(cert);
    }

    client_builder
        .build()
        .map_err(|e| anyhow!("Build AA http client failed: {:?}", e))
}

fn key_test() -> anyhow::Result<()> {
    let tee_key = TeeKeyPair::new()?;

    let key_pem = tee_key.to_pkcs1_pem();
    info!("confilesystem - key_test(): key_pem = {:?}", key_pem);

    let pubkey = tee_key.export_pubkey()?;
    let pubkey_str = serde_json::to_string(&pubkey)?;
    info!("confilesystem - key_test(): pubkey_str = {:?}", pubkey_str);

    return Ok(())
}
