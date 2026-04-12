use base64::{engine::general_purpose::STANDARD, Engine};
use rcgen::{BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct LocalCA {
    cert_pem: String,
    key_pair: KeyPair,
}

impl LocalCA {
    /// Load existing CA from disk, or create a new one.
    pub fn load_or_create(data_dir: &Path) -> anyhow::Result<Self> {
        let cert_path = data_dir.join("ca.crt");
        let key_path = data_dir.join("ca.key");

        if cert_path.exists() && key_path.exists() {
            let cert_pem = std::fs::read_to_string(&cert_path)?;
            let key_pem = std::fs::read_to_string(&key_path)?;
            let key_pair = KeyPair::from_pem(&key_pem)?;
            tracing::info!("Loaded existing CA from {}", cert_path.display());
            return Ok(Self { cert_pem, key_pair });
        }

        // Generate new CA
        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
        ];

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "DevHost Local CA");
        dn.push(DnType::OrganizationName, "DevHost");
        params.distinguished_name = dn;

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        std::fs::create_dir_all(data_dir)?;
        std::fs::write(&cert_path, &cert_pem)?;
        std::fs::write(&key_path, &key_pem)?;

        tracing::info!("Created new CA at {}", cert_path.display());
        Ok(Self { cert_pem, key_pair })
    }

    /// Issue a leaf certificate for a specific domain.
    /// Returns `(cert_pem, key_pem)`.
    pub fn issue_for_domain(&self, domain: &str) -> anyhow::Result<(String, String)> {
        // CertificateParams::new takes Vec<String> for subject_alt_names
        let san_strings: Vec<String> = vec![domain.to_string(), format!("*.{}", domain)];

        let mut params = CertificateParams::new(san_strings)?;

        // Ensure leaf certificate validity doesn't exceed modern browser maximums
        // using arbitrary safe recent dates within ~2 year limits if possible.
        // Actually rcgen defaults can trigger 398/825 day limits.
        params.not_before = rcgen::date_time_ymd(2026, 1, 1);
        params.not_after = rcgen::date_time_ymd(2027, 1, 1);

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, domain);
        params.distinguished_name = dn;

        // Re-create the CA cert from stored PEM for signing
        let ca_key_pair = &self.key_pair;

        // Build a temporary CA params with exact same DN to get a Certificate for signing.
        // AKI is derived from the ca_key_pair public key, which perfectly matches the trust store.
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
        ];
        let mut ca_dn = DistinguishedName::new();
        ca_dn.push(DnType::CommonName, "DevHost Local CA");
        ca_dn.push(DnType::OrganizationName, "DevHost");
        ca_params.distinguished_name = ca_dn;
        let ca_cert = ca_params.self_signed(ca_key_pair)?;

        let leaf_key = KeyPair::generate()?;
        let leaf_cert = params.signed_by(&leaf_key, &ca_cert, ca_key_pair)?;

        tracing::info!("Issued certificate for {}", domain);
        Ok((leaf_cert.pem(), leaf_key.serialize_pem()))
    }

    pub fn cert_pem(&self) -> &str {
        &self.cert_pem
    }

    /// Compute the SHA-256 fingerprint of the CA cert (DER bytes).
    /// Returns colon-separated uppercase hex, e.g. "AB:CD:EF:..."
    pub fn fingerprint(&self) -> Option<String> {
        // Strip PEM header/footer and decode base64 → DER bytes
        let b64: String = self
            .cert_pem
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect();
        let der = STANDARD.decode(b64.trim()).ok()?;

        let hash = Sha256::digest(&der);
        let fingerprint = hash
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":");
        Some(fingerprint)
    }
}
