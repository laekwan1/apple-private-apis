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
            // [Shard 2026-10] client-info의 **기기·OS는 anisette 서버가 준 값을 그대로 쓴다** — 그것이
            // 서버가 프로비저닝한 ADI 머신과 일치해야 애플이 받아준다(불일치면 429/-80045; isideload PR #26
            // "use client info from anisette server"와 동일). 단 **앱 구간만 akd로** 강제한다: 애플 엣지가
            // Xcode 클라이언트 신원(com.apple.dt.Xcode)을 503으로 막기 때문(AltStore PR #1796). 예전엔 기기·OS까지
            // 통째로 하드코딩(<MacBookPro13,2> macOS;13.1)했는데, 실제 Mac-ADI 서버(mac-ani: <MacBookPro13,3>
            // Mac OS X;10.12.6)로 바꾸자 그 하드코딩이 ADI와 어긋나 로그인이 막혔다. 서버 값이 없으면 종전 기본으로.
            let client_info = match &old_client_info {
                Some(server) => {
                    // server 예: "<MacBookPro13,3> <Mac OS X;10.12.6;16G29> <...앱...>" — 앞 두 <...>(기기·OS)만
                    // 취하고 앱 구간을 akd로 교체.
                    let mut segs = server.split('>');
                    let model = segs.next().unwrap_or("<MacBookPro13,2").trim();
                    let os = segs.next().unwrap_or("<macOS;13.1;22C65").trim();
                    format!("{model}> {os}> <com.apple.AuthKit/1 (com.apple.akd/1.0)>")
                }
                None => {
                    "<MacBookPro13,2> <macOS;13.1;22C65> <com.apple.AuthKit/1 (com.apple.akd/1.0)>".to_owned()
                }
            };
            headers.insert("X-Mme-Client-Info".to_owned(), client_info);
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
