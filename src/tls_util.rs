use anyhow::{anyhow, ensure, Result};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Load a certificate from `filename`.
pub fn load_cert(filename: &PathBuf) -> Result<Vec<rustls::Certificate>> {
    let certfile = File::open(filename)?;
    let mut reader = BufReader::new(certfile);
    certs(&mut reader).map_err(|_| anyhow!("File contains an invalid certificate"))
}

/// Load a private key from `filename`.
pub fn load_private_key(filename: &PathBuf) -> Result<rustls::PrivateKey> {
    let rsa_keys = {
        let keyfile = File::open(filename)?;
        let mut reader = BufReader::new(keyfile);
        rsa_private_keys(&mut reader)
            .map_err(|_| anyhow!("File contains invalid RSA private key"))?
    };

    let pkcs8_keys = {
        let keyfile = File::open(filename)?;
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::pkcs8_private_keys(&mut reader).map_err(|_| {
            anyhow!("File contains invalid PKCS #8 private key (encrypted keys not supported)")
        })?
    };

    // prefer to load pkcs8 keys
    if !pkcs8_keys.is_empty() {
        Ok(pkcs8_keys[0].clone())
    } else {
        // At this point, our array of rsa_keys must not be empty.
        // If it is, loading the provided keys has failed entirely.
        ensure!(
            !rsa_keys.is_empty(),
            "Key doesn't seem to be RSA nor PKCS #8"
        );
        Ok(rsa_keys[0].clone())
    }
}
