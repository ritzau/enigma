use enigma_auth::{EnigmaAuthClient, UserId};
use enigma_auth_grpc::auth_client::AuthClient;
use enigma_auth_grpc::{
    CreateAccountReply, CreateAccountRequest, DeleteAccountRequest, GetSessionReply,
    GetSessionRequest, ListAccountsReply, ListAccountsRequest, LoginReply, LoginRequest,
    PurgeExpiredSessionsReply, PurgeExpiredSessionsRequest,
};
use tonic::transport::Channel;
use tonic::{async_trait, Response};
use uuid::Uuid;

pub struct GrpcAuthClient {
    client: AuthClient<Channel>,
}

impl GrpcAuthClient {
    fn new(client: AuthClient<Channel>) -> Self {
        GrpcAuthClient { client }
    }

    pub async fn connect(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::new(AuthClient::connect(String::from(url)).await?))
    }

    pub async fn default() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::connect("http://[::1]:50051").await?)
    }
}

#[async_trait]
impl EnigmaAuthClient for GrpcAuthClient {
    async fn create_account(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<UserId, Box<dyn std::error::Error>> {
        println!(
            "Sending request to create account: {}/{}",
            username, password
        );

        let request = tonic::Request::new(CreateAccountRequest {
            username: username.into(),
            password: password.into(),
        });

        let response: Response<CreateAccountReply> = self.client.create_account(request).await?;

        Ok(UserId::from(response.get_ref().user_id))
    }
    async fn delete_account(&mut self, user_id: &UserId) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(DeleteAccountRequest {
            user_id: user_id.value(),
        });

        self.client.delete_account(request).await?;

        Ok(())
    }
    async fn get_session(
        &mut self,
        access_token: &Uuid,
    ) -> Result<UserId, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetSessionRequest {
            access_token: access_token.to_string(),
        });

        let response: Response<GetSessionReply> = self.client.get_session(request).await?;

        Ok(UserId::from(response.get_ref().user_id))
    }
    async fn list_accounts(&mut self) -> Result<Vec<(UserId, String)>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(ListAccountsRequest {});

        let response: Response<ListAccountsReply> = self.client.list_accounts(request).await?;

        Ok(response
            .get_ref()
            .users
            .iter()
            .map(|user| (UserId::from(user.id), user.name.clone()))
            .collect())
    }
    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(LoginRequest {
            name: username.into(),
            password: password.into(),
        });

        let response: Response<LoginReply> = self.client.login(request).await?;

        Ok(response.get_ref().access_token.clone())
    }
    async fn purge_expired_sessions(&mut self) -> Result<u64, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PurgeExpiredSessionsRequest {});
        let response: Response<PurgeExpiredSessionsReply> =
            self.client.purge_expired_sessions(request).await?;
        Ok(response.get_ref().purged_session_count)
    }
}

impl From<AuthClient<Channel>> for GrpcAuthClient {
    fn from(client: AuthClient<Channel>) -> Self {
        GrpcAuthClient::new(client)
    }
}
