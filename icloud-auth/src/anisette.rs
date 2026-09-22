use crate::Error;
use omnisette::{AnisetteConfiguration, AnisetteHeaders};
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, Clone)]
pub struct AnisetteData {
    pub base_headers: HashMap<String, String>,
    pub generated_at: SystemTime,
    pub config: AnisetteConfiguration,
}

impl AnisetteData {
    /// Fetches the data at an anisette server
    pub async fn new(config: AnisetteConfiguration) -> Result<Self, crate::Error> {
        let mut b = AnisetteHeaders::get_anisette_headers_provider(config.clone())?;
        let base_headers = b.provider.get_authentication_headers().await?;

        Ok(AnisetteData { base_headers, generated_at: SystemTime::now(), config })
    }

    pub fn needs_refresh(&self) -> bool {
        let elapsed = self.generated_at.elapsed().unwrap();
        elapsed.as_secs() > 60
    }

    pub fn is_valid(&self) -> bool {
        let elapsed = self.generated_at.elapsed().unwrap();
        elapsed.as_secs() < 90
    }

    pub async fn refresh(&self) -> Result<Self, crate::Error> {
        Self::new(self.config.clone()).await
    }

    pub fn generate_headers(
        &self,
        cpd: bool,
        client_info: bool,
        app_info: bool,
    ) -> HashMap<String, String> {
        if !self.is_valid() {
            panic!("Invalid data!")
        }
        let mut headers = self.base_headers.clone();
        let old_client_info = headers.remove("X-Mme-Client-Info");
        if client_info {
            // [Shard/AltStore PR #1796] 애플 엣지가 "Xcode" 클라이언트 신원(com.apple.dt.Xcode)을 503으로
            // 차단하기 시작했다. anisette 서버가 주는 Xcode client-info를 그대로 복사해 쓰면 503이 나므로,
            // akd 신원 + 현재 OS(macOS 27)로 하드코딩해 덮어쓴다(같은 요청이 akd 신원이면 서비스에 도달).
            // AltServer의 AnisetteDataManager와 동일한 접근. 서버가 준 old_client_info(모델·OS·Xcode)는 무시.
            let _ = old_client_info;
            headers.insert(
                "X-Mme-Client-Info".to_owned(),
                "<Mac15,7> <macOS;27.0;26A5378j> <com.apple.AuthKit/1 (com.apple.akd/1.0)>".to_owned(),
            );
        }

        if app_info {
            headers.insert(
                "X-Apple-App-Info".to_owned(),
                "com.apple.gs.xcode.auth".to_owned(),
            );
            headers.insert("X-Xcode-Version".to_owned(), "11.2 (11B41)".to_owned());
        }

        if cpd {
            headers.insert("bootstrap".to_owned(), "true".to_owned());
            headers.insert("icscrec".to_owned(), "true".to_owned());
            headers.insert("loc".to_owned(), "en_GB".to_owned());
            headers.insert("pbe".to_owned(), "false".to_owned());
            headers.insert("prkgen".to_owned(), "true".to_owned());
            headers.insert("svct".to_owned(), "iCloud".to_owned());
        }

        headers
    }

    pub fn to_plist(&self, cpd: bool, client_info: bool, app_info: bool) -> plist::Dictionary {
        let mut plist = plist::Dictionary::new();
        for (key, value) in self.generate_headers(cpd, client_info, app_info).iter() {
            plist.insert(key.to_owned(), plist::Value::String(value.to_owned()));
        }

        plist
    }

    pub fn get_header(&self, header: &str) -> Result<String, Error> {
        let headers = self
            .generate_headers(true, true, true)
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.to_lowercase()))
            .collect::<HashMap<String, String>>();

        match headers.get(&header.to_lowercase()) {
            Some(v) => Ok(v.to_string()),
            None => Err(Error::Parse),
        }
    }
}
