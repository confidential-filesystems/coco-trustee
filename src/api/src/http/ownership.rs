use std::collections::HashMap;
use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse, web};
use attestation_service::cfs;
use kbs_protocol::TeeKeyPair;
use log::info;
use tokio::sync::Mutex;
use crate::http::{Error, EvidenceRsp};
use crate::session::{Session, SessionMap};

//pub use attestation_service::cfs::BurnFilesystemReq;

/// POST /cfs/filesystems
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

/// GET /cfs/filesystems/{name}
pub(crate) async fn get_filesystem(
    request: HttpRequest,
    _map: web::Data<SessionMap<'_>>,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - get_filesystem(): request = {:?}", request);

    let filesystem_name = request
        .match_info()
        .get("name")
        .ok_or_else(|| Error::InvalidRequest(String::from("no `name` in url")))?
        .to_string();
    info!("confilesystem - get_filesystem(): filesystem_name = {:?}", filesystem_name);

    let cfsi = cfs::Cfs::new("".to_string())
        .map_err(|e| Error::GetFilesystemFailed(format!("new cfs error: {e}")))?;
    let get_rsp = cfsi.get_filesystem(&filesystem_name)
        .await
        .map_err(|e| Error::GetFilesystemFailed(format!("get filesystem error: {e}")))?;
    info!("confilesystem - get_filesystem(): get_rsp = {:?}", get_rsp);

    let get_rsp_str = serde_json::to_string(&get_rsp)
        .map_err(|e| Error::GetFilesystemFailed(format!("Serialize response failed {e}")))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(get_rsp_str))
}

/// DELETE /cfs/filesystems
pub(crate) async fn burn_filesystem(
    request: HttpRequest,
    _map: web::Data<SessionMap<'_>>,
    data: web::Bytes,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - burn_filesystem(): request = {:?}", request);

    let burn_req = serde_json::from_slice::<cfs::BurnFilesystemReq>(data.as_ref())
        .map_err(|e| Error::BurnFilesystemFailed(format!("burn request data error: {e}")))?;
    info!("confilesystem - burn_filesystem(): burn_req = {:?}", burn_req);

    let cfsi = cfs::Cfs::new("".to_string())
        .map_err(|e| Error::BurnFilesystemFailed(format!("new cfs error: {e}")))?;
    let burn_rsp = cfsi.burn_filesystem(&burn_req)
        .await
        .map_err(|e| Error::BurnFilesystemFailed(format!("burn filesystem error: {e}")))?;
    info!("confilesystem - burn_filesystem(): burn_rsp = {:?}", burn_rsp);

    Ok(HttpResponse::Ok().finish())
}

/// GET /cfs/accounts/{addr}/metatx
pub(crate) async fn get_account_metatx(
    request: HttpRequest,
    _map: web::Data<SessionMap<'_>>,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - get_account_metatx(): request = {:?}", request);

    let account_addr = request
        .match_info()
        .get("addr")
        .ok_or_else(|| Error::InvalidRequest(String::from("no `addr` in url")))?
        .to_string();
    info!("confilesystem - get_account_metatx(): account_addr = {:?}", account_addr);

    let cfsi = cfs::Cfs::new("".to_string())
        .map_err(|e| Error::GetAccountMetaTxFailed(format!("new cfs error: {e}")))?;
    let get_rsp = cfsi.get_account_metatx(&account_addr)
        .await
        .map_err(|e| Error::GetAccountMetaTxFailed(format!("get account metatx error: {e}")))?;
    info!("confilesystem - get_account_metatx(): get_rsp = {:?}", get_rsp);

    let get_rsp_str = serde_json::to_string(&get_rsp)
        .map_err(|e| Error::GetAccountMetaTxFailed(format!("Serialize response failed {e}")))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(get_rsp_str))
}

/// GET /cfs/configure/.well-known
pub(crate) async fn get_wellknown(
    request: HttpRequest,
    _map: web::Data<SessionMap<'_>>,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - get_wellknown(): request = {:?}", request);

    let cfsi = cfs::Cfs::new("".to_string())
        .map_err(|e| Error::GetWellKnownCfgFailed(format!("new cfs error: {e}")))?;
    let get_rsp = cfsi.get_wellknown()
        .await
        .map_err(|e| Error::GetWellKnownCfgFailed(format!("get wellknown config error: {e}")))?;
    info!("confilesystem - get_wellknown(): get_rsp = {:?}", get_rsp);

    let get_rsp_str = serde_json::to_string(&get_rsp)
        .map_err(|e| Error::GetWellKnownCfgFailed(format!("Serialize response failed {e}")))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(get_rsp_str))
}