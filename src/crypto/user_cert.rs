use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use pbkdf2::pbkdf2_hmac;
use rcgen::{CertificateParams, DnType, KeyPair, KeyUsagePurpose, date_time_ymd};
use sha2::Sha256;

use super::ca::CaBundle;

/// Result of user certificate generation
pub struct UserCertificate {
    /// X.509 certificate in PEM format
    pub certificate_pem: String,
    /// Encrypted private key (base64 encoded)
    pub encrypted_private_key: String,
}

/// Errors that can occur during user certificate generation
#[derive(Debug)]
pub enum UserCertError {
    /// Error generating RSA keypair
    KeyGeneration(String),
    /// Error creating certificate
    CertificateCreation(String),
    /// Error encrypting private key
    Encryption(String),
}

impl std::fmt::Display for UserCertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserCertError::KeyGeneration(msg) => write!(f, "Key generation error: {}", msg),
            UserCertError::CertificateCreation(msg) => write!(f, "Certificate creation error: {}", msg),
            UserCertError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
        }
    }
}

impl std::error::Error for UserCertError {}

/// Generate a certificate for an organizer
pub fn generate_organizer_certificate(
    organization: &str,
    identification_number: &str,
    password: &str,
    ca_bundle: &CaBundle,
) -> Result<UserCertificate, UserCertError> {
    let key_pair = KeyPair::generate()
        .map_err(|e| UserCertError::KeyGeneration(e.to_string()))?;

    let mut params = CertificateParams::default();
    
    // Set certificate validity period
    params.not_before = date_time_ymd(2025, 1, 1);
    params.not_after = date_time_ymd(2030, 1, 1);
    
    // Set organizer-specific distinguished name
    params.distinguished_name.push(DnType::CommonName, organization);
    params.distinguished_name.push(DnType::OrganizationName, organization);
    params.distinguished_name.push(DnType::OrganizationalUnitName, "Organizer");
    // Note: identification number will be included in SAN instead of DN
    
    // Set key usage for organizers (digital signature, key encipherment)
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
        KeyUsagePurpose::DataEncipherment,
    ];
    
    // Add subject alternative name with identification number  
    let dns_name = format!("org-{}.secure-vottyng.etf.unibl", identification_number);
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(dns_name.try_into().unwrap()),
    ];

    let ca_issuer = ca_bundle.issuer();
    let certificate = params.signed_by(&key_pair, &ca_issuer)
        .map_err(|e| UserCertError::CertificateCreation(e.to_string()))?;
    
    let certificate_pem = certificate.pem();
    let private_key_pem = key_pair.serialize_pem();
    
    let encrypted_private_key = encrypt_private_key(&private_key_pem, password)?;
    
    Ok(UserCertificate {
        certificate_pem,
        encrypted_private_key,
    })
}

/// Generate a certificate for a voter
pub fn generate_voter_certificate(
    first_name: &str,
    last_name: &str,
    username: &str,
    password: &str,
    ca_bundle: &CaBundle,
) -> Result<UserCertificate, UserCertError> {
    let key_pair = KeyPair::generate()
        .map_err(|e| UserCertError::KeyGeneration(e.to_string()))?;

    let mut params = CertificateParams::default();
    
    // Set certificate validity period
    params.not_before = date_time_ymd(2025, 1, 1);
    params.not_after = date_time_ymd(2030, 1, 1);
    
    // Set voter-specific distinguished name
    let full_name = format!("{} {}", first_name, last_name);
    params.distinguished_name.push(DnType::CommonName, &full_name);
    params.distinguished_name.push(DnType::OrganizationName, "Secure Vottyng System");
    params.distinguished_name.push(DnType::OrganizationalUnitName, "Voter");
    // Note: first_name, last_name, and username will be included in SAN instead of DN
    
    // Set key usage for voters (digital signature for voting)
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
    ];
    
    // Add subject alternative name with username
    let dns_name = format!("voter-{}.secure-vottyng.etf.unibl", username);
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(dns_name.try_into().unwrap()),
    ];

    let ca_issuer = ca_bundle.issuer();
    let certificate = params.signed_by(&key_pair, &ca_issuer)
        .map_err(|e| UserCertError::CertificateCreation(e.to_string()))?;
    
    let certificate_pem = certificate.pem();
    let private_key_pem = key_pair.serialize_pem();
    
    let encrypted_private_key = encrypt_private_key(&private_key_pem, password)?;
    
    Ok(UserCertificate {
        certificate_pem,
        encrypted_private_key,
    })
}

/// Encrypt a private key using the user's password
fn encrypt_private_key(private_key_pem: &str, password: &str) -> Result<String, UserCertError> {
    // Generate a random salt for key derivation
    let mut salt = [0u8; 16];
    use rand_core::RngCore;
    OsRng.fill_bytes(&mut salt);
    
    // Derive encryption key from password using PBKDF2
    let mut key = [0u8; 32]; // 256 bits for AES-256
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);
    
    // Generate random nonce for AES-GCM
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt the private key
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| UserCertError::Encryption(format!("Failed to create cipher: {}", e)))?;
    
    let ciphertext = cipher.encrypt(nonce, private_key_pem.as_bytes())
        .map_err(|e| UserCertError::Encryption(format!("Encryption failed: {}", e)))?;
    
    // Combine salt + nonce + ciphertext and encode as base64
    let mut combined = Vec::new();
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(combined))
}

/// Decrypt a private key using the user's password
pub fn decrypt_private_key(encrypted_key: &str, password: &str) -> Result<String, UserCertError> {
    // Decode from base64
    let combined = general_purpose::STANDARD.decode(encrypted_key)
        .map_err(|e| UserCertError::Encryption(format!("Base64 decode failed: {}", e)))?;
    
    if combined.len() < 16 + 12 {
        return Err(UserCertError::Encryption("Invalid encrypted key format".to_string()));
    }
    
    // Extract salt, nonce, and ciphertext
    let salt = &combined[0..16];
    let nonce_bytes = &combined[16..28];
    let ciphertext = &combined[28..];
    
    // Derive the same key from password
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    
    // Decrypt
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| UserCertError::Encryption(format!("Failed to create cipher: {}", e)))?;
    
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| UserCertError::Encryption(format!("Decryption failed: {}", e)))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| UserCertError::Encryption(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_private_key_encryption_decryption() {
        let original_key = "-----BEGIN PRIVATE KEY-----\ntest_key_content\n-----END PRIVATE KEY-----";
        let password = "test_password123";
        
        let encrypted = encrypt_private_key(original_key, password).expect("Encryption failed");
        let decrypted = decrypt_private_key(&encrypted, password).expect("Decryption failed");
        
        assert_eq!(original_key, decrypted);
        
        // Test with wrong password
        let wrong_result = decrypt_private_key(&encrypted, "wrong_password");
        assert!(wrong_result.is_err());
    }
}
