#[cfg(not(any(feature = "default", feature = "__rustcrypto-tls")))]
compile_error! {"You must either have the default feature enabled (remove
the no-default-features rust argument) or the no-c-deps feature"}

#[cfg(feature = "default")]
pub fn new() -> reqwest::blocking::Client {
    reqwest::blocking::Client::new()
}

#[cfg(feature = "default")]
pub fn new_async() -> reqwest::Client {
    reqwest::Client::new()
}

#[cfg(feature = "__rustcrypto-tls")]
pub fn new() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .use_preconfigured_tls(tls_config())
        .build()
        .expect("Could not create HTTP client.")
}

#[cfg(feature = "__rustcrypto-tls")]
pub fn new_async() -> reqwest::Client {
    reqwest::Client::builder()
        .use_preconfigured_tls(tls_config())
        .build()
        .expect("Could not create HTTP client.")
}

#[cfg(feature = "__rustcrypto-tls")]
fn tls_config() -> rustls::ClientConfig {
    use std::sync::Arc;

    let root_store = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };

    let provider = Arc::new(rustls_rustcrypto::provider());
    rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("Should support safe default protocols")
        .with_root_certificates(root_store)
        .with_no_client_auth()
}
