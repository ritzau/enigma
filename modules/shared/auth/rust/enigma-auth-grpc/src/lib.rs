use enigma_auth::{AuthExtension, UserId, UserName};
use log::info;
use tonic::{Request, Status};

tonic::include_proto!("auth");

#[allow(dead_code)]
pub async fn verify_anonymous<R>(request: &Request<R>) -> Result<(), Status> {
    match request.extensions().get::<AuthExtension>() {
        Some(ext) => match ext {
            AuthExtension::Authenticated(..) => Err(Status::permission_denied("Permission denied")),
            AuthExtension::Failed => Err(Status::unauthenticated("Invalid access token")),
            AuthExtension::Anonymous => Ok(()),
        },
        None => Err(Status::internal("Internal error")),
    }
}

pub async fn verify_auth<R>(
    request: &Request<R>,
) -> Result<(UserId, UserName, Vec<String>), Status> {
    match request.extensions().get::<AuthExtension>() {
        Some(ext) => match ext {
            AuthExtension::Authenticated(user_id, username, roles) => {
                info!(
                    "Authenticated user: {}/{}/{}",
                    user_id,
                    username,
                    roles.join(", ")
                );
                Ok((user_id.clone(), username.clone(), roles.clone()))
            }
            AuthExtension::Failed => Err(Status::unauthenticated("Invalid access token")),
            AuthExtension::Anonymous => Err(Status::permission_denied("Permission denied")),
        },
        None => Err(Status::internal("Internal error")),
    }
}

pub async fn verify_role<R>(
    request: &Request<R>,
    role: &str,
) -> Result<(UserId, UserName, Vec<String>), Status> {
    let (user_id, username, roles) = verify_auth(request).await?;
    if roles.contains(&role.to_string()) {
        Ok((user_id, username, roles))
    } else {
        Err(Status::permission_denied("Permission denied"))
    }
}

pub async fn verify_admin<R>(
    request: &Request<R>,
) -> Result<(UserId, UserName, Vec<String>), Status> {
    verify_role(request, "admin").await
}
