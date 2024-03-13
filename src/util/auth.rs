use actix_web::dev::ServiceRequest;
use actix_web::{error, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;

use crate::args::ARGS;

pub async fn auth_validator(
    req: ServiceRequest,
    creds: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    match (
        ARGS.auth_basic_username.as_ref(),
        ARGS.auth_basic_password.as_ref(),
        creds.password(),
    ) {
        (Some(conf_user), Some(conf_pwd), Some(cred_pwd))
            if creds.user_id() == conf_user && conf_pwd == cred_pwd =>
        {
            Ok(req)
        }
        _ => Err((error::ErrorBadRequest("Invalid login details."), req)),
    }
}
