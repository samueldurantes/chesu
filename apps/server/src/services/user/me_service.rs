use crate::http::Result;
use crate::models::user::User;
use crate::repositories::user_repository::UserRepositoryTrait;
use uuid::Uuid;

pub struct MeService<R: UserRepositoryTrait> {
    user_repository: R,
}

impl<R: UserRepositoryTrait> MeService<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<User> {
        Ok(self.user_repository.find_by_id(user_id).await?)
    }
}
