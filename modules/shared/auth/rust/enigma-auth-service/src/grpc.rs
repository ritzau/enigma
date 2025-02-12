use crate::EnigmaAuthService;
use async_trait::async_trait;
use enigma_auth::{AccessToken, RefreshToken, Role, UserId};
use enigma_auth_grpc::{
    auth_server, verify_admin, verify_id_or_admin, verify_role, AddRoleReply, AddRoleRequest,
    ChangePasswordReply, ChangePasswordRequest, CreateAccountReply, CreateAccountRequest,
    DeleteAccountReply, DeleteAccountRequest, GetSessionReply, GetSessionRequest, GetUserInfoReply,
    GetUserInfoRequest, ListAccountsReply, ListAccountsRequest, LoginReply, LoginRequest,
    PurgeExpiredSessionsReply, PurgeExpiredSessionsRequest, RefreshSessionReply,
    RefreshSessionRequest, RemoveRoleReply, RemoveRoleRequest, User,
};
use sqlx::types::Uuid;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::instrument;

pub struct GrpcAuthService<Svc>
where
    Svc: EnigmaAuthService,
{
    auth_service: Arc<Svc>,
}

impl<Svc> GrpcAuthService<Svc>
where
    Svc: EnigmaAuthService,
{
    pub fn new(auth_service: Arc<Svc>) -> Self {
        GrpcAuthService { auth_service }
    }
}

#[async_trait]
impl<Svc> auth_server::Auth for GrpcAuthService<Svc>
where
    Svc: EnigmaAuthService + Send + Sync + 'static,
{
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
        let parameters = request.get_ref();
        let username = &parameters.username;
        let password = &parameters.password;

        let result = self.auth_service.create_account(username, password).await;
        let user_id = result.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateAccountReply { user_id }))
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
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        verify_id_or_admin(&request, &user_id).await?;

        let result = self.auth_service.delete_account(&user_id).await;
        result.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteAccountReply {}))
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
        verify_admin(&request).await?;

        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let role = &parameters.role;

        let result = self.auth_service.add_role(&user_id, role).await;
        result.map_err(|_| Status::invalid_argument("No such user"))?;

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
        verify_admin(&request).await?;

        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let role = &parameters.role;

        let result = self.auth_service.remove_role(&user_id, role).await;
        result.map_err(|_| Status::invalid_argument("No such user"))?;

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
        verify_role(&request, &Role::User).await?;

        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        let result = self.auth_service.get_user_info(&user_id).await;
        let (user_id, username, roles) =
            result.map_err(|_| Status::invalid_argument("Invalid account"))?;

        Ok(Response::new(GetUserInfoReply {
            user_id: user_id.into(),
            username: username.to_string(),
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
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let old_password = &parameters.old_password;
        let new_password = &parameters.new_password;

        verify_id_or_admin(&request, &user_id).await?;

        let result = self
            .auth_service
            .change_password(&user_id, old_password, new_password)
            .await;
        result.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ChangePasswordReply {}))
    }

    #[instrument(err, skip(self, request), fields(caller = to_caller_string(&request)))]
    async fn refresh_session(
        &self,
        request: Request<RefreshSessionRequest>,
    ) -> Result<Response<RefreshSessionReply>, Status> {
        let parameters = request.get_ref();
        let refresh_token = Uuid::parse_str(&parameters.refresh_token)
            .map_err(|e| Status::internal(e.to_string()))?;
        let refresh_token = RefreshToken::from(refresh_token);
        let remote_ip = request.remote_addr().map(|addr| addr.ip());

        let result = self
            .auth_service
            .refresh_session(&refresh_token, &remote_ip)
            .await;
        let (access_token, refresh_token) = result.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RefreshSessionReply {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
        }))
    }

    #[instrument(
        err,
        skip(self, request),
        fields(
            caller = to_caller_string(&request),
            username = request.get_ref().name))]
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginReply>, Status> {
        let parameters = request.get_ref();
        let name = &parameters.name;
        let password = &parameters.password;
        let remote_ip = request.remote_addr().map(|addr| addr.ip());

        let result = self.auth_service.login(name, password, remote_ip).await;
        let (user_id, access_token, refresh_token) =
            result.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(LoginReply {
            user_id: user_id.value(),
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
        }))
    }

    #[instrument(err, skip(self, request), fields(caller = to_caller_string(&request)))]
    async fn list_accounts(
        &self,
        request: Request<ListAccountsRequest>,
    ) -> Result<Response<ListAccountsReply>, Status> {
        verify_role(&request, &Role::User).await?;

        let result = self.auth_service.list_accounts().await;
        let proto_users = result.map_err(|e| Status::internal(e.to_string()))?;
        let users = proto_users
            .into_iter()
            .map(|(id, name)| User { id, name })
            .collect();

        Ok(Response::new(ListAccountsReply { users }))
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
        verify_admin(&request).await?;

        let parameters = request.get_ref();
        let uuid_access_token = Uuid::parse_str(&parameters.access_token)
            .map_err(|e| Status::internal(e.to_string()))?;
        let access_token = AccessToken::from(uuid_access_token);

        let result = self.auth_service.get_session(&access_token).await;
        let (is_valid, user_id) = result.map_err(|e| Status::internal(e.to_string()))?;

        if is_valid {
            Ok(Response::new(GetSessionReply {
                user_id: user_id.map(|id| id.value()).unwrap_or(-1),
            }))
        } else {
            Err(Status::unauthenticated("Invalid access token"))
        }
    }

    #[instrument(err, skip(self, request), fields(caller = to_caller_string(&request)))]
    async fn purge_expired_sessions(
        &self,
        request: Request<PurgeExpiredSessionsRequest>,
    ) -> Result<Response<PurgeExpiredSessionsReply>, Status> {
        verify_admin(&request).await?;

        let result = self.auth_service.purge_expired_sessions().await;
        let purged_session_count = result.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(PurgeExpiredSessionsReply {
            purged_session_count,
        }))
    }
}

fn to_caller_string<T>(request: &Request<T>) -> String {
    match request.remote_addr() {
        Some(addr) => addr.ip().to_string(),
        None => "unknown".to_string(),
    }
}
