use rustls::internal::pemfile::{certs, rsa_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;

/// Load a certificate from `filename`.
pub fn load_cert(filename: &PathBuf) -> std::io::Result<Vec<rustls::Certificate>> {
    let certfile = File::open(filename)?;
    let mut reader = BufReader::new(certfile);
    certs(&mut reader)
        .map_err(|_| IoError::new(IoErrorKind::Other, "File contains an invalid certificate"))
}

/// Load a private key from `filename`.
pub fn load_private_key(filename: &PathBuf) -> std::io::Result<rustls::PrivateKey> {
    let rsa_keys = {
        let keyfile = File::open(filename)?;
        let mut reader = BufReader::new(keyfile);
        rsa_private_keys(&mut reader).map_err(|_| {
            IoError::new(IoErrorKind::Other, "File contains invalid RSA private key")
        })?
    };

    let pkcs8_keys = {
        let keyfile = File::open(filename)?;
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::pkcs8_private_keys(&mut reader).map_err(|_| {
            IoError::new(
                IoErrorKind::Other,
                "File contains invalid pkcs8 private key (encrypted keys not supported)",
            )
        })?
    };

    // prefer to load pkcs8 keys
    if !pkcs8_keys.is_empty() {
        Ok(pkcs8_keys[0].clone())
    } else {
        assert!(!rsa_keys.is_empty());
        Ok(rsa_keys[0].clone())
    }
}
