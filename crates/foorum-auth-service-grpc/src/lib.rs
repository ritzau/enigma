use foorum_auth_grpc::{
    auth_server, CreateAccountReply, CreateAccountRequest, DeleteAccountReply,
    DeleteAccountRequest, GetSessionReply, GetSessionRequest, ListAccountsReply,
    ListAccountsRequest, LoginReply, LoginRequest, PurgeExpiredSessionsReply,
    PurgeExpiredSessionsRequest, User,
};
use foorum_auth_service::FoorumAuthService;
use sqlx::types::Uuid;
use tonic::{Request, Response, Status};

pub struct GrpcAuthService<T: FoorumAuthService> {
    auth_service: T,
}

impl<T: FoorumAuthService> GrpcAuthService<T> {
    pub fn new(auth_service: T) -> Self {
        GrpcAuthService { auth_service }
    }
}

#[tonic::async_trait]
impl<T: FoorumAuthService + 'static> auth_server::Auth for GrpcAuthService<T> {
    async fn create_account(
        &self,
        request: Request<CreateAccountRequest>,
    ) -> Result<Response<CreateAccountReply>, Status> {
        let request = request.get_ref();
        println!(
            "Create account request:{}/{}",
            request.username, request.password
        );

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

    async fn delete_account(
        &self,
        request: Request<DeleteAccountRequest>,
    ) -> Result<Response<DeleteAccountReply>, Status> {
        let request = request.get_ref();
        println!("Delete account request:{}", request.user_id);

        match self
            .auth_service
            .delete_account(&request.user_id.into())
            .await
        {
            Ok(_) => Ok(Response::new(DeleteAccountReply {})),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_accounts(
        &self,
        _: Request<ListAccountsRequest>,
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

    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let request = request.into_inner();
        println!(" Login request:{}/{}", request.name, request.password);
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

    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<GetSessionReply>, Status> {
        let request = request.into_inner();
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

    async fn purge_expired_sessions(
        &self,
        _: Request<PurgeExpiredSessionsRequest>,
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
