use uuid::Uuid;

use std::collections::HashMap;

use pbkdf2::{
    Pbkdf2,
    password_hash::{PasswordHasher, PasswordVerifier},
    phc::PasswordHash,
};

pub trait Users {
    fn create_user(&mut self, username: String, password: String) -> Result<(), String>;
    fn get_user_uuid(&self, username: &str, password: &str) -> Option<String>;
    fn delete_user(&mut self, user_uuid: String);
}

#[derive(Clone)]
pub struct User {
    user_uuid: String,
    username: String,
    password: String,
}

#[derive(Default)]
pub struct UsersImpl {
    uuid_to_user: HashMap<String, User>,
    username_to_user: HashMap<String, User>,
}

impl Users for UsersImpl {
    fn create_user(&mut self, username: String, password: String) -> Result<(), String> {
        if self.username_to_user.contains_key(&username) {
            return Err("Username already exists.".to_string());
        }

        let user = User {
            username,
            user_uuid: Uuid::new_v4().to_string(),
            password: Pbkdf2::default()
                .hash_password(password.as_bytes())
                .map_err(|_| "Failed to hash password.")?
                .to_string(),
        };

        self.uuid_to_user
            .insert(user.user_uuid.clone(), user.clone());
        self.username_to_user
            .insert(user.username.clone(), user.clone());

        Ok(())
    }

    fn get_user_uuid(&self, username: &str, password: &str) -> Option<String> {
        let Some(user) = self.username_to_user.get(username) else {
            return None;
        };

        let parsed_hash = PasswordHash::new(&user.password).ok()?;

        Pbkdf2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .ok()
            .map(|_| user.user_uuid.clone())
    }

    fn delete_user(&mut self, user_uuid: String) {
        if let Some(user) = self.uuid_to_user.remove(&user_uuid) {
            self.username_to_user.remove(&user.username);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_user() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        assert_eq!(user_service.uuid_to_user.len(), 1);
        assert_eq!(user_service.username_to_user.len(), 1);
    }

    #[test]
    fn should_fail_creating_user_with_existing_username() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        let result = user_service.create_user("username".to_owned(), "password".to_owned());

        assert!(result.is_err());
    }

    #[test]
    fn should_retrieve_user_uuid() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        assert!(user_service.get_user_uuid("username", "password").is_some());
    }

    #[test]
    fn should_fail_to_retrieve_user_uuid_with_incorrect_password() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        assert!(
            user_service
                .get_user_uuid("username", "incorrect password")
                .is_none()
        );
    }

    #[test]
    fn should_delete_user() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        let user_uuid = user_service.get_user_uuid("username", "password").unwrap();

        user_service.delete_user(user_uuid);

        assert_eq!(user_service.uuid_to_user.len(), 0);
        assert_eq!(user_service.username_to_user.len(), 0);
    }
}
