use enigma_auth::{EnigmaAuthClient, UserId};
use enigma_auth_grpc::{auth_client::AuthClient, ChangePasswordRequest, CreateAccountRequest, DeleteAccountRequest, GetSessionRequest, ListAccountsRequest, LoginRequest, PurgeExpiredSessionsRequest};
use std::convert::Into;
use tonic::async_trait;
use tonic::transport::Channel;
use tracing::instrument;
use uuid::Uuid;

pub struct GrpcAuthClient {
    client: AuthClient<Channel>,
}

impl GrpcAuthClient {
    fn new(client: AuthClient<Channel>) -> Self {
        GrpcAuthClient { client }
    }

    #[instrument(err)]
    pub async fn connect(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = AuthClient::connect(String::from(url)).await?;
        Ok(Self::new(client))
    }

    pub async fn default() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::connect("http://[::1]:50051").await?)
    }
}

#[async_trait]
impl EnigmaAuthClient for GrpcAuthClient {
    #[instrument(skip(self, password), err)]
    async fn create_account(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<UserId, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(CreateAccountRequest {
            username: username.into(),
            password: password.into(),
        });

        let response = self.client.create_account(request).await?;

        Ok(UserId::from(response.get_ref().user_id))
    }

    #[instrument(skip(self), err)]
    async fn delete_account(&mut self, user_id: &UserId) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(DeleteAccountRequest {
            user_id: user_id.value(),
        });

        self.client.delete_account(request).await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn change_password(
        &mut self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(ChangePasswordRequest {
            user_id: user_id,
            old_password: old_password.into(),
            new_password: new_password.into(),
        });
        let _response = self.client.change_password(request).await?;
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_session(
        &mut self,
        access_token: &Uuid,
    ) -> Result<UserId, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetSessionRequest {
            access_token: access_token.to_string(),
        });

        let response = self.client.get_session(request).await?;

        Ok(UserId::from(response.get_ref().user_id))
    }

    #[instrument(skip(self), err)]
    async fn list_accounts(&mut self) -> Result<Vec<(UserId, String)>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(ListAccountsRequest {});
        let response = self.client.list_accounts(request).await?;

        let users = &response.get_ref().users;
        let result = users
            .iter()
            .map(|user| (UserId::from(user.id), user.name.clone()))
            .collect();

        Ok(result)
    }

    #[instrument(skip(self, password), err)]
    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(LoginRequest {
            name: username.into(),
            password: password.into(),
        });

        let response = self.client.login(request).await?;

        Ok(response.get_ref().access_token.clone())
    }

    #[instrument(skip(self), err)]
    async fn purge_expired_sessions(&mut self) -> Result<u64, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PurgeExpiredSessionsRequest {});
        let response = self.client.purge_expired_sessions(request).await?;
        Ok(response.get_ref().purged_session_count)
    }
}

impl From<AuthClient<Channel>> for GrpcAuthClient {
    fn from(client: AuthClient<Channel>) -> Self {
        GrpcAuthClient::new(client)
    }
}
