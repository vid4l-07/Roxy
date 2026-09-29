use rcgen::{BasicConstraints, Certificate, CertificateParams, IsCa, KeyPair};

use tokio_rustls::rustls::{pki_types::{CertificateDer, PrivateKeyDer}};

use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};

use std::{fs::{self, File}, io::BufReader, sync::Arc};

pub fn generate_ca() -> Result<(Certificate, KeyPair), rcgen::Error> {
    let key_pair = KeyPair::generate()?;

    let mut params = CertificateParams::new(vec![])?;

    params.distinguished_name.push(rcgen::DnType::CommonName, "Roxy CA");

    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    let certificate = params.self_signed(&key_pair)?;

    Ok((certificate, key_pair))
}

pub fn generate_certificate(ca: &Certificate, ca_key: &KeyPair, host: &str) -> Result<(Certificate, KeyPair), rcgen::Error> {
    let key_pair = KeyPair::generate()?;

    let mut params = CertificateParams::new(vec![ host.to_string() ])?;

    params.distinguished_name.push(rcgen::DnType::CommonName, host);

    let certificate = params.signed_by(&key_pair, ca, ca_key)?;

    Ok((certificate, key_pair))
}

pub fn save_ca(certificate: &Certificate, key_pair: &KeyPair) -> std::io::Result<()> {
    fs::create_dir_all("certs")?;

    fs::write("certs/ca.crt", certificate.pem())?;

    fs::write("certs/ca.key", key_pair.serialize_pem())?;

    Ok(())
}

pub fn save_certificate(certificate: &Certificate, key_pair: &KeyPair, host: &str) -> std::io::Result<()> {
    std::fs::write( format!("certs/{host}.crt"), certificate.pem())?;

    std::fs::write( format!("certs/{host}.key"), key_pair.serialize_pem())?;

    Ok(())
}

pub fn load_certificates(path: &str) -> std::io::Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })
}

pub fn load_private_key(path: &str) -> std::io::Result<PrivateKeyDer<'static>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    rustls_pemfile::private_key(&mut reader)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })?
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No private key found",
            )
        })
}

pub fn create_tls_acceptor() -> std::io::Result<TlsAcceptor> {
    let certificates = load_certificates(
        "certs/localhost.crt"
    )?;

    let private_key = load_private_key(
        "certs/localhost.key"
    )?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, private_key)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
