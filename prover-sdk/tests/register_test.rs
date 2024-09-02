use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};
use url::Url;

#[tokio::test]
async fn test_register_authorized() {
    let private_key = std::env::var("PRIVATE_KEY");
    let url = std::env::var("PROVER_URL");
    let admin_private_key = std::env::var("ADMIN_PRIVATE_KEY");
    assert!(private_key.is_ok());
    assert!(url.is_ok());
    assert!(admin_private_key.is_ok());
}
//     let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
//     let url = Url::parse(&url).unwrap();
//     let sdk = ProverSDK::new(url, access_key).await.unwrap();
// }
#[tokio::test]
async fn test_register_unauthorized() {
    // Test logic for attempting to register with unauthorized access
}