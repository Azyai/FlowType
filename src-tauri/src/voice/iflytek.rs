use crate::{
    asr::{credentials_for, IflytekCredentials},
    error::{AppError, AppResult},
    settings::{AppSettings, IflytekLanguage},
    voice::audio::{resample_to_16khz, RecordedAudio},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::{
    net::TcpStream,
    time::{Duration, SystemTime},
};
use tungstenite::{connect, stream::MaybeTlsStream, Message};

const IFLYTEK_HOST: &str = "iat-api.xfyun.cn";
const IFLYTEK_PATH: &str = "/v2/iat";
const IFLYTEK_ENDPOINT: &str = "wss://iat-api.xfyun.cn/v2/iat";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionResult {
    pub text: String,
}

pub fn recognize_blocking(
    settings: &AppSettings,
    audio: RecordedAudio,
    mut on_partial: impl FnMut(String),
) -> AppResult<RecognitionResult> {
    retry_asr_service(settings.iflytek_retry_count, || recognize_once(settings, &audio, &mut on_partial))
}

fn recognize_once(
    settings: &AppSettings,
    audio: &RecordedAudio,
    mut on_partial: impl FnMut(String),
) -> AppResult<RecognitionResult> {
    let credentials = credentials_for(settings)
        .ok_or_else(|| AppError::AsrConfigMissing("iFlytek credentials are incomplete".to_string()))?;
    let auth_url = build_auth_url(&credentials, SystemTime::now())?;
    let (mut socket, _) = connect(auth_url.as_str())
        .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    configure_socket_timeout(socket.get_mut(), request_timeout(settings))?;

    let audio = resample_to_16khz(audio);
    let frames = audio_frames(&audio.pcm, 1280);
    let mut final_text = String::new();

    for (index, frame) in frames.iter().enumerate() {
        let status = if frames.len() == 1 {
            2
        } else if index == 0 {
            0
        } else if index + 1 == frames.len() {
            2
        } else {
            1
        };
        let payload = request_payload(settings, &credentials.app_id, status, frame);
        socket
            .send(Message::Text(payload.into()))
            .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;

        let message = socket
            .read()
            .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
        if let Message::Text(text) = message {
            let response = parse_response(&text)?;
            if !response.text.is_empty() {
                final_text.push_str(&response.text);
                on_partial(final_text.clone());
            }
            if response.done {
                break;
            }
        }
    }

    let _ = socket.close(None);
    Ok(RecognitionResult { text: final_text })
}

fn retry_asr_service<T, F>(retry_count: u8, mut operation: F) -> AppResult<T>
where
    F: FnMut() -> AppResult<T>,
{
    let max_attempts = usize::from(retry_count).saturating_add(1);
    let mut last_error = None;

    for attempt_index in 0..max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(AppError::AsrServiceUnavailable(message)) => {
                let error = AppError::AsrServiceUnavailable(message);
                let attempt_number = attempt_index + 1;
                if attempt_number >= max_attempts {
                    return Err(error);
                }

                log::warn!(
                    "iFlytek recognition attempt {attempt_number}/{max_attempts} failed, retrying: {error}"
                );
                last_error = Some(error);
            }
            Err(error) => return Err(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::AsrServiceUnavailable("iFlytek recognition failed without a retryable error".to_string())
    }))
}

fn request_timeout(settings: &AppSettings) -> Duration {
    Duration::from_millis(settings.iflytek_timeout_ms.max(1_000))
}

fn configure_socket_timeout(stream: &mut MaybeTlsStream<TcpStream>, timeout: Duration) -> AppResult<()> {
    match stream {
        MaybeTlsStream::Plain(socket) => configure_tcp_stream_timeout(socket, timeout),
        MaybeTlsStream::NativeTls(socket) => configure_tcp_stream_timeout(socket.get_mut(), timeout),
        _ => Ok(()),
    }
}

fn configure_tcp_stream_timeout(stream: &mut TcpStream, timeout: Duration) -> AppResult<()> {
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    Ok(())
}

pub fn build_auth_url(credentials: &IflytekCredentials, time: SystemTime) -> AppResult<String> {
    let date = httpdate::fmt_http_date(time);
    let signature_origin = format!("host: {IFLYTEK_HOST}\ndate: {date}\nGET {IFLYTEK_PATH} HTTP/1.1");
    let mut mac = HmacSha256::new_from_slice(credentials.api_secret.as_bytes())
        .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    mac.update(signature_origin.as_bytes());
    let signature = STANDARD.encode(mac.finalize().into_bytes());
    let authorization_origin = format!(
        "api_key=\"{}\", algorithm=\"hmac-sha256\", headers=\"host date request-line\", signature=\"{}\"",
        credentials.api_key, signature
    );
    let authorization = STANDARD.encode(authorization_origin.as_bytes());
    Ok(format!(
        "{IFLYTEK_ENDPOINT}?authorization={}&date={}&host={}",
        urlencoding::encode(&authorization),
        urlencoding::encode(&date),
        IFLYTEK_HOST
    ))
}

fn request_payload(settings: &AppSettings, app_id: &str, status: i32, pcm: &[i16]) -> String {
    let audio_bytes: Vec<u8> = pcm.iter().flat_map(|sample| sample.to_le_bytes()).collect();
    let language = match settings.iflytek_language {
        IflytekLanguage::ZhCn | IflytekLanguage::ZhEn => "zh_cn",
        IflytekLanguage::EnUs => "en_us",
    };
    let accent = if settings.iflytek_mixed_language || matches!(settings.iflytek_language, IflytekLanguage::ZhEn) {
        "mandarin"
    } else {
        "mandarin"
    };

    json!({
        "common": { "app_id": app_id },
        "business": {
            "language": language,
            "domain": "iat",
            "accent": accent,
            "dwa": if settings.iflytek_mixed_language { "wpgs" } else { "" }
        },
        "data": {
            "status": status,
            "format": "audio/L16;rate=16000",
            "encoding": "raw",
            "audio": STANDARD.encode(audio_bytes)
        }
    })
    .to_string()
}

fn audio_frames(pcm: &[i16], frame_samples: usize) -> Vec<Vec<i16>> {
    if pcm.is_empty() {
        return vec![Vec::new()];
    }

    pcm.chunks(frame_samples).map(|chunk| chunk.to_vec()).collect()
}

#[derive(Debug, Deserialize)]
struct IflytekResponse {
    code: i32,
    message: Option<String>,
    data: Option<IflytekData>,
}

#[derive(Debug, Deserialize)]
struct IflytekData {
    status: i32,
    result: Option<IflytekResult>,
}

#[derive(Debug, Deserialize)]
struct IflytekResult {
    ws: Vec<IflytekWordSegment>,
}

#[derive(Debug, Deserialize)]
struct IflytekWordSegment {
    cw: Vec<IflytekCandidate>,
}

#[derive(Debug, Deserialize)]
struct IflytekCandidate {
    w: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedResponse {
    text: String,
    done: bool,
}

fn parse_response(raw: &str) -> AppResult<ParsedResponse> {
    let response: IflytekResponse =
        serde_json::from_str(raw).map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    if response.code != 0 {
        return Err(AppError::AsrServiceUnavailable(
            response.message.unwrap_or_else(|| format!("iFlytek returned code {}", response.code)),
        ));
    }

    let Some(data) = response.data else {
        return Ok(ParsedResponse {
            text: String::new(),
            done: false,
        });
    };
    let text = data
        .result
        .map(|result| {
            result
                .ws
                .into_iter()
                .filter_map(|segment| segment.cw.into_iter().next())
                .map(|candidate| candidate.w)
                .collect::<String>()
        })
        .unwrap_or_default();

    Ok(ParsedResponse {
        text,
        done: data.status == 2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    #[test]
    fn auth_url_contains_required_query_parts_without_plain_secret() {
        let credentials = IflytekCredentials {
            app_id: "appid".to_string(),
            api_key: "apikey".to_string(),
            api_secret: "secret".to_string(),
        };

        let url = build_auth_url(&credentials, SystemTime::UNIX_EPOCH).unwrap();

        assert!(url.starts_with(IFLYTEK_ENDPOINT));
        assert!(url.contains("authorization="));
        assert!(url.contains("date="));
        assert!(url.contains("host=iat-api.xfyun.cn"));
        assert!(!url.contains("secret"));
    }

    #[test]
    fn parses_final_text_from_iflytek_response() {
        let raw = r#"{
            "code": 0,
            "message": "success",
            "data": {
              "status": 2,
              "result": {
                "ws": [
                  {"cw": [{"w": "你好"}]},
                  {"cw": [{"w": "世界"}]}
                ]
              }
            }
        }"#;

        let parsed = parse_response(raw).unwrap();

        assert_eq!(parsed.text, "你好世界");
        assert!(parsed.done);
    }

    #[test]
    fn propagates_iflytek_error_code() {
        let raw = r#"{"code": 10105, "message": "auth failed"}"#;

        let error = parse_response(raw).unwrap_err();

        assert!(error.to_string().contains("auth failed"));
    }

    #[test]
    fn retries_service_unavailable_errors_until_success() {
        let mut attempts = 0;

        let result = retry_asr_service(2, || {
            attempts += 1;
            if attempts < 3 {
                Err(AppError::AsrServiceUnavailable("temporary".to_string()))
            } else {
                Ok("done")
            }
        })
        .unwrap();

        assert_eq!(result, "done");
        assert_eq!(attempts, 3);
    }

    #[test]
    fn does_not_retry_non_service_errors() {
        let mut attempts = 0;

        let error = retry_asr_service(3, || {
            attempts += 1;
            Err::<(), AppError>(AppError::AsrConfigMissing("missing".to_string()))
        })
        .unwrap_err();

        assert!(matches!(error, AppError::AsrConfigMissing(_)));
        assert_eq!(attempts, 1);
    }

    #[test]
    fn returns_last_retryable_error_after_retries_are_exhausted() {
        let mut attempts = 0;

        let error = retry_asr_service(1, || {
            attempts += 1;
            Err::<(), AppError>(AppError::AsrServiceUnavailable(format!("temporary-{attempts}")))
        })
        .unwrap_err();

        assert!(matches!(error, AppError::AsrServiceUnavailable(_)));
        assert_eq!(attempts, 2);
        assert!(error.to_string().contains("temporary-2"));
    }
}
