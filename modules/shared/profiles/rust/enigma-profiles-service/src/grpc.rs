use crate::EnigmaProfilesService;
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_auth_grpc::verify_admin;
use enigma_profiles::EnigmaUserProfile;
use enigma_profiles_grpc::profiles_server::Profiles;
use enigma_profiles_grpc::{
    AddConnectionReply, AddConnectionRequest, CreateProfileReply, CreateProfileRequest,
    GetConnectionsReply, GetConnectionsRequest, RemoveConnectionReply, RemoveConnectionRequest,
    UpdateConnectionReply, UpdateConnectionRequest,
};
use enigma_profiles_grpc::{
    DeleteProfileReply, DeleteProfileRequest, GetProfileReply, GetProfileRequest,
    UpdateProfileReply, UpdateProfileRequest,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::instrument;

pub struct GrpcProfilesService<Svc>
where
    Svc: EnigmaProfilesService,
{
    profiles_service: Arc<Mutex<Svc>>,
}

impl<Svc> GrpcProfilesService<Svc>
where
    Svc: EnigmaProfilesService,
{
    pub fn new(profiles_service: Arc<Mutex<Svc>>) -> Self {
        Self { profiles_service }
    }
}

#[async_trait]
impl<Svc> Profiles for GrpcProfilesService<Svc>
where
    Svc: EnigmaProfilesService + Send + 'static,
{
    #[instrument(err, skip_all)]
    async fn create_profile(
        &self,
        request: Request<CreateProfileRequest>,
    ) -> Result<Response<CreateProfileReply>, Status> {
        verify_admin(&request).await?;

        let request = request.into_inner();

        let profile_message: enigma_profiles_grpc::UserProfile = request
            .profile
            .ok_or_else(|| Status::invalid_argument("Missing profile"))?;

        let profile = EnigmaUserProfile::try_from(profile_message)?;

        {
            let service = self.profiles_service.lock().await;
            service
                .create_profile(&profile)
                .await
                .map_err(|_| Status::invalid_argument("Invalid user"))?;
        }

        Ok(Response::new(CreateProfileReply {}))
    }

    async fn delete_profile(
        &self,
        request: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileReply>, Status> {
        verify_admin(&request).await?;

        {
            let service = self.profiles_service.lock().await;
            service
                .delete_profile(&UserId::from(request.get_ref().user_id))
                .await
                .map_err(|_| Status::invalid_argument("Invalid user"))?;
        }

        Ok(Response::new(DeleteProfileReply {}))
    }

    #[instrument(err, skip_all)]
    async fn get_profile(
        &self,
        request: Request<GetProfileRequest>,
    ) -> Result<Response<GetProfileReply>, Status> {
        verify_admin(&request).await?;

        let request = request.get_ref();
        let user_id = UserId::from(request.user_id);

        let profile = {
            let service = self.profiles_service.lock().await;
            service
                .get_profile(&user_id)
                .await
                .map_err(|_| Status::invalid_argument("Invalid user"))?
        };

        let reply = GetProfileReply {
            profile: Some(profile.try_into()?),
        };

        Ok(Response::new(reply))
    }

    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileReply>, Status> {
        verify_admin(&request).await?;

        let profile_message = request
            .into_inner()
            .profile
            .ok_or_else(|| Status::invalid_argument("Missing profile"))?;
        let profile = enigma_profiles::EnigmaUserProfile::try_from(profile_message)?;

        {
            let service = self.profiles_service.lock().await;
            service
                .update_profile(&profile)
                .await
                .map_err(|_| Status::invalid_argument("Invalid user"))?;
        }

        Ok(Response::new(UpdateProfileReply {}))
    }

    async fn get_connections(
        &self,
        _request: Request<GetConnectionsRequest>,
    ) -> Result<Response<GetConnectionsReply>, Status> {
        todo!()
    }

    async fn add_connection(
        &self,
        _request: Request<AddConnectionRequest>,
    ) -> Result<Response<AddConnectionReply>, Status> {
        todo!()
    }

    async fn remove_connection(
        &self,
        _request: Request<RemoveConnectionRequest>,
    ) -> Result<Response<RemoveConnectionReply>, Status> {
        todo!()
    }

    async fn update_connection(
        &self,
        _request: Request<UpdateConnectionRequest>,
    ) -> Result<Response<UpdateConnectionReply>, Status> {
        todo!()
    }
}
