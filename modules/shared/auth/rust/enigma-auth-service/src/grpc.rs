use crate::EnigmaAuthClient;
use enigma_auth_grpc::{
    auth_server, CreateAccountReply, CreateAccountRequest, DeleteAccountReply,
    DeleteAccountRequest, GetSessionReply, GetSessionRequest, ListAccountsReply,
    ListAccountsRequest, LoginReply, LoginRequest, PurgeExpiredSessionsReply,
    PurgeExpiredSessionsRequest, User,
};
use sqlx::types::Uuid;
use tonic::{Request, Response, Status};
use tracing::instrument;

pub struct GrpcAuthService<T: EnigmaAuthClient> {
    auth_service: T,
}

impl<T: EnigmaAuthClient> GrpcAuthService<T> {
    pub fn new(auth_service: T) -> Self {
        GrpcAuthService { auth_service }
    }
}

#[tonic::async_trait]
impl<T: EnigmaAuthClient + 'static> auth_server::Auth for GrpcAuthService<T> {
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
        let request = request.get_ref();
        match self
            .auth_service
            .delete_account(&request.user_id.into())
            .await
        {
            Ok(_) => Ok(Response::new(DeleteAccountReply {})),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    #[instrument(err, skip(self, _request), fields(caller = to_caller_string(&_request)))]
    async fn list_accounts(
        &self,
        _request: Request<ListAccountsRequest>,
    ) -> Result<Response<ListAccountsReply>, Status> {
        match self.auth_service.list_accounts().await {
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
            username = request.get_ref().name))]
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginReply>, Status> {
        let request = request.get_ref();
        let session_id = self
            .auth_service
            .login(&request.name, &request.password)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let reply = LoginReply {
            access_token: session_id.to_string(),
        };

        Ok(Response::new(reply))
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
        let request = request.get_ref();
        let access_token =
            Uuid::parse_str(&request.access_token).map_err(|e| Status::internal(e.to_string()))?;

        let (is_valid, user_id) = self
            .auth_service
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

    #[instrument(err, skip(self, _request), fields(caller = to_caller_string(&_request)))]
    async fn purge_expired_sessions(
        &self,
        _request: Request<PurgeExpiredSessionsRequest>,
    ) -> Result<Response<PurgeExpiredSessionsReply>, Status> {
        let purged_session_count = self
            .auth_service
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
