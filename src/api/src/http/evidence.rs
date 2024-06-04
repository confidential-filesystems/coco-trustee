use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use serde::{Serialize, Deserialize};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use uuid::Uuid;
use kbs_protocol::keypair::TeeKeyPair;
use kbs_types::TeePubKey;
use crate::http::Error;
use crate::session::{Session, SessionMap};
//use anyhow::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Evidence {
    #[serde(rename = "tee-pubkey")]
    pub tee_pubkey: TeePubKey,
    #[serde(rename = "evidence")]
    pub evidence: String,
}

/// GET /evidence
pub(crate) async fn get_evidence(
    //attestation: web::Json<Attestation>,
    request: HttpRequest,
    map: web::Data<SessionMap<'_>>,
    //attestation_service: web::Data<AttestationService>,
    agent_service_url: web::Data<String>,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - get_evidence(): request = {:?}", request);
    //info!("confilesystem - get_evidence(): map = {:?}", map);
    info!("confilesystem - get_evidence(): agent_service_url = {:?}", agent_service_url);

    {
        let sessions = map.sessions.read().await;
        info!("confilesystem - get_evidence(): sessions.len() = {:?}", sessions.len());
        for session in sessions.iter() {
            let mut session_content = session.1.lock().await;
            info!("    confilesystem - get_evidence(): session.0 = {:?}, session_content.id() = {:?}, session_content.cookie() = {:?}",
            session.0, session_content.id(), session_content.cookie());
        }
    }

    {
        //key_test();
    }

    let tee_key = TeeKeyPair::new()
        .map_err(|e| Error::EvidenceIssueFailed(format!("New TeeKeyPair failed {e}")))?;
    let tee_pubkey = tee_key.export_pubkey()
        .map_err(|e| Error::EvidenceIssueFailed(format!("Export TeePubKey failed {e}")))?;
    let tee_pubkey_str = serde_json::to_string(&tee_pubkey)
        .map_err(|e| Error::EvidenceIssueFailed(format!("Json TeePubKey failed {e}")))?;
    info!("confilesystem - key_test(): tee_pubkey_str = {:?}", tee_pubkey_str);

    let mut session = Session::default();
    session.set_authenticated();
    session.set_tee_key(tee_key);
    //let cookie = session.borrow().cookie().clone();
    info!("confilesystem 1- get_evidence(): session.cookie().to_string() = {:?}", session.cookie().to_string());
    map.sessions
        .write()
        .await
        .insert(session.id().to_string(), Arc::new(Mutex::new(session.clone())));
    info!("confilesystem 2- get_evidence(): ");

    /*
    let cookie = request.cookie(KBS_SESSION_ID).ok_or(Error::MissingCookie)?;

    let sessions = map.sessions.read().await;
    let locked_session = sessions.get(cookie.value()).ok_or(Error::InvalidCookie)?;

    let mut session = locked_session.lock().await;

    info!("Cookie {} attestation {:?}", session.id(), attestation);

    if session.is_expired() {
        raise_error!(Error::ExpiredCookie);
    }

    let token = attestation_service
        .0
        .lock()
        .await
        .verify(
            session.tee(),
            session.nonce(),
            &serde_json::to_string(&attestation).unwrap(),
        )
        .await
        .map_err(|e| Error::AttestationFailed(e.to_string()))?;

    let claims_b64 = token
        .split('.')
        .nth(1)
        .ok_or_else(|| Error::TokenIssueFailed("Illegal token format".to_string()))?;
    let claims = String::from_utf8(
        URL_SAFE_NO_PAD
            .decode(claims_b64)
            .map_err(|e| Error::TokenIssueFailed(format!("Illegal token base64 claims: {e}")))?,
    )
        .map_err(|e| Error::TokenIssueFailed(format!("Illegal token base64 claims: {e}")))?;

    session.set_tee_public_key(attestation.tee_pubkey.clone());
    session.set_authenticated();
    session.set_attestation_claims(claims);

    let body = serde_json::to_string(&json!({
        "token": token,
    }))
        .map_err(|e| Error::TokenIssueFailed(format!("Serialize token failed {e}")))?;
    */
    let evidence = Evidence {
        tee_pubkey: tee_pubkey,
        evidence: "evidence-rsp".to_string(),
    };
    let evidence_rsp = serde_json::to_string(&evidence)
        .map_err(|e| Error::EvidenceIssueFailed(format!("Serialize evidence failed {e}")))?;
    Ok(HttpResponse::Ok()
        .cookie(session.cookie())
        .content_type("application/json")
        .body(evidence_rsp))
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
