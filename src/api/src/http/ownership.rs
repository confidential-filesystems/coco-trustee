use std::collections::HashMap;
use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse, web};
use attestation_service::cfs;
use kbs_protocol::TeeKeyPair;
use log::info;
use tokio::sync::Mutex;
use crate::http::{Error, EvidenceRsp};
use crate::session::{Session, SessionMap};

/// POST /mint-filesystem
pub(crate) async fn mint_filesystem(
    request: HttpRequest,
    _map: web::Data<SessionMap<'_>>,
    data: web::Bytes,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - mint_filesystem(): request = {:?}", request);

    let mint_req = serde_json::from_slice::<cfs::MintFilesystemReq>(data.as_ref())
        .map_err(|e| Error::MintFilesystemFailed(format!("mint request data error: {e}")))?;
    info!("confilesystem - mint_filesystem(): mint_req = {:?}", mint_req);

    let cfsi = cfs::Cfs::new("".to_string())
        .map_err(|e| Error::MintFilesystemFailed(format!("new cfs error: {e}")))?;
    let mint_rsp = cfsi.mint_filesystem(&mint_req)
        .await
        .map_err(|e| Error::MintFilesystemFailed(format!("mint filesystem error: {e}")))?;
    info!("confilesystem - mint_filesystem(): mint_rsp = {:?}", mint_rsp);

    let mint_rsp_str = serde_json::to_string(&mint_rsp)
        .map_err(|e| Error::MintFilesystemFailed(format!("Serialize response failed {e}")))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(mint_rsp_str))
}