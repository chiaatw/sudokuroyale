use sqlx::PgPool;
use uuid::Uuid;
use crate::user::model::User;

pub struct UserRepository {
    pool: PgPool,
}
impl UserRepository {
    pub fn new (pool: PgPool) -> Self {
        Self { pool  }
    }
   
   
   
   // platzhalter für spätere änderungen 




    pub fn add_user(&mut self, _user: User) {
        todo!("add_user wird später mit SQL implementiert");
    }

    pub fn find_by_username(&self, _username: &str) -> Option<User> {
        todo!("find_by_username wird später mit SQL implementiert");
    }

    pub fn find_by_email(&self, _email: &str) -> Option<User> {
        todo!("find_by_email wird später mit SQL implementiert");
    }

    pub fn find_by_id(&self, _id: &Uuid) -> Option<User> {
        todo!("find_by_id wird später mit SQL implementiert");
    }

    pub fn update_password(&mut self, _user_id: &Uuid, _new_hash: String) -> bool {
        todo!("update_password wird später mit SQL implementiert");
    }
}


 