use super::auth_errors::AuthorizerError;
use ed25519_dalek::Verifier;
use ed25519_dalek::{Signature, VerifyingKey};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

pub(crate) trait AuthorizationProvider {
    async fn is_authorized(
        &self,
        signature: Signature,
        data_hash: &[u8],
    ) -> Result<bool, AuthorizerError>;
    async fn authorize(&self, public_key: VerifyingKey) -> Result<(), AuthorizerError>;

    #[cfg(test)]
    async fn is_key_authorized(&self, public_key: VerifyingKey) -> Result<bool, AuthorizerError>;
}

#[derive(Debug, Clone)]
pub enum Authorizer {
    Open,
    Persistent(FileAuthorizer),
}

impl AuthorizationProvider for Authorizer {
    async fn is_authorized(
        &self,
        signature: Signature,
        data_hash: &[u8],
    ) -> Result<bool, AuthorizerError> {
        Ok(match self {
            Authorizer::Open => true,
            Authorizer::Persistent(authorizer) => {
                authorizer.is_authorized(signature, data_hash).await?
            }
        })
    }
    async fn authorize(&self, public_key: VerifyingKey) -> Result<(), AuthorizerError> {
        match self {
            Authorizer::Open => Ok(()),
            Authorizer::Persistent(authorizer) => authorizer.authorize(public_key).await,
        }
    }

    #[cfg(test)]
    async fn is_key_authorized(&self, public_key: VerifyingKey) -> Result<bool, AuthorizerError> {
        Ok(match self {
            Authorizer::Open => true,
            Authorizer::Persistent(authorizer) => authorizer.is_key_authorized(public_key).await?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileAuthorizer(PathBuf);

impl FileAuthorizer {
    pub async fn new(path: PathBuf) -> Result<Self, AuthorizerError> {
        if !path.exists() {
            tokio::fs::write(&path, "[]")
                .await
                .map_err(AuthorizerError::FileAccessError)?;
        } else {
            File::open(&path)
                .await
                .map_err(AuthorizerError::FileAccessError)?;
        }
        Ok(Self(path))
    }
}
impl AuthorizationProvider for FileAuthorizer {
    async fn is_authorized(
        &self,
        signature: Signature,
        data_hash: &[u8],
    ) -> Result<bool, AuthorizerError> {
        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        if contents.trim().is_empty() {
            return Ok(false);
        }

        let serialized_keys: Vec<String> =
            serde_json::from_str(&contents).map_err(AuthorizerError::FormatError)?;

        for key in serialized_keys.iter() {
            let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(key)
                .map_err(|e| AuthorizerError::PrefixHexConversionError(e.to_string()))?;
            let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes.try_into()?)?;
            if verifying_key.verify(data_hash, &signature).is_ok() {
                return Ok(true);
            }
        }
        Ok(false)
    }
    #[cfg(test)]
    async fn is_key_authorized(&self, public_key: VerifyingKey) -> Result<bool, AuthorizerError> {
        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        if contents.trim().is_empty() {
            return Ok(false);
        }

        let serialized_keys: Vec<String> =
            serde_json::from_str(&contents).map_err(AuthorizerError::FormatError)?;

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
        let mut contents = String::new();

        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut serialized_keys: Vec<String> =
            serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new());

        for key in serialized_keys.iter() {
            let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(key)
                .map_err(|e| AuthorizerError::PrefixHexConversionError(e.to_string()))?;
            let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes.try_into()?)?;
            if verifying_key == public_key {
                return Ok(());
            }
        }
        serialized_keys.push(prefix_hex::encode(public_key.to_bytes()));
        let new_contents = serde_json::to_string(&serialized_keys)
            .map_err(AuthorizerError::FormatError)?
            .as_bytes()
            .to_vec();

        tokio::fs::write(&self.0, new_contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{ed25519::signature::Signer, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;
    use sha2::Digest;
    use tempfile::tempdir;
    use tokio::fs;

    fn generate_signing_key() -> SigningKey {
        SigningKey::generate(&mut OsRng)
    }

    fn generate_verifying_key(signing_key: &SigningKey) -> VerifyingKey {
        signing_key.verifying_key()
    }

    #[tokio::test]
    async fn test_authorize_new_key_in_empty_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);
        authorizer.authorize(public_key).await.unwrap();

        assert!(authorizer.is_key_authorized(public_key).await.unwrap());

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_authorize_existing_key() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);
        authorizer.authorize(public_key).await.unwrap();

        // Try authorizing the same key again
        authorizer.authorize(public_key).await.unwrap();

        // Verify the key is still authorized
        assert!(authorizer.is_key_authorized(public_key).await.unwrap());

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_is_authorized_with_empty_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        // Create a new FileAuthorizer (this should create an empty file)
        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Generate a new key (but don't authorize it)
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        // Verify that the key is not authorized
        assert!(!authorizer.is_key_authorized(public_key).await.unwrap());

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_authorize_and_check_multiple_keys() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Generate and authorize multiple keys
        let mut keys = Vec::new();
        for _ in 0..5 {
            let signing_key = generate_signing_key();
            let public_key = generate_verifying_key(&signing_key);
            authorizer.authorize(public_key).await.unwrap();
            keys.push(public_key);
        }

        // Verify all keys are authorized
        for key in keys.iter() {
            assert!(authorizer.is_key_authorized(*key).await.unwrap());
        }

        temp_dir.close().unwrap();
    }
    #[tokio::test]
    async fn test_authorize_with_prepopulated_keys() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        // Prepopulate the file with some keys
        let mut initial_keys = Vec::new();
        for _ in 0..3 {
            let signing_key = generate_signing_key();
            let public_key = generate_verifying_key(&signing_key);
            initial_keys.push(prefix_hex::encode(public_key.to_bytes()));
        }

        // Serialize the keys and write them to the file
        let serialized_keys = serde_json::to_string(&initial_keys).unwrap();
        fs::write(&file_path, serialized_keys).await.unwrap();

        // Create a FileAuthorizer that will use the prepopulated file
        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Verify that the initial keys are authorized
        for encoded_key in &initial_keys {
            let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(encoded_key).unwrap();
            let verifying_key =
                VerifyingKey::from_bytes(&verifying_key_bytes.try_into().unwrap()).unwrap();
            assert!(authorizer.is_key_authorized(verifying_key).await.unwrap());
        }

        // Add a new key and verify it
        let new_signing_key = generate_signing_key();
        let new_public_key = generate_verifying_key(&new_signing_key);
        authorizer.authorize(new_public_key).await.unwrap();

        // Verify that the new key is authorized
        assert!(authorizer.is_key_authorized(new_public_key).await.unwrap());

        // Clean up
        temp_dir.close().unwrap();
    }
    #[tokio::test]
    async fn test_is_authorized_valid_signature() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Generate key pair
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        // Authorize the public key
        authorizer.authorize(public_key).await.unwrap();

        // Create data to sign
        let data = b"test data";
        let hash = sha2::Sha256::digest(data);
        let signature = signing_key.sign(&hash);

        // Check authorization
        let is_auth = authorizer.is_authorized(signature, &hash).await.unwrap();
        assert!(is_auth);

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_is_authorized_invalid_signature() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Generate key pair
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        // Authorize the public key
        authorizer.authorize(public_key).await.unwrap();

        // Create data but do NOT sign it with the authorized key
        let wrong_signing_key = generate_signing_key();
        let wrong_hash = sha2::Sha256::digest(b"wrong data");
        let wrong_signature = wrong_signing_key.sign(&wrong_hash);

        // Check authorization (should be false)
        let is_auth = authorizer
            .is_authorized(wrong_signature, &wrong_hash)
            .await
            .unwrap();
        assert!(!is_auth);

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_is_authorized_with_empty_authorizer() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Generate key pair but do NOT authorize it
        let signing_key = generate_signing_key();
        let hash = sha2::Sha256::digest(b"test data");
        let signature = signing_key.sign(&hash);

        // Check authorization (should be false)
        let is_auth = authorizer.is_authorized(signature, &hash).await.unwrap();
        assert!(!is_auth);

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_is_authorized_multiple_keys() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Generate multiple keys and authorize them
        let mut signing_keys = Vec::new();
        let mut verifying_keys = Vec::new();
        for _ in 0..3 {
            let signing_key = generate_signing_key();
            let public_key = generate_verifying_key(&signing_key);
            authorizer.authorize(public_key).await.unwrap();
            signing_keys.push(signing_key);
            verifying_keys.push(public_key);
        }

        // Sign data with each key and check authorization
        for signing_key in signing_keys.iter() {
            let hash = sha2::Sha256::digest(b"test data");
            let signature = signing_key.sign(&hash);
            let is_auth = authorizer.is_authorized(signature, &hash).await.unwrap();
            assert!(is_auth);
        }

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_is_authorized_with_modified_data() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authorized_keys.json");

        let authorizer = FileAuthorizer::new(file_path.clone()).await.unwrap();

        // Generate key pair
        let signing_key = generate_signing_key();
        let public_key = generate_verifying_key(&signing_key);

        // Authorize the public key
        authorizer.authorize(public_key).await.unwrap();

        // Sign original data
        let original_data = b"original data";
        let modified_data = b"modified data";
        let original_hash = sha2::Sha256::digest(original_data);
        let modified_hash = sha2::Sha256::digest(modified_data);
        let signature = signing_key.sign(&original_hash);

        // Verify authorization with modified data (should be false)
        let is_auth = authorizer
            .is_authorized(signature, &modified_hash)
            .await
            .unwrap();
        assert!(!is_auth);

        temp_dir.close().unwrap();
    }
}
