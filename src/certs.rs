use rcgen::{BasicConstraints, Certificate, CertificateParams, IsCa, KeyPair};
use tokio_rustls::rustls::{pki_types::{CertificateDer, PrivateKeyDer}};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use std::{fs::{self, File}, io::BufReader, sync::Arc, path::PathBuf};

const CERTS_FOLDER: &str = "/tmp/roxy_certs";

fn ca_folder() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not found",
            )
        })?;

    Ok(PathBuf::from(home).join(".config").join("roxy"))
}

pub fn generate_ca() -> Result<(Certificate, KeyPair), rcgen::Error> {
    let key_pair = KeyPair::generate()?;

    let mut params = CertificateParams::new(vec![])?;

    params.distinguished_name.push(rcgen::DnType::CommonName, "Roxy CA");

    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    let certificate = params.self_signed(&key_pair)?;

    let _ = fs::remove_dir_all(CERTS_FOLDER);

    Ok((certificate, key_pair))
}

pub fn save_ca(certificate: &Certificate, key_pair: &KeyPair) -> std::io::Result<()> {
    let folder = ca_folder()?;

    fs::create_dir_all(&folder)?;

    fs::write(folder.join("ca.crt"), certificate.pem())?;

    fs::write(folder.join("ca.key"), key_pair.serialize_pem())?;

    Ok(())
}

fn exists_ca() -> std::io::Result<bool> {
    let folder = ca_folder()?;

    Ok(folder.join("ca.crt").exists() && folder.join("ca.key").exists())
}

fn get_or_create_ca() -> std::io::Result<(Certificate, KeyPair)> {
    if exists_ca()? {
        let folder = ca_folder()?;

        let ca_pem = fs::read_to_string(folder.join("ca.crt"))?;

        let ca_key_pem = fs::read_to_string(folder.join("ca.key"))?;

        let ca_key = KeyPair::from_pem(&ca_key_pem)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )
            })?;

        let params = CertificateParams::from_ca_cert_pem(&ca_pem)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )
            })?;
        let ca = params.self_signed(&ca_key)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )
            })?;

        Ok((ca, ca_key))
    } else {
        let (ca, ca_key) = generate_ca()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )
            })?;

        save_ca(&ca, &ca_key)?;

        Ok((ca, ca_key))
    }
}


pub fn generate_certificate(ca: &Certificate, ca_key: &KeyPair, host: &str) -> Result<(Certificate, KeyPair), rcgen::Error> {
    let key_pair = KeyPair::generate()?;

    let mut params = CertificateParams::new(vec![ host.to_string() ])?;

    params.distinguished_name.push(rcgen::DnType::CommonName, host);

    let certificate = params.signed_by(&key_pair, ca, ca_key)?;

    Ok((certificate, key_pair))
}

pub fn save_certificate(certificate: &Certificate, key_pair: &KeyPair, host: &str) -> std::io::Result<()> {
    fs::create_dir_all(CERTS_FOLDER)?;

    std::fs::write( format!("{}/{}.crt", CERTS_FOLDER, host), certificate.pem())?;

    std::fs::write( format!("{}/{}.key", CERTS_FOLDER, host), key_pair.serialize_pem())?;

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

fn exists_cert(host: &str) -> bool {
    fs::metadata(format!("{}/{}.crt", CERTS_FOLDER, host)).is_ok() && fs::metadata(format!("{}/{}.key", CERTS_FOLDER, host)).is_ok()
}

fn get_or_create_certificate(ca: &Certificate, ca_key: &KeyPair, host: &str) -> std::io::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {

    if exists_cert(host) {
        let certificates = load_certificates(&format!("{}/{}.crt", CERTS_FOLDER, host))?;

        let private_key = load_private_key(&format!("{}/{}.key", CERTS_FOLDER, host))?;

        Ok((certificates, private_key))

    } else {
        let (certificate, key_pair) = generate_certificate(ca, ca_key, host)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )
            })?;

        save_certificate(&certificate, &key_pair, host)?;

        let certificates = vec![CertificateDer::from(certificate.der().to_vec())];

        let private_key = PrivateKeyDer::Pkcs8(
            key_pair.serialize_der().into()
        );

        Ok((certificates, private_key))
    }
}

pub fn create_tls_acceptor(host: &str) -> std::io::Result<TlsAcceptor> {
    let (ca, ca_key) = get_or_create_ca()?;

    let (certificates, private_key) = get_or_create_certificate(&ca, &ca_key, host)?;

    let config = ServerConfig::builder().with_no_client_auth().with_single_cert(certificates, private_key)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )
        })?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
