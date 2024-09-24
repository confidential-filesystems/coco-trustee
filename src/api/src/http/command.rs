use actix_web::{HttpRequest, HttpResponse, web};
use attestation_service::cfs;
use log::info;
use crate::http::Error;
use crate::session::SessionMap;

/// GET /cfs/{addr}/commands/commands
pub(crate) async fn get_commands(
    request: HttpRequest,
    data: web::Bytes,
    _map: web::Data<SessionMap<'_>>,
) -> crate::http::Result<HttpResponse> {
    info!("confilesystem - get_commands(): request = {:?}", request);

    let addr = request
        .match_info()
        .get("addr")
        .ok_or_else(|| Error::InvalidRequest(String::from("no `addr` in url")))?
        .to_string().to_lowercase();
    info!("confilesystem - get_commands(): addr = {:?}", addr);
    let data_vec = data.as_ref().to_vec().clone();
    let extra_request = std::str::from_utf8(&data_vec).map_err(|e| {
        Error::InvalidRequest(format!("illegal extra request: {e}"))
    })?;
    info!("confilesystem - get_commands(): extra_request = {:?}", extra_request.clone());

    let cfsi = cfs::Cfs::new("".to_string(), "".to_string())
        .map_err(|e| Error::GetCommandsFailed(format!("new cfs error: {e}")))?;
    let get_rsp = cfsi.get_resource(addr.clone(), "commands".to_string(), "commands".to_string(), extra_request)
        .await
        .map_err(|e| Error::GetCommandsFailed(format!("get commands error: {e}")))?;
    info!("confilesystem - get_commands(): get_rsp = {:?}", get_rsp);

    let get_rsp_str: String = String::from_utf8(get_rsp)
        .map_err(|e| Error::GetCommandsFailed(format!("Convert response error: {e}")))?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(get_rsp_str))
}