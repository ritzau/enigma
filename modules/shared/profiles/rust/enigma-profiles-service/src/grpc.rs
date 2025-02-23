use crate::EnigmaProfilesService;
use async_trait::async_trait;
use enigma_auth::{Role, UserId};
use enigma_auth_grpc::{verify_admin, verify_auth, verify_id_or_admin, verify_role};
use enigma_profiles::{EnigmaUserProfile, PostId};
use enigma_profiles_grpc::profiles_server::Profiles;
use enigma_profiles_grpc::{
    AcceptConnectionReply, AcceptConnectionRequest, AddConnectionReply, AddConnectionRequest,
    Connection, CreatePostReply, CreatePostRequest, CreateProfileReply, CreateProfileRequest,
    DeletePostReply, DeletePostRequest, DeleteProfileReply, DeleteProfileRequest, GetProfileReply,
    GetProfileRequest, ListConnectionsReply, ListConnectionsRequest, ListFeedPostsReply,
    ListFeedPostsRequest, ListProfilePostsReply, ListProfilePostsRequest, RejectConnectionReply,
    RejectConnectionRequest, RemoveConnectionReply, RemoveConnectionRequest,
    RequestConnectionReply, RequestConnectionRequest, SearchProfilesReply, SearchProfilesRequest,
    UpdateConnectionReply, UpdateConnectionRequest, UpdateProfileReply, UpdateProfileRequest,
};
use itertools::Itertools;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{event, instrument, Level};

pub struct GrpcProfilesService<Svc>
where
    Svc: EnigmaProfilesService,
{
    profiles_service: Arc<Svc>,
}

impl<Svc> GrpcProfilesService<Svc>
where
    Svc: EnigmaProfilesService,
{
    pub fn new(profiles_service: Arc<Svc>) -> Self {
        Self { profiles_service }
    }
}

#[async_trait]
impl<Svc> Profiles for GrpcProfilesService<Svc>
where
    Svc: EnigmaProfilesService + Send + Sync + 'static,
{
    // Profiles

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

        let result = self.profiles_service.create_profile(&profile).await;
        result.map_err(|_| Status::invalid_argument("Invalid user"))?;

        Ok(Response::new(CreateProfileReply {}))
    }

    async fn delete_profile(
        &self,
        request: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        verify_id_or_admin(&request, &user_id).await?;

        let result = self.profiles_service.delete_profile(&user_id).await;
        result.map_err(|_| Status::invalid_argument("Invalid user"))?;

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

        let result = self.profiles_service.get_profile(&user_id).await;
        let profile = result.map_err(|_| Status::invalid_argument("Invalid user"))?;

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

        let result = self.profiles_service.update_profile(&profile).await;
        result.map_err(|_| Status::invalid_argument("Invalid user"))?;

        Ok(Response::new(UpdateProfileReply {}))
    }

    async fn search_profiles(
        &self,
        request: Request<SearchProfilesRequest>,
    ) -> Result<Response<SearchProfilesReply>, Status> {
        let parameters = request.get_ref();
        let query = &parameters.query;

        verify_role(&request, &Role::User).await?;

        let result = self.profiles_service.search_profiles(query).await;
        let proto_profiles = result.map_err(|_| Status::internal("Cannot search profiles"))?;
        let profiles = proto_profiles
            .into_iter()
            .map(|profile| profile.try_into())
            .try_collect()?;

        Ok(Response::new(SearchProfilesReply { profiles }))
    }

    // Connections

    async fn request_connection(
        &self,
        request: Request<RequestConnectionRequest>,
    ) -> Result<Response<RequestConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let peer_id = UserId::from(parameters.peer_id);
        let relationship = &parameters.relationship;

        verify_id_or_admin(&request, &user_id).await?;

        let result = self
            .profiles_service
            .request_connection(&user_id, &peer_id, relationship)
            .await;
        result.map_err(|_| Status::invalid_argument("Cannot request connection"))?;

        Ok(Response::new(RequestConnectionReply {}))
    }

    async fn accept_connection(
        &self,
        request: Request<AcceptConnectionRequest>,
    ) -> Result<Response<AcceptConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let peer_id = UserId::from(parameters.peer_id);
        let relationship = &parameters.relationship;

        verify_id_or_admin(&request, &user_id).await?;

        let result = self
            .profiles_service
            .accept_connection(&user_id, &peer_id, relationship)
            .await;
        result.map_err(|_| Status::invalid_argument("Cannot request connection"))?;

        Ok(Response::new(AcceptConnectionReply {}))
    }

    async fn reject_connection(
        &self,
        request: Request<RejectConnectionRequest>,
    ) -> Result<Response<RejectConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let peer_id = UserId::from(parameters.peer_id);

        verify_id_or_admin(&request, &user_id).await?;

        let result = self
            .profiles_service
            .reject_connection(&user_id, &peer_id)
            .await;
        result.map_err(|_| Status::invalid_argument("Cannot reject connection"))?;

        Ok(Response::new(RejectConnectionReply {}))
    }

    #[instrument(err, skip_all)]
    async fn list_connections(
        &self,
        request: Request<ListConnectionsRequest>,
    ) -> Result<Response<ListConnectionsReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        verify_id_or_admin(&request, &user_id).await?;

        let result = self.profiles_service.list_connections(&user_id).await;
        let proto_connections = result.map_err(|_| Status::internal("Cannot get connections"))?;
        let connections = proto_connections
            .into_iter()
            .map(|(relationship, profile)| {
                let profile = profile.try_into()?;
                Ok::<Connection, Status>(Connection {
                    relationship,
                    profile: Some(profile),
                })
            })
            .try_collect()?;

        Ok(Response::new(ListConnectionsReply { connections }))
    }

    #[instrument(err, skip_all)]
    async fn add_connection(
        &self,
        request: Request<AddConnectionRequest>,
    ) -> Result<Response<AddConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let peer_id = UserId::from(parameters.peer_id);
        let relationship = &parameters.relationship;

        verify_id_or_admin(&request, &user_id).await?;

        let result = self
            .profiles_service
            .add_connection(&user_id, &peer_id, relationship)
            .await;
        result.map_err(|_| Status::invalid_argument("Cannot add connection"))?;

        Ok(Response::new(AddConnectionReply {}))
    }

    #[instrument(err, skip_all)]
    async fn remove_connection(
        &self,
        request: Request<RemoveConnectionRequest>,
    ) -> Result<Response<RemoveConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let peer_id = UserId::from(parameters.peer_id);

        verify_id_or_admin(&request, &user_id).await?;

        let result = self
            .profiles_service
            .remove_connection(&user_id, &peer_id)
            .await;
        result.map_err(|_| Status::invalid_argument("Cannot remove connection"))?;

        Ok(Response::new(RemoveConnectionReply {}))
    }

    #[instrument(err, skip_all)]
    async fn update_connection(
        &self,
        request: Request<UpdateConnectionRequest>,
    ) -> Result<Response<UpdateConnectionReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let peer_id = UserId::from(parameters.peer_id);
        let relationship = &parameters.relationship;

        verify_id_or_admin(&request, &user_id).await?;

        let result = self
            .profiles_service
            .update_connection(&user_id, &peer_id, relationship)
            .await;
        result.map_err(|_| Status::invalid_argument("Cannot update connection"))?;

        Ok(Response::new(UpdateConnectionReply {}))
    }

    // Posts

    #[instrument(err, skip_all)]
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);
        let content = &parameters.content;

        verify_id_or_admin(&request, &user_id).await?;

        let result = self.profiles_service.create_post(&user_id, content).await;
        let post_id = result.map_err(|e| {
            event!(Level::WARN, "Cannot create post: {:?}", e);
            Status::invalid_argument("Cannot create post")
        })?;

        Ok(Response::new(CreatePostReply {
            post_id: post_id.to_string(),
        }))
    }

    #[instrument(err, skip_all)]
    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostReply>, Status> {
        let parameters = request.get_ref();
        let post_id = PostId::from(parameters.post_id.as_str());

        let (user_id, _, roles) = verify_auth(&request).await?;
        let user_id = if roles.contains(&Role::Admin) {
            None
        } else {
            Some(&user_id)
        };

        let result = self.profiles_service.delete_post(&post_id, user_id).await;
        result.map_err(|e| {
            event!(Level::WARN, "Cannot delete post: {:?}", e);
            Status::not_found("Cannot delete post")
        })?;

        Ok(Response::new(DeletePostReply {}))
    }

    #[instrument(err, skip_all)]
    async fn list_profile_posts(
        &self,
        request: Request<ListProfilePostsRequest>,
    ) -> Result<Response<ListProfilePostsReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        verify_id_or_admin(&request, &user_id).await?;

        let result = self.profiles_service.list_profile_posts(&user_id).await;
        let proto_posts = result.map_err(|_| Status::internal("Cannot list profile posts"))?;
        let posts = proto_posts
            .into_iter()
            .map(|post| post.try_into())
            .try_collect()?;

        Ok(Response::new(ListProfilePostsReply { posts }))
    }

    #[instrument(err, skip_all)]
    async fn list_feed_posts(
        &self,
        request: Request<ListFeedPostsRequest>,
    ) -> Result<Response<ListFeedPostsReply>, Status> {
        let parameters = request.get_ref();
        let user_id = UserId::from(parameters.user_id);

        verify_id_or_admin(&request, &user_id).await?;

        let result = self.profiles_service.list_feed_posts(&user_id).await;
        let proto_posts = result.map_err(|_| Status::internal("Cannot list profile posts"))?;
        let posts = proto_posts
            .into_iter()
            .map(|post| post.try_into())
            .try_collect()?;

        Ok(Response::new(ListFeedPostsReply { posts }))
    }
}
