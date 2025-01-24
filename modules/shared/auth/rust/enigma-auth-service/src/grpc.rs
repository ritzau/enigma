use crate::EnigmaAuthService;
use async_trait::async_trait;
use enigma_auth::{AuthExtension, RefreshToken, UserId, UserName};
use enigma_auth_grpc::{
    auth_server, AddRoleReply, AddRoleRequest, ChangePasswordReply, ChangePasswordRequest,
    CreateAccountReply, CreateAccountRequest, DeleteAccountReply, DeleteAccountRequest,
    GetSessionReply, GetSessionRequest, GetUserInfoReply, GetUserInfoRequest, ListAccountsReply,
    ListAccountsRequest, LoginReply, LoginRequest, PurgeExpiredSessionsReply,
    PurgeExpiredSessionsRequest, RefreshSessionReply, RefreshSessionRequest, RemoveRoleReply,
    RemoveRoleRequest, User,
};
use sqlx::types::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{info, instrument};

pub struct GrpcAuthService<T: EnigmaAuthService> {
    auth_service: Arc<Mutex<T>>,
}

impl<T: EnigmaAuthService> GrpcAuthService<T> {
    pub fn new(auth_service: Arc<Mutex<T>>) -> Self {
        GrpcAuthService { auth_service }
    }

    #[allow(dead_code)]
    async fn verify_anonymous<R>(&self, request: &Request<R>) -> Result<(), Status> {
        match request.extensions().get::<AuthExtension>() {
            Some(ext) => match ext {
                AuthExtension::Authenticated(..) => {
                    Err(Status::permission_denied("Permission denied"))
                }
                AuthExtension::Failed => Err(Status::unauthenticated("Invalid access token")),
                AuthExtension::Anonymous => Ok(()),
            },
            None => Err(Status::internal("Internal error")),
        }
    }

    async fn verify_auth<R>(
        &self,
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

    async fn verify_role<R>(
        &self,
        request: &Request<R>,
        role: &str,
    ) -> Result<(UserId, UserName, Vec<String>), Status> {
        let (user_id, username, roles) = self.verify_auth(request).await?;
        if roles.contains(&role.to_string()) {
            Ok((user_id, username, roles))
        } else {
            Err(Status::permission_denied("Permission denied"))
        }
    }

    async fn verify_admin<R>(
        &self,
        request: &Request<R>,
    ) -> Result<(UserId, UserName, Vec<String>), Status> {
        self.verify_role(request, "admin").await
    }
}

#[async_trait]
impl<T: EnigmaAuthService + 'static> auth_server::Auth for GrpcAuthService<T> {
    #[instrument(
        err,
        skip(self, request),
        fields(
            caller = to_caller_string(&request),
            username = request.get_ref().username))]
    async fn create_account(
        &self,
        request: Request<CreateAccountRequest>,
    ) -> Result<Response<CreateAccountReply>, Status> {
        let request = request.get_ref();
        match self
            .auth_service
            .lock()
            .await
            .create_account(&request.username, &request.password)
            .await
        {
            Ok(user_id) => {
                let reply = CreateAccountReply { user_id };
                Ok(Response::new(reply))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    #[instrument(
        err,
        skip(self, request),
        fields(
            caller = to_caller_string(&request),
            user_id = request.get_ref().user_id))]
    async fn delete_account(
        &self,
        request: Request<DeleteAccountRequest>,
    ) -> Result<Response<DeleteAccountReply>, Status> {
        self.verify_admin(&request).await?;

        let request = request.get_ref();
        match self
            .auth_service
            .lock()
            .await
            .delete_account(&request.user_id.into())
            .await
        {
            Ok(_) => Ok(Response::new(DeleteAccountReply {})),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    #[instrument(
        err,
        skip_all,
        fields(
            caller = to_caller_string(&request),
            user_id = request.get_ref().user_id,
            role = request.get_ref().role))]
    async fn add_role(
        &self,
        request: Request<AddRoleRequest>,
    ) -> Result<Response<AddRoleReply>, Status> {
        self.verify_admin(&request).await?;

        let request = request.into_inner();
        let service = self.auth_service.lock().await;
        service
            .add_role(&UserId::from(request.user_id), &request.role)
            .await
            .map_err(|_| Status::invalid_argument("No such user"))?;

        Ok(Response::new(AddRoleReply {}))
    }

    #[instrument(
        err,
        skip_all,
        fields(
            caller = to_caller_string(&request),
            user_id = request.get_ref().user_id,
            role = request.get_ref().role))]
    async fn remove_role(
        &self,
        request: Request<RemoveRoleRequest>,
    ) -> Result<Response<RemoveRoleReply>, Status> {
        self.verify_admin(&request).await?;

        let request = request.into_inner();
        let service = self.auth_service.lock().await;
        service
            .remove_role(&UserId::from(request.user_id), &request.role)
            .await
            .map_err(|_| Status::invalid_argument("No such user"))?;

        Ok(Response::new(RemoveRoleReply {}))
    }

    #[instrument(
        err,
        skip_all,
        fields(
            caller = to_caller_string(&request),
            user_id = request.get_ref().user_id))]
    async fn get_user_info(
        &self,
        request: Request<GetUserInfoRequest>,
    ) -> Result<Response<GetUserInfoReply>, Status> {
        self.verify_admin(&request).await?;

        let request = request.get_ref();
        let (user_id, username, roles) = self
            .auth_service
            .lock()
            .await
            .get_user_info(&request.user_id.into())
            .await
            .map_err(|_| Status::invalid_argument("Invalid account"))?;

        Ok(Response::new(GetUserInfoReply {
            user_id: user_id.value(),
            username: username.as_str().to_string(),
            roles,
        }))
    }

    #[instrument(
        err,
        skip(self, request),
        fields(
            caller = to_caller_string(&request),
            user_id = request.get_ref().user_id))]
    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordReply>, Status> {
        let request = request.get_ref();
        self.auth_service
            .lock()
            .await
            .change_password(
                &UserId::from(request.user_id),
                &request.old_password,
                &request.new_password,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let reply = ChangePasswordReply {};
        Ok(Response::new(reply))
    }

    #[instrument(err, skip(self, request), fields(caller = to_caller_string(&request)))]
    async fn refresh_session(
        &self,
        request: Request<RefreshSessionRequest>,
    ) -> Result<Response<RefreshSessionReply>, Status> {
        let remote_ip = request.remote_addr().map(|addr| addr.ip());
        let request = request.get_ref();

        let refresh_token =
            Uuid::parse_str(&request.refresh_token).map_err(|e| Status::internal(e.to_string()))?;
        let refresh_token = RefreshToken::from(refresh_token);

        let (access_token, refresh_token) = self
            .auth_service
            .lock()
            .await
            .refresh_session(&refresh_token, &remote_ip)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let reply = RefreshSessionReply {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
        };

        Ok(Response::new(reply))
    }

    #[instrument(
        err,
        skip(self, request),
        fields(
            caller = to_caller_string(&request),
            username = request.get_ref().name))]
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginReply>, Status> {
        let remote_ip = request.remote_addr().map(|addr| addr.ip());
        let request = request.get_ref();
        let (user_id, access_token, refresh_token) = self
            .auth_service
            .lock()
            .await
            .login(&request.name, &request.password, remote_ip)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let reply = LoginReply {
            user_id: user_id.value(),
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
        };

        Ok(Response::new(reply))
    }

    #[instrument(err, skip(self, request), fields(caller = to_caller_string(&request)))]
    async fn list_accounts(
        &self,
        request: Request<ListAccountsRequest>,
    ) -> Result<Response<ListAccountsReply>, Status> {
        self.verify_admin(&request).await?;

        match self.auth_service.lock().await.list_accounts().await {
            Ok(users) => {
                let reply = ListAccountsReply {
                    users: users
                        .into_iter()
                        .map(|(id, name)| User { id, name })
                        .collect(),
                };
                Ok(Response::new(reply))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    #[instrument(
        err,
        skip(self, request),
        fields(
            caller = to_caller_string(&request),
            access_token = request.get_ref().access_token))]
    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<GetSessionReply>, Status> {
        self.verify_admin(&request).await?;

        let request = request.get_ref();
        let access_token =
            Uuid::parse_str(&request.access_token).map_err(|e| Status::internal(e.to_string()))?;

        let (is_valid, user_id) = self
            .auth_service
            .lock()
            .await
            .get_session(&access_token.into())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if is_valid {
            let reply = GetSessionReply {
                user_id: user_id.map(|id| id.value()).unwrap_or(-1),
            };

            Ok(Response::new(reply))
        } else {
            Err(Status::unauthenticated("Invalid access token"))
        }
    }

    #[instrument(err, skip(self, request), fields(caller = to_caller_string(&request)))]
    async fn purge_expired_sessions(
        &self,
        request: Request<PurgeExpiredSessionsRequest>,
    ) -> Result<Response<PurgeExpiredSessionsReply>, Status> {
        self.verify_admin(&request).await?;

        let purged_session_count = self
            .auth_service
            .lock()
            .await
            .purge_expired_sessions()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let reply = PurgeExpiredSessionsReply {
            purged_session_count,
        };

        Ok(Response::new(reply))
    }
}

fn to_caller_string<T>(request: &Request<T>) -> String {
    match request.remote_addr() {
        Some(addr) => addr.ip().to_string(),
        None => "unknown".to_string(),
    }
}
