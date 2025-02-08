use enigma_auth::{AuthExtension, Role, UserId, UserName};
use itertools::Itertools;
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

pub async fn verify_auth<R>(request: &Request<R>) -> Result<(UserId, UserName, Vec<Role>), Status> {
    match request.extensions().get::<AuthExtension>() {
        Some(ext) => match ext {
            AuthExtension::Authenticated(user_id, username, roles) => {
                info!(
                    "Authenticated user: {}/{}/{}",
                    user_id,
                    username,
                    roles.join(", ")
                );
                let roles = roles
                    .iter()
                    .map(|r| Role::try_from(r.as_str()))
                    .try_collect()
                    .map_err(|_| Status::internal("Failed to process roles"))?;
                Ok((user_id.clone(), username.clone(), roles))
            }
            AuthExtension::Failed => Err(Status::unauthenticated("Invalid access token")),
            AuthExtension::Anonymous => Err(Status::permission_denied("Permission denied")),
        },
        None => Err(Status::internal("Internal error")),
    }
}

pub async fn verify_role<R>(
    request: &Request<R>,
    role: &Role,
) -> Result<(UserId, UserName, Vec<Role>), Status> {
    let (user_id, username, roles) = verify_auth(request).await?;
    if roles.contains(role) {
        Ok((user_id, username, roles))
    } else {
        Err(Status::permission_denied("Permission denied"))
    }
}

pub async fn verify_admin<R>(
    request: &Request<R>,
) -> Result<(UserId, UserName, Vec<Role>), Status> {
    verify_role(request, &Role::Admin).await
}

pub async fn verify_id_or_admin<R>(
    request: &Request<R>,
    user_id: &UserId,
) -> Result<(UserId, UserName, Vec<Role>), Status> {
    let (req_user_id, req_username, req_roles) = verify_auth(request).await?;
    if user_id == &req_user_id || req_roles.contains(&Role::Admin) {
        Ok((req_user_id, req_username, req_roles))
    } else {
        Err(Status::permission_denied("Permission denied"))
    }
}
