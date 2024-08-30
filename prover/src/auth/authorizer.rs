use super::auth_errors::AuthorizerError;
use ed25519_dalek::VerifyingKey;
use std::{collections::HashSet, path::PathBuf, sync::Arc};
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex};

pub(crate) trait AuthorizationProvider {
    async fn is_authorized(&self, public_key: VerifyingKey) -> Result<bool, AuthorizerError>;

    #[allow(dead_code)] //this function will be used later in endpoint /register to add a new public key to the authorized keys
                        //for now skip highlighting this function
    async fn authorize(&self, public_key: VerifyingKey) -> Result<(), AuthorizerError>;
}

#[derive(Debug, Clone)]
pub enum Authorizer {
    Open,
    Persistent(FileAuthorizer),
    ReadOnlyFile(FileAuthorizer, MemoryAuthorizer),
    Memory(MemoryAuthorizer),
}

impl AuthorizationProvider for Authorizer {
    async fn is_authorized(&self, public_key: VerifyingKey) -> Result<bool, AuthorizerError> {
        Ok(match self {
            Authorizer::Open => true,
            Authorizer::Persistent(authorizer) => authorizer.is_authorized(public_key).await?,
            Authorizer::Memory(authorizer) => authorizer.is_authorized(public_key).await?,
            Authorizer::ReadOnlyFile(file_authorizer, memory_authorizer) => {
                memory_authorizer.is_authorized(public_key).await?
                    || file_authorizer.is_authorized(public_key).await?
            }
        })
    }

    async fn authorize(&self, public_key: VerifyingKey) -> Result<(), AuthorizerError> {
        match self {
            Authorizer::Open => Ok(()),
            Authorizer::Persistent(authorizer) => authorizer.authorize(public_key).await,
            Authorizer::Memory(authorizer) => authorizer.authorize(public_key).await,
            Authorizer::ReadOnlyFile(_, memory_authorizer) => {
                memory_authorizer.authorize(public_key).await
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileAuthorizer(PathBuf);

impl FileAuthorizer {
    pub async fn new(path: PathBuf) -> Result<Self, AuthorizerError> {
        File::open(&path)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        Ok(Self(path))
    }
}

impl AuthorizationProvider for FileAuthorizer {
    async fn is_authorized(&self, public_key: VerifyingKey) -> Result<bool, AuthorizerError> {
        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;
        let serialized_keys =
            serde_json::from_str::<Vec<String>>(&contents).map_err(AuthorizerError::FormatError)?;
        for key in serialized_keys.iter() {
            let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(key)
                .map_err(|e| AuthorizerError::PrefixHexConversionError(e.to_string()))?;
            let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes.try_into()?)?;
            if verifying_key == public_key {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn authorize(&self, public_key: VerifyingKey) -> Result<(), AuthorizerError> {
        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut authorized_keys: HashSet<VerifyingKey> =
            serde_json::from_str::<Vec<VerifyingKey>>(&contents)
                .map_err(AuthorizerError::FormatError)?
                .into_iter()
                .collect();

        authorized_keys.insert(public_key);

        let new_contents =
            serde_json::to_string(&authorized_keys).map_err(AuthorizerError::FormatError)?;

        tokio::fs::write(&self.0, new_contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MemoryAuthorizer(Arc<Mutex<HashSet<VerifyingKey>>>);

impl AuthorizationProvider for MemoryAuthorizer {
    async fn is_authorized(&self, public_key: VerifyingKey) -> Result<bool, AuthorizerError> {
        println!("{}", self.0.lock().await.contains(&public_key));
        Ok(self.0.lock().await.contains(&public_key))
    }

    async fn authorize(&self, public_key: VerifyingKey) -> Result<(), AuthorizerError> {
        self.0.lock().await.insert(public_key);
        Ok(())
    }
}

impl From<Vec<VerifyingKey>> for MemoryAuthorizer {
    fn from(keys: Vec<VerifyingKey>) -> Self {
        Self(Arc::new(Mutex::new(keys.into_iter().collect())))
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use rand::rngs::OsRng;
    use tempfile::tempdir;
    use tokio::fs;

    fn generate_signing_key() -> SigningKey {
        SigningKey::generate(&mut OsRng)
    }

    fn generate_verifying_key(signing_key: &SigningKey) -> VerifyingKey {
        signing_key.verifying_key()
    }

    #[tokio::test]
    async fn test_serialize_deserialize_verifying_key() {
        let mut path = PathBuf::new();
        path.push("authorized_keys.json");
        let mut verifying_keys: Vec<String> = Vec::new();
        let mut verifying_keys_deserialized: Vec<VerifyingKey> = Vec::new();
        for _ in 0..10 {
            let signing_key = generate_signing_key();
            let verifying_key = generate_verifying_key(&signing_key);
            let encoded = verifying_key.to_bytes();
            let encoded_strig = prefix_hex::encode(encoded);
            verifying_keys_deserialized.push(verifying_key);
            verifying_keys.push(encoded_strig);
        }
        let serialized_keys = serde_json::to_string(&verifying_keys).unwrap();
        fs::write(&path, serialized_keys).await.unwrap();

        let file = FileAuthorizer::new(path.clone()).await.unwrap();
        for key in verifying_keys_deserialized.iter() {
            assert!(file.is_authorized(*key).await.unwrap());
        }
    }
    #[tokio::test]
    async fn test_memory_authorizer() {
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        let authorizer = MemoryAuthorizer::from(vec![public_key]);
        assert!(authorizer.is_authorized(public_key).await.unwrap());

        let new_signing_key = generate_signing_key();
        let new_public_key = generate_verifying_key(&new_signing_key);
        assert!(!authorizer.is_authorized(new_public_key).await.unwrap());

        authorizer.authorize(new_public_key).await.unwrap();
        assert!(authorizer.is_authorized(new_public_key).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_authorizer() {
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        let file_path = PathBuf::from_str("authorized_keys.json").unwrap();

        fs::write(&file_path, serde_json::to_string(&public_key).unwrap())
            .await
            .unwrap();

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();
        assert!(authorizer.is_authorized(public_key).await.unwrap());

        let new_signing_key = generate_signing_key();
        let new_public_key = generate_verifying_key(&new_signing_key);
        assert!(!authorizer.is_authorized(new_public_key).await.unwrap());

        authorizer.authorize(new_public_key).await.unwrap();
        let updated_authorizer = FileAuthorizer::new(file_path).await.unwrap();
        assert!(updated_authorizer
            .is_authorized(new_public_key)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_read_only_file_authorizer() {
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("authorized_keys.json");

        fs::write(
            &file_path,
            serde_json::to_string(&vec![public_key]).unwrap(),
        )
        .await
        .unwrap();

        let file_authorizer = FileAuthorizer::new(file_path).await.unwrap();
        let memory_authorizer = MemoryAuthorizer::from(vec![]);

        let authorizer =
            Authorizer::ReadOnlyFile(file_authorizer.clone(), memory_authorizer.clone());
        assert!(authorizer.is_authorized(public_key).await.unwrap());

        let new_signing_key = generate_signing_key();
        let new_public_key = generate_verifying_key(&new_signing_key);
        assert!(!authorizer.is_authorized(new_public_key).await.unwrap());

        authorizer.authorize(new_public_key).await.unwrap();
        assert!(memory_authorizer
            .is_authorized(new_public_key)
            .await
            .unwrap());
        assert!(!file_authorizer.is_authorized(new_public_key).await.unwrap());
    }

    #[tokio::test]
    async fn test_open_authorizer() {
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        let authorizer = Authorizer::Open;
        assert!(authorizer.is_authorized(public_key).await.unwrap());
    }

    #[tokio::test]
    async fn test_persistent_authorizer() {
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("authorized_keys.json");

        fs::write(
            &file_path,
            serde_json::to_string(&vec![public_key]).unwrap(),
        )
        .await
        .unwrap();

        let file_authorizer = FileAuthorizer::new(file_path).await.unwrap();
        let authorizer = Authorizer::Persistent(file_authorizer.clone());

        assert!(authorizer.is_authorized(public_key).await.unwrap());

        let new_signing_key = generate_signing_key();
        let new_public_key = generate_verifying_key(&new_signing_key);
        assert!(!authorizer.is_authorized(new_public_key).await.unwrap());

        authorizer.authorize(new_public_key).await.unwrap();
        assert!(file_authorizer.is_authorized(new_public_key).await.unwrap());
    }
}
