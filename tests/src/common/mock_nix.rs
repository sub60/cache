use core::num::NonZeroU64;
use std::collections::HashMap;
use std::sync::Mutex;

use axum::body::{Body, Bytes};
use axum::http::{self, Request, StatusCode, header};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use sha2::{Digest, Sha256};
use tower_service::Service;
use types::{
    CompressionAlgorithm,
    NarFileName,
    NarInfo,
    Nix32Digest,
    NixStoreBaseName,
    NixStoreHash,
    NixStoreName,
    NixStorePath,
};

use crate::common::{CopyDirection, CopyOptions, NixClient, NixStore};

/// TODO: docs.
#[derive(Default)]
pub struct MockNixClient {
    store: MockNixStore,
}

/// TODO: docs.
#[derive(Default)]
pub struct MockNixStore {
    /// Maps store path basenames to their file contents.
    files: Mutex<HashMap<NixStoreBaseName, Bytes>>,
}

#[derive(Debug, derive_more::Display, cauchy::Error)]
pub enum CopyError {
    #[display("GET NAR failed with code {_0}")]
    GetNarFailed(http::StatusCode),
    #[display("GET NARInfo failed with code {_0}")]
    GetNarInfoFailed(http::StatusCode),
    #[display("no file at {_0}")]
    MissingStorePath(NixStorePath),
    #[display("PUT to NAR endpoint failed with code {_0}")]
    PutNarFailed(http::StatusCode),
    #[display("PUT to NARInfo endpoint failed with code {_0}")]
    PutNarInfoFailed(http::StatusCode),
    #[display("PUT to redirected NAR endpoint failed with code {_0}")]
    PutNarRedirectFailed(http::StatusCode),
}

impl MockNixStore {
    /// Returns the file contents of a previously added store path.
    fn get_file(&self, store_path: &NixStorePath) -> Option<Bytes> {
        #[expect(clippy::unwrap_used)]
        self.files.lock().unwrap().get(store_path.basename()).cloned()
    }
}

impl NixClient for MockNixClient {
    type Store = MockNixStore;
    type CacheHost = axum::Router;
    type CopyError = CopyError;

    fn store(&self) -> &Self::Store {
        &self.store
    }

    async fn copy_impl(
        &self,
        store_paths: Vec<NixStorePath>,
        options: CopyOptions<true, Self>,
    ) -> Result<(), CopyError> {
        let direction = options.direction;
        let (app, user, cache) = options.cache_uri();
        let base = format!("/{user}/{cache}");

        let mut futures = store_paths
            .into_iter()
            .map(|store_path| async {
                match direction {
                    CopyDirection::To => {
                        copy_store_path_to(
                            &self.store,
                            store_path,
                            &mut app.clone(),
                            &base,
                        )
                        .await
                    },
                    CopyDirection::From => {
                        copy_store_path_from(
                            &self.store,
                            store_path,
                            &mut app.clone(),
                            &base,
                        )
                        .await
                    },
                }
            })
            .collect::<FuturesUnordered<_>>();

        while let Some(res) = futures.next().await {
            res?;
        }

        Ok(())
    }
}

impl NixStore for MockNixStore {
    async fn add_file(
        &self,
        store_name: &str,
        file_contents: &[u8],
    ) -> NixStorePath {
        let store_name =
            store_name.parse::<NixStoreName>().expect("invalid store name");

        let store_hash = {
            let sha = Sha256::digest(file_contents);
            NixStoreHash::new(
                sha.as_slice()[..20].try_into().expect("correct length"),
            )
        };

        let store_basename =
            NixStoreBaseName { hash: store_hash, name: store_name };

        #[expect(clippy::unwrap_used)]
        self.files
            .lock()
            .unwrap()
            .insert(store_basename.clone(), file_contents.to_vec().into());

        NixStorePath::new(store_basename)
    }

    async fn delete(&self, store_path: &NixStorePath) {
        #[expect(clippy::unwrap_used)]
        self.files.lock().unwrap().remove(store_path.basename());
    }

    async fn nar_info_and_bytes(
        &self,
        store_path: &NixStorePath,
    ) -> (NarInfo<String>, Bytes) {
        let nar_bytes = self.get_file(store_path).unwrap_or_else(|| {
            panic!("store path {store_path} not found in mock store")
        });

        let narinfo = mock_narinfo(store_path.clone(), &nar_bytes)
            .with_url("".to_owned());

        (narinfo, nar_bytes)
    }

    async fn contains(&self, store_path: &NixStorePath) -> bool {
        self.get_file(store_path).is_some()
    }
}

/// Replicates the HTTP calls that `nix copy --to` makes when uploading a single
/// store path to an HTTP binary cache.
async fn copy_store_path_to(
    store: &MockNixStore,
    store_path: NixStorePath,
    app: &mut axum::Router,
    base: &str,
) -> Result<(), CopyError> {
    let Some(nar_bytes) = store.get_file(&store_path) else {
        return Err(CopyError::MissingStorePath(store_path));
    };

    // Step 1: GET /nix-cache-info
    //
    // The real Nix client does this to verify the StoreDir matches.
    let response = send(
        app,
        Request::builder()
            .method(http::Method::GET)
            .uri(format!("{base}/nix-cache-info"))
            .body(Body::empty())
            .expect("valid request"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK, "GET nix-cache-info failed");

    // Step 2: HEAD /{store_hash}.narinfo
    //
    // The real Nix client checks if the store path is already cached.
    let response = send(
        app,
        Request::builder()
            .method(http::Method::HEAD)
            .uri(format!("{base}/{}.narinfo", store_path.hash()))
            .body(Body::empty())
            .expect("valid request"),
    )
    .await;

    // If the narinfo already exists, the path is already cached.
    if response.status() == StatusCode::OK {
        return Ok(());
    }

    let narinfo = mock_narinfo(store_path, &nar_bytes);

    let nar_file_name = NarFileName::from(&narinfo);

    // Step 3: PUT /nar/{file_hash}.nar
    put_nar(app, base, &nar_file_name, nar_bytes).await?;

    // Set the NARInfo's URL to the correct format.
    let narinfo = narinfo.with_url(format!("nar/{nar_file_name}"));

    // Step 4: PUT /{store_hash}.narinfo
    put_narinfo(app, base, &narinfo).await.map_err(CopyError::PutNarInfoFailed)
}

/// Replicates the HTTP calls that `nix copy --from` makes when downloading a
/// single store path from an HTTP binary cache.
async fn copy_store_path_from(
    store: &MockNixStore,
    store_path: NixStorePath,
    app: &mut axum::Router,
    base: &str,
) -> Result<(), CopyError> {
    // Step 1: GET /{store_hash}.narinfo
    let response = send(
        app,
        Request::builder()
            .uri(format!("{base}/{}.narinfo", store_path.hash()))
            .body(Body::empty())
            .expect("valid request"),
    )
    .await;

    if !response.status().is_success() {
        return Err(CopyError::GetNarInfoFailed(response.status()));
    }

    let narinfo_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read narinfo body");

    let narinfo = core::str::from_utf8(&narinfo_bytes)
        .expect("narinfo is valid UTF-8")
        .parse::<NarInfo<String>>()
        .expect("narinfo is valid");

    // Step 2: GET the NAR URL from the narinfo.
    let response = send(
        app,
        Request::builder()
            .uri(&narinfo.url)
            .body(Body::empty())
            .expect("valid request"),
    )
    .await;

    if !response.status().is_success() {
        return Err(CopyError::GetNarFailed(response.status()));
    }

    let nar_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read NAR body");

    // Step 3: Store the NAR bytes in the mock store.
    #[expect(clippy::unwrap_used)]
    store
        .files
        .lock()
        .unwrap()
        .insert(store_path.basename().clone(), nar_bytes);

    Ok(())
}

/// Builds a [`NarInfo`] from a store path and its NAR bytes.
fn mock_narinfo(store_path: NixStorePath, nar_bytes: &[u8]) -> NarInfo<()> {
    let nar_hash_bytes = Sha256::digest(nar_bytes);

    let nar_hash = Nix32Digest::<32>::new(
        nar_hash_bytes
            .as_slice()
            .try_into()
            .expect("SHA-256 produces 32 bytes"),
    );

    let nar_size =
        NonZeroU64::new(nar_bytes.len() as u64).expect("NAR is never empty");

    NarInfo {
        store_path,
        url: (),
        compression: CompressionAlgorithm::None,
        file_hash: nar_hash,
        file_size: nar_size,
        nar_hash,
        nar_size,
        references: Default::default(),
        deriver: None,
        signatures: Default::default(),
        content_address: None,
    }
}

/// Uploads the NAR bytes to the cache server, following redirects to the object
/// storage.
async fn put_nar(
    app: &mut axum::Router,
    base: &str,
    nar_file_name: &NarFileName,
    nar_bytes: Bytes,
) -> Result<(), CopyError> {
    let make_request = |uri: &str| {
        Request::builder()
            .method(http::Method::PUT)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-nix-nar")
            .body(Body::from(nar_bytes.clone()))
            .expect("valid request")
    };

    let response =
        send(app, make_request(&format!("{base}/nar/{nar_file_name}"))).await;

    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),

        StatusCode::TEMPORARY_REDIRECT => {
            // Follow the 307 redirect.
            let location = response
                .headers()
                .get(header::LOCATION)
                .expect("307 must have Location")
                .to_str()
                .expect("Location is valid UTF-8");

            let response = send(app, make_request(location)).await;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(CopyError::PutNarRedirectFailed(response.status()))
            }
        },

        other => Err(CopyError::PutNarFailed(other)),
    }
}

/// Uploads the NARInfo to the cache server.
async fn put_narinfo(
    app: &mut axum::Router,
    base: &str,
    narinfo: &NarInfo<String>,
) -> Result<(), http::StatusCode> {
    let response = send(
        app,
        Request::builder()
            .method(http::Method::PUT)
            .uri(format!("{base}/{}.narinfo", narinfo.store_path.hash()))
            .header(header::CONTENT_TYPE, "text/x-nix-narinfo")
            .body(Body::from(narinfo.to_string()))
            .expect("valid request"),
    )
    .await;

    if response.status().is_success() { Ok(()) } else { Err(response.status()) }
}

/// Sends a request to the axum router and returns the response.
async fn send(
    app: &mut axum::Router,
    request: Request<Body>,
) -> axum::response::Response {
    match app.call(request).await {
        Ok(response) => response,
    }
}
