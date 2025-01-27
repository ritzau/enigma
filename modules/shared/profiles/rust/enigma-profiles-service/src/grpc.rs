use crate::EnigmaProfilesService;
use async_trait::async_trait;
use enigma_auth::{Role, UserId};
use enigma_auth_grpc::{verify_admin, verify_id_or_admin, verify_role};
use enigma_profiles::EnigmaUserProfile;
use enigma_profiles_grpc::profiles_server::Profiles;
use enigma_profiles_grpc::{
    AcceptConnectionReply, AcceptConnectionRequest, AddConnectionReply, AddConnectionRequest,
    Connection, CreateProfileReply, CreateProfileRequest, GetConnectionsReply,
    GetConnectionsRequest, RejectConnectionReply, RejectConnectionRequest, RemoveConnectionReply,
    RemoveConnectionRequest, RequestConnectionReply, RequestConnectionRequest,
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

        let parameters = request.get_ref();
        let profile_message = parameters
            .profile
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Missing profile"))?;

        let profile = EnigmaUserProfile::try_from(profile_message)?;

        self.profiles_service
            .lock()
            .await
            .create_profile(&profile)
            .await
            .map_err(|_| Status::invalid_argument("Invalid user"))?;

        Ok(Response::new(CreateProfileReply {}))
    }

    async fn delete_profile(
        &self,
        request: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        verify_id_or_admin(&request, &user_id).await?;

        self.profiles_service
            .lock()
            .await
            .delete_profile(&user_id)
            .await
            .map_err(|_| Status::invalid_argument("Invalid user"))?;

        Ok(Response::new(DeleteProfileReply {}))
    }

    #[instrument(err, skip_all)]
    async fn get_profile(
        &self,
        request: Request<GetProfileRequest>,
    ) -> Result<Response<GetProfileReply>, Status> {
        verify_role(&request, &Role::User).await?;

        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        let profile = self
            .profiles_service
            .lock()
            .await
            .get_profile(&user_id)
            .await
            .map_err(|_| Status::invalid_argument("Invalid user"))?;

        let reply = GetProfileReply {
            profile: Some(profile.try_into()?),
        };

        Ok(Response::new(reply))
    }

    #[instrument(err, skip_all)]
    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileReply>, Status> {
        let parameters = request.get_ref();
        let profile_message = parameters
            .profile
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Missing profile"))?;
        let profile = enigma_profiles::EnigmaUserProfile::try_from(profile_message)?;

        verify_id_or_admin(&request, &profile.user_id).await?;

        self.profiles_service
            .lock()
            .await
            .update_profile(&profile)
            .await
            .map_err(|_| Status::invalid_argument("Invalid user"))?;

        Ok(Response::new(UpdateProfileReply {}))
    }

    async fn request_connection(
        &self,
        request: Request<RequestConnectionRequest>,
    ) -> Result<Response<RequestConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let connection_id = UserId::from(parameters.connection_id);
        let kind = &parameters.kind;

        verify_id_or_admin(&request, &user_id).await?;

        self.profiles_service
            .lock()
            .await
            .request_connection(&user_id, &connection_id, kind)
            .await
            .map_err(|_| Status::invalid_argument("Cannot request connection"))?;

        Ok(Response::new(RequestConnectionReply {}))
    }

    async fn accept_connection(
        &self,
        request: Request<AcceptConnectionRequest>,
    ) -> Result<Response<AcceptConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let connection_id = UserId::from(parameters.connection_id);
        let kind = &parameters.kind;

        verify_id_or_admin(&request, &user_id).await?;

        self.profiles_service
            .lock()
            .await
            .accept_connection(&user_id, &connection_id, kind)
            .await
            .map_err(|_| Status::invalid_argument("Cannot request connection"))?;

        Ok(Response::new(AcceptConnectionReply {}))
    }

    async fn reject_connection(
        &self,
        request: Request<RejectConnectionRequest>,
    ) -> Result<Response<RejectConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let connection_id = UserId::from(parameters.connection_id);

        verify_id_or_admin(&request, &user_id).await?;

        self.profiles_service
            .lock()
            .await
            .reject_connection(&user_id, &connection_id)
            .await
            .map_err(|_| Status::invalid_argument("Cannot reject connection"))?;

        Ok(Response::new(RejectConnectionReply {}))
    }

    #[instrument(err, skip_all)]
    async fn get_connections(
        &self,
        request: Request<GetConnectionsRequest>,
    ) -> Result<Response<GetConnectionsReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        verify_id_or_admin(&request, &user_id).await?;

        let connections = self
            .profiles_service
            .lock()
            .await
            .get_connections(&user_id)
            .await
            .map_err(|_| Status::internal("Cannot get connections"))?
            .into_iter()
            .map(|(kind, profile)| Connection {
                kind,
                profile: Some(profile.try_into().unwrap()),
            })
            .collect();

        Ok(Response::new(GetConnectionsReply { connections }))
    }

    #[instrument(err, skip_all)]
    async fn add_connection(
        &self,
        request: Request<AddConnectionRequest>,
    ) -> Result<Response<AddConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let connection_id = UserId::from(parameters.connection_user_id);
        let kind = &parameters.kind;

        verify_id_or_admin(&request, &user_id).await?;

        self.profiles_service
            .lock()
            .await
            .add_connection(&user_id, &connection_id, kind)
            .await
            .map_err(|_| Status::invalid_argument("Cannot add connection"))?;

        Ok(Response::new(AddConnectionReply {}))
    }

    #[instrument(err, skip_all)]
    async fn remove_connection(
        &self,
        request: Request<RemoveConnectionRequest>,
    ) -> Result<Response<RemoveConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let connection_id = UserId::from(parameters.connection_user_id);

        verify_id_or_admin(&request, &user_id).await?;

        self.profiles_service
            .lock()
            .await
            .remove_connection(&user_id, &connection_id)
            .await
            .map_err(|_| Status::invalid_argument("Cannot remove connection"))?;

        Ok(Response::new(RemoveConnectionReply {}))
    }

    #[instrument(err, skip_all)]
    async fn update_connection(
        &self,
        request: Request<UpdateConnectionRequest>,
    ) -> Result<Response<UpdateConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let connection_id = UserId::from(parameters.connection_user_id);
        let kind = &parameters.kind;

        verify_id_or_admin(&request, &user_id).await?;

        self.profiles_service
            .lock()
            .await
            .update_connection(&user_id, &connection_id, kind)
            .await
            .map_err(|_| Status::invalid_argument("Cannot update connection"))?;

        Ok(Response::new(UpdateConnectionReply {}))
    }
}
