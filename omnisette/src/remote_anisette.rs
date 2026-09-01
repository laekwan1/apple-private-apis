use crate::{anisette_headers_provider::AnisetteHeadersProvider, AnisetteError};
#[cfg(not(feature = "async"))]
use reqwest::blocking::get;
#[cfg(feature = "async")]
use reqwest::get;
use std::collections::HashMap;

pub struct RemoteAnisetteProvider {
    url: String,
}

impl RemoteAnisetteProvider {
    pub fn new(url: String) -> RemoteAnisetteProvider {
        RemoteAnisetteProvider { url }
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl AnisetteHeadersProvider for RemoteAnisetteProvider {
    #[cfg_attr(not(feature = "async"), remove_async_await::remove_async_await)]
    async fn get_anisette_headers(
        &mut self,
        _skip_provisioning: bool,
    ) -> Result<HashMap<String, String>, AnisetteError> {
        let mut headers: HashMap<String, String> = get(&self.url).await?.json().await?;
        // [Shard patch] v1 anisette 서버가 X-MMe-Client-Info(대문자 MMe)를 주면 icloud_auth가
        // X-Mme-Client-Info로 못 찾아(대소문자 구분) Xcode 앱 헤더가 빠진다. 정규화한다.
        if let Some(k) = headers.keys().find(|k| k.eq_ignore_ascii_case("X-Mme-Client-Info")).cloned() {
            if k != "X-Mme-Client-Info" {
                if let Some(v) = headers.remove(&k) { headers.insert("X-Mme-Client-Info".to_string(), v); }
            }
        }
        Ok(headers)
    }
}

#[cfg(all(test, not(feature = "async")))]
mod tests {
    use crate::anisette_headers_provider::AnisetteHeadersProvider;
    use crate::remote_anisette::RemoteAnisetteProvider;
    use crate::DEFAULT_ANISETTE_URL;
    use log::info;

    #[test]
    fn fetch_anisette_remote() -> Result<(), AnisetteError> {
        crate::tests::init_logger();

        let mut provider = RemoteAnisetteProvider::new(DEFAULT_ANISETTE_URL.to_string());
        info!(
            "Remote headers: {:?}",
            (&mut provider as &mut dyn AnisetteHeadersProvider).get_authentication_headers()?
        );
        Ok(())
    }
}
