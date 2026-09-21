pub mod anisette;
mod client;
use std::fmt::Display;

pub use client::{AppleAccount, LoginState, TrustedPhoneNumber, AuthenticationExtras, VerifyBody};
pub use omnisette::AnisetteConfiguration;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse the response")]
    Parse,
    #[error("Failed to authenticate.")]
    AuthSrp,
    #[error("Bad 2fa code.")]
    Bad2faCode,
    #[error("{1} ({0})")]
    AuthSrpWithMessage(i64, String),
    #[error("Please login to appleid.apple.com to fix this account")]
    ExtraStep(String),
    #[error("Failed to parse a plist {0}")]
    PlistError(#[from] plist::Error),
    // [Shard patch] 애플이 plist 대신 HTML/에러페이지를 돌려줄 때, 그 본문 앞부분을 담아 폰 로그로
    // 실제 원인(본인확인·업데이트·지역차단·계정상태 등)을 보이게 한다. 예전엔 opaque PlistError로 숨었다.
    #[error("Apple returned non-plist (likely HTML): {0}")]
    ServerNonPlist(String),
    #[error("Request failed {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed getting anisette data {0}")]
    ErrorGettingAnisette(#[from] omnisette::AnisetteError)
}