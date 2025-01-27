use enigma_auth::UserId;
use enigma_profiles::EnigmaUserProfile;
use enigma_profiles_grpc::{
    CreateProfileRequest, DeleteProfileRequest, GetProfileRequest, UpdateProfileRequest,
};
use tonic::{IntoRequest, Request};

pub struct CreateProfileParameters(pub EnigmaUserProfile);

impl IntoRequest<CreateProfileRequest> for CreateProfileParameters {
    fn into_request(self) -> Request<CreateProfileRequest> {
        Request::new(CreateProfileRequest {
            // TODO(ENIGMA-26): Parameter conversion must not fail
            profile: Some(self.0.try_into().unwrap()),
        })
    }
}

pub struct DeleteProfileParameters(pub UserId);

impl IntoRequest<DeleteProfileRequest> for DeleteProfileParameters {
    fn into_request(self) -> Request<DeleteProfileRequest> {
        Request::new(DeleteProfileRequest {
            user_id: self.0.into(),
        })
    }
}

pub struct GetProfileParameters(pub UserId);

impl IntoRequest<GetProfileRequest> for GetProfileParameters {
    fn into_request(self) -> Request<GetProfileRequest> {
        Request::new(GetProfileRequest {
            user_id: self.0.into(),
        })
    }
}

pub struct UpdateProfileParameters(pub EnigmaUserProfile);

impl IntoRequest<UpdateProfileRequest> for UpdateProfileParameters {
    fn into_request(self) -> Request<UpdateProfileRequest> {
        Request::new(UpdateProfileRequest {
            profile: Some(self.0.try_into().unwrap()),
        })
    }
}
