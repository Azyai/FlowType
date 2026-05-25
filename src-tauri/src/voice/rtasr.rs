use crate::{
    asr::{credentials_for, RtasrCredentials},
    error::{AppError, AppResult},
    settings::{AppSettings, RtasrLanguage},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha1::Sha1;
use std::{
    collections::BTreeMap,
    net::TcpStream,
    net::ToSocketAddrs,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tungstenite::{
    client::IntoClientRequest,
    client_tls_with_config,
    handshake::HandshakeError,
    stream::MaybeTlsStream,
    Message,
    WebSocket,
};

const RTASR_ENDPOINT: &str = "wss://rtasr.xfyun.cn/v1/ws";
const FRAME_BYTES: usize = 1280;
const FRAME_INTERVAL: Duration = Duration::from_millis(40);
const READ_POLL_INTERVAL: Duration = Duration::from_millis(5);
const FINISH_MESSAGE: &[u8] = br#"{"end": true}"#;
const POST_END_IDLE_TIMEOUT: Duration = Duration::from_millis(800);
const POST_END_MAX_WAIT: Duration = Duration::from_secs(2);
const SESSION_START_MAX_ATTEMPTS: usize = 2;
const SESSION_START_RETRY_DELAY: Duration = Duration::from_millis(350);

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionResult {
    pub text: String,
}

enum RecognizerCommand {
    Audio { pcm: Vec<i16>, sample_rate: u32 },
    Finish,
    Cancel,
}

#[derive(Clone)]
pub struct StreamingRecognizerSink {
    command_tx: Sender<RecognizerCommand>,
}

impl StreamingRecognizerSink {
    pub fn push_audio(&self, pcm: Vec<i16>, sample_rate: u32) -> AppResult<()> {
        if pcm.is_empty() {
            return Ok(());
        }

        self.command_tx
            .send(RecognizerCommand::Audio { pcm, sample_rate })
            .map_err(|_| AppError::AsrServiceUnavailable("RTASR session is no longer running.".to_string()))
    }
}

pub struct StreamingRecognizer {
    command_tx: Sender<RecognizerCommand>,
    result_rx: Receiver<AppResult<RecognitionResult>>,
    join_handle: Option<JoinHandle<()>>,
}

impl StreamingRecognizer {
    pub fn sink(&self) -> StreamingRecognizerSink {
        StreamingRecognizerSink {
            command_tx: self.command_tx.clone(),
        }
    }

    pub fn finish(mut self) -> AppResult<RecognitionResult> {
        let _ = self.command_tx.send(RecognizerCommand::Finish);
        let result = self
            .result_rx
            .recv()
            .map_err(|_| AppError::AsrServiceUnavailable("RTASR session exited unexpectedly.".to_string()))?;
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
        result
    }

    pub fn cancel(mut self) {
        let _ = self.command_tx.send(RecognizerCommand::Cancel);
        let _ = self.result_rx.recv();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

pub fn start_streaming_session(
    settings: AppSettings,
    mut on_partial: impl FnMut(String) + Send + 'static,
) -> AppResult<StreamingRecognizer> {
    let mut socket = start_session_socket_with_retry(&settings)?;

    let (command_tx, command_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let join_handle = thread::spawn(move || {
        let result = run_session_loop(&mut socket, &settings, command_rx, &mut on_partial);
        let _ = socket.close(None);
        let _ = result_tx.send(result);
    });

    Ok(StreamingRecognizer {
        command_tx,
        result_rx,
        join_handle: Some(join_handle),
    })
}

fn start_session_socket_with_retry(
    settings: &AppSettings,
) -> AppResult<WebSocket<MaybeTlsStream<TcpStream>>> {
    let mut last_error = None;

    for attempt in 1..=SESSION_START_MAX_ATTEMPTS {
        match start_session_socket(settings) {
            Ok(socket) => return Ok(socket),
            Err(error) if attempt < SESSION_START_MAX_ATTEMPTS && is_retryable_session_start_error(&error) => {
                log::warn!(
                    "RTASR startup attempt {attempt}/{SESSION_START_MAX_ATTEMPTS} failed, retrying once: {error}"
                );
                last_error = Some(error);
                thread::sleep(SESSION_START_RETRY_DELAY);
            }
            Err(error) => return Err(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::AsrServiceUnavailable("RTASR session could not be started.".to_string())
    }))
}

fn start_session_socket(settings: &AppSettings) -> AppResult<WebSocket<MaybeTlsStream<TcpStream>>> {
    let credentials = credentials_for(settings)
        .ok_or_else(|| AppError::AsrConfigMissing("RTASR credentials are incomplete.".to_string()))?;
    let timeout = request_timeout(settings);
    let auth_url = build_auth_url(&credentials, settings, SystemTime::now())?;
    let mut socket = connect_websocket(&auth_url, timeout)?;
    await_started(&mut socket)?;
    configure_socket_streaming_timeout(socket.get_mut(), timeout)?;
    Ok(socket)
}

fn is_retryable_session_start_error(error: &AppError) -> bool {
    let AppError::AsrServiceUnavailable(message) = error else {
        return false;
    };

    message.contains("Timed out while connecting")
        || message.contains("refused the connection")
        || message.contains("closed the connection before the session started")
        || message.contains("Network error while contacting the RTASR gateway")
}

fn run_session_loop(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    _settings: &AppSettings,
    command_rx: Receiver<RecognizerCommand>,
    on_partial: &mut impl FnMut(String),
) -> AppResult<RecognitionResult> {
    let mut accumulator = ResultAccumulator::default();
    let mut audio_buffer = Vec::new();
    let mut finishing = false;
    let mut end_sent = false;
    let mut end_sent_at = None;
    let mut last_send_at = Instant::now()
        .checked_sub(FRAME_INTERVAL)
        .unwrap_or_else(Instant::now);
    let mut last_server_update_at = Instant::now();
    let mut received_any_result = false;

    loop {
        while let Ok(command) = command_rx.try_recv() {
            match command {
                RecognizerCommand::Audio { pcm, sample_rate } => {
                    audio_buffer.extend(resample_chunk_to_16khz_bytes(&pcm, sample_rate));
                }
                RecognizerCommand::Finish => finishing = true,
                RecognizerCommand::Cancel => {
                    return Err(AppError::Voice("RTASR session canceled.".to_string()));
                }
            }
        }

        if audio_buffer.len() >= FRAME_BYTES && last_send_at.elapsed() >= FRAME_INTERVAL {
            let frame = audio_buffer.drain(..FRAME_BYTES).collect::<Vec<_>>();
            socket
                .send(Message::Binary(frame.into()))
                .map_err(map_transport_error_to_asr)?;
            last_send_at = Instant::now();
        } else if finishing && !audio_buffer.is_empty() && last_send_at.elapsed() >= FRAME_INTERVAL {
            let frame = std::mem::take(&mut audio_buffer);
            socket
                .send(Message::Binary(frame.into()))
                .map_err(map_transport_error_to_asr)?;
            last_send_at = Instant::now();
        } else if finishing && !end_sent && audio_buffer.is_empty() && last_send_at.elapsed() >= FRAME_INTERVAL {
            socket
                .send(Message::Binary(FINISH_MESSAGE.to_vec().into()))
                .map_err(map_transport_error_to_asr)?;
            end_sent = true;
            end_sent_at = Some(Instant::now());
            last_send_at = Instant::now();
        }

        let server_state = drain_server_messages(socket, &mut accumulator, on_partial)?;
        if server_state.updated {
            last_server_update_at = Instant::now();
            received_any_result = true;
        }

        if finishing && end_sent {
            if server_state.closed {
                break;
            }
            if received_any_result && last_server_update_at.elapsed() >= POST_END_IDLE_TIMEOUT {
                break;
            }
            if end_sent_at.is_some_and(|sent_at| sent_at.elapsed() >= POST_END_MAX_WAIT) {
                break;
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    Ok(RecognitionResult {
        text: accumulator.combined_text(),
    })
}

fn drain_server_messages(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    accumulator: &mut ResultAccumulator,
    on_partial: &mut impl FnMut(String),
) -> AppResult<ServerDrainState> {
    let mut state = ServerDrainState::default();

    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                if let Some(text) = process_server_message(&text, accumulator)? {
                    if !text.is_empty() {
                        on_partial(text);
                    }
                    state.updated = true;
                }
            }
            Ok(Message::Close(_)) => {
                state.closed = true;
                return Ok(state);
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                return Ok(state);
            }
            Err(error) => return Err(map_transport_error_to_asr(error)),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ServerDrainState {
    updated: bool,
    closed: bool,
}

fn connect_websocket(
    auth_url: &str,
    timeout: Duration,
) -> AppResult<WebSocket<MaybeTlsStream<TcpStream>>> {
    let request = auth_url
        .into_client_request()
        .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    let host = request
        .uri()
        .host()
        .ok_or_else(|| AppError::AsrServiceUnavailable("RTASR websocket URL is missing a hostname".to_string()))?;
    let host = if host.starts_with('[') {
        &host[1..host.len() - 1]
    } else {
        host
    };
    let port = request.uri().port_u16().unwrap_or(443);
    let stream = connect_tcp_stream(host, port, timeout)?;
    let (mut socket, _) = client_tls_with_config(request, stream, None, None).map_err(|error| match error {
        HandshakeError::Failure(error) => {
            AppError::AsrServiceUnavailable(normalize_transport_error_message(&error.to_string()))
        }
        HandshakeError::Interrupted(_) => AppError::AsrServiceUnavailable(
            "The RTASR websocket handshake was interrupted before completion.".to_string(),
        ),
    })?;
    configure_socket_startup_timeout(socket.get_mut(), timeout)?;
    Ok(socket)
}

fn await_started(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> AppResult<()> {
    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                let frame: RtasrFrame =
                    serde_json::from_str(&text).map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
                ensure_success(&frame)?;
                if frame.action == "started" {
                    return Ok(());
                }
                if frame.action == "result" {
                    return Ok(());
                }
            }
            Ok(Message::Close(_)) => {
                return Err(AppError::AsrServiceUnavailable(
                    "RTASR closed the connection before the session started.".to_string(),
                ));
            }
            Ok(_) => {}
            Err(error) => return Err(map_transport_error_to_asr(error)),
        }
    }
}

fn request_timeout(settings: &AppSettings) -> Duration {
    Duration::from_millis(settings.rtasr_timeout_ms.max(5_000))
}

fn connect_tcp_stream(host: &str, port: u16, timeout: Duration) -> AppResult<TcpStream> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(map_io_error_to_asr)?;
    let mut last_error = None;

    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(stream) => {
                stream.set_nodelay(true).map_err(map_io_error_to_asr)?;
                return Ok(stream);
            }
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.map_or_else(
        || {
            AppError::AsrServiceUnavailable(format!(
                "Could not resolve or connect to the RTASR gateway at {host}:{port}."
            ))
        },
        map_io_error_to_asr,
    ))
}

fn configure_socket_startup_timeout(stream: &mut MaybeTlsStream<TcpStream>, timeout: Duration) -> AppResult<()> {
    configure_socket_timeout(stream, startup_read_timeout(timeout), timeout)
}

fn configure_socket_streaming_timeout(stream: &mut MaybeTlsStream<TcpStream>, timeout: Duration) -> AppResult<()> {
    configure_socket_timeout(stream, streaming_read_timeout(timeout), timeout)
}

fn configure_socket_timeout(
    stream: &mut MaybeTlsStream<TcpStream>,
    read_timeout: Duration,
    write_timeout: Duration,
) -> AppResult<()> {
    match stream {
        MaybeTlsStream::Plain(socket) => configure_tcp_stream_timeout(socket, read_timeout, write_timeout),
        MaybeTlsStream::NativeTls(socket) => {
            configure_tcp_stream_timeout(socket.get_mut(), read_timeout, write_timeout)
        }
        _ => Ok(()),
    }
}

fn configure_tcp_stream_timeout(
    stream: &mut TcpStream,
    read_timeout: Duration,
    write_timeout: Duration,
) -> AppResult<()> {
    stream
        .set_read_timeout(Some(read_timeout))
        .map_err(map_io_error_to_asr)?;
    stream
        .set_write_timeout(Some(write_timeout))
        .map_err(map_io_error_to_asr)?;
    Ok(())
}

fn startup_read_timeout(timeout: Duration) -> Duration {
    timeout.max(READ_POLL_INTERVAL)
}

fn streaming_read_timeout(timeout: Duration) -> Duration {
    READ_POLL_INTERVAL.min(timeout)
}

fn map_io_error_to_asr(error: std::io::Error) -> AppError {
    AppError::AsrServiceUnavailable(normalize_transport_error_message(&error.to_string()))
}

fn normalize_transport_error_message(message: &str) -> String {
    let lowered = message.to_lowercase();

    if lowered.contains("10060")
        || lowered.contains("timed out")
        || message.contains("连接尝试失败")
        || message.contains("连接的主机没有反应")
    {
        return "Timed out while connecting to the RTASR gateway. Please check network, proxy, or firewall access to rtasr.xfyun.cn:443.".to_string();
    }

    if lowered.contains("10061") || lowered.contains("actively refused") || message.contains("由于目标计算机积极拒绝") {
        return "The RTASR gateway refused the connection. Please check whether rtasr.xfyun.cn:443 is reachable from this network.".to_string();
    }

    if lowered.contains("dns") || lowered.contains("no such host") || message.contains("找不到主机") {
        return "Could not resolve the RTASR gateway hostname. Please check DNS or proxy settings.".to_string();
    }

    format!("Network error while contacting the RTASR gateway: {message}")
}

fn normalize_service_error_message(frame: &RtasrFrame) -> String {
    let code = frame.code.trim();
    let desc = frame.desc.trim();

    if code == "10110" || desc.contains("license") {
        return "RTASR service is not enabled for the current AppID/APIKey, or quota has been exhausted.".to_string();
    }

    if desc.contains("illegal signa") || desc.contains("authorization") {
        return "RTASR authentication failed. Please verify that AppID and APIKey match the realtime transcription service.".to_string();
    }

    if desc.contains("illegal client_ip") {
        return "RTASR rejected this client IP. Please check the iFlytek IP whitelist configuration.".to_string();
    }

    if desc.is_empty() {
        format!("RTASR returned code {}.", frame.code)
    } else {
        desc.to_string()
    }
}

pub fn build_auth_url(
    credentials: &RtasrCredentials,
    settings: &AppSettings,
    time: SystemTime,
) -> AppResult<String> {
    let ts = time
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    let base = format!("{}{}", credentials.app_id, ts);
    let digest = format!("{:x}", md5::compute(base));
    let mut mac = HmacSha1::new_from_slice(credentials.api_key.as_bytes())
        .map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    mac.update(digest.as_bytes());
    let signa = STANDARD.encode(mac.finalize().into_bytes());
    let mut query = vec![
        format!("appid={}", urlencoding::encode(&credentials.app_id)),
        format!("ts={}", urlencoding::encode(&ts)),
        format!("signa={}", urlencoding::encode(&signa)),
    ];
    match settings.rtasr_language {
        RtasrLanguage::ZhCn => {
            query.push("lang=cn".to_string());
            query.push("engLangType=4".to_string());
        }
        RtasrLanguage::ZhEn => {
            query.push("lang=cn".to_string());
            query.push("engLangType=1".to_string());
        }
        RtasrLanguage::EnUs => {
            query.push("lang=en".to_string());
        }
    }
    Ok(format!("{RTASR_ENDPOINT}?{}", query.join("&")))
}

fn ensure_success(frame: &RtasrFrame) -> AppResult<()> {
    if frame.action == "error" || frame.code != "0" {
        return Err(AppError::AsrServiceUnavailable(normalize_service_error_message(frame)));
    }
    Ok(())
}

fn process_server_message(raw: &str, accumulator: &mut ResultAccumulator) -> AppResult<Option<String>> {
    let frame: RtasrFrame =
        serde_json::from_str(raw).map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    ensure_success(&frame)?;
    if frame.action != "result" || frame.data.trim().is_empty() {
        return Ok(None);
    }

    log::debug!("rtasr raw result payload: {}", frame.data.trim());

    let payload: RtasrResultPayload =
        serde_json::from_str(frame.data.trim()).map_err(|error| AppError::AsrServiceUnavailable(error.to_string()))?;
    let previous = accumulator.combined_text();
    let text = accumulator.apply(payload);
    if text == previous {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

#[derive(Debug, Deserialize)]
struct RtasrFrame {
    action: String,
    code: String,
    #[serde(default)]
    data: String,
    #[serde(default)]
    desc: String,
    #[allow(dead_code)]
    sid: Option<String>,
}

#[derive(Debug, Default)]
struct ResultAccumulator {
    final_segments: BTreeMap<i64, String>,
    live_segment: Option<TrackedSegment>,
}

#[derive(Debug, Clone)]
struct TrackedSegment {
    seg_id: i64,
    bg: Option<i64>,
    ed: Option<i64>,
    text: String,
}

impl ResultAccumulator {
    fn apply(&mut self, payload: RtasrResultPayload) -> String {
        let Some(segment) = payload.tracked_segment() else {
            return self.combined_text();
        };

        if payload.is_final() {
            self.commit_final_segment(segment);
        } else {
            self.update_live_segment(segment);
        }

        self.combined_text()
    }

    fn commit_final_segment(&mut self, segment: TrackedSegment) {
        let accumulated = self.combined_text();
        if should_replace_combined_text(&accumulated, &segment.text) {
            log::debug!(
                "rtasr final snapshot replaced accumulated text: {accumulated} -> {}",
                segment.text
            );
            self.final_segments.clear();
            self.live_segment = None;
            self.final_segments.insert(segment.seg_id, segment.text);
            return;
        }

        if let Some(existing) = self.combined_text_if_finalized() {
            if should_replace_combined_text(&existing, &segment.text) {
                log::debug!("rtasr final snapshot replaced previous combined text: {existing} -> {}", segment.text);
                self.final_segments.clear();
            }
        }

        if self
            .live_segment
            .as_ref()
            .is_some_and(|live| segments_match(live, &segment) || should_replace_combined_text(&live.text, &segment.text))
        {
            self.live_segment = None;
        }

        self.commit_segment_delta(segment);
    }

    fn update_live_segment(&mut self, segment: TrackedSegment) {
        let accumulated = self.combined_text();
        if should_replace_combined_text(&accumulated, &segment.text) {
            log::debug!(
                "rtasr interim snapshot replaced accumulated text: {accumulated} -> {}",
                segment.text
            );
            self.final_segments.clear();
            self.live_segment = Some(segment);
            return;
        }

        if let Some(existing) = self.combined_text_if_finalized() {
            if should_replace_combined_text(&existing, &segment.text) {
                log::debug!(
                    "rtasr interim snapshot replaced previous combined text: {existing} -> {}",
                    segment.text
                );
                self.final_segments.clear();
                self.live_segment = Some(segment);
                return;
            }
        }

        match self.live_segment.take() {
            Some(current) if is_substantially_contained(&current.text, &segment.text) => {
                log::debug!(
                    "rtasr ignored redundant interim tail already covered by live segment: {}",
                    segment.text
                );
                self.live_segment = Some(current);
            }
            Some(current)
                if segments_match(&current, &segment)
                    || should_replace_combined_text(&current.text, &segment.text) =>
            {
                self.live_segment = Some(segment);
            }
            Some(current) => {
                self.commit_segment_delta(current);
                self.live_segment = Some(segment);
            }
            None => {
                self.live_segment = Some(segment);
            }
        }
    }

    fn commit_segment_delta(&mut self, segment: TrackedSegment) {
        let committed = self.combined_text_if_finalized().unwrap_or_default();
        let delta = unseen_suffix_after_merge(&committed, &segment.text);
        if delta.is_empty() {
            return;
        }

        self.final_segments.insert(segment.seg_id, delta);
    }

    fn combined_text_if_finalized(&self) -> Option<String> {
        if self.final_segments.is_empty() {
            None
        } else {
            Some(
                self.final_segments
                    .values()
                    .fold(String::new(), |combined, segment| merge_with_overlap(&combined, segment)),
            )
        }
    }

    fn combined_text(&self) -> String {
        let finalized = self.combined_text_if_finalized().unwrap_or_default();
        let Some(live) = self.live_segment.as_ref() else {
            return finalized;
        };

        if should_replace_combined_text(&finalized, &live.text) {
            live.text.clone()
        } else {
            merge_with_overlap(&finalized, &live.text)
        }
    }
}

fn segments_match(existing: &TrackedSegment, next: &TrackedSegment) -> bool {
    existing.seg_id == next.seg_id
        || existing.bg.zip(next.bg).is_some_and(|(left, right)| left == right)
        || existing.ed.zip(next.ed).is_some_and(|(left, right)| left == right)
}

fn merge_with_overlap(existing: &str, next: &str) -> String {
    if existing.is_empty() {
        return next.to_string();
    }
    if next.is_empty() {
        return existing.to_string();
    }

    let existing_chars: Vec<char> = existing.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();
    let max_overlap = existing_chars.len().min(next_chars.len());

    let overlap = (1..=max_overlap)
        .rev()
        .find(|&len| existing_chars[existing_chars.len() - len..] == next_chars[..len])
        .unwrap_or(0);

    let mut merged = String::with_capacity(existing.len() + next.len().saturating_sub(overlap));
    merged.push_str(existing);
    merged.extend(next_chars[overlap..].iter());
    merged
}

fn unseen_suffix_after_merge(existing: &str, next: &str) -> String {
    if next.is_empty() {
        return String::new();
    }
    if existing.is_empty() {
        return next.to_string();
    }
    if should_replace_combined_text(existing, next) || is_substantially_contained(existing, next) {
        return String::new();
    }

    let merged = merge_with_overlap(existing, next);
    let existing_len = existing.chars().count();
    merged.chars().skip(existing_len).collect()
}

fn should_replace_combined_text(existing: &str, next: &str) -> bool {
    if existing.is_empty() || next.is_empty() {
        return false;
    }

    let existing_chars: Vec<char> = existing.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();

    if next_chars.len() < existing_chars.len() {
        return false;
    }

    if next.starts_with(existing) {
        return true;
    }

    let shared_prefix_len = existing_chars
        .iter()
        .zip(next_chars.iter())
        .take_while(|(left, right)| left == right)
        .count();
    let shorter_len = existing_chars.len().min(next_chars.len());

    shared_prefix_len * 100 >= shorter_len * 75
}

fn is_substantially_contained(existing: &str, next: &str) -> bool {
    if existing.is_empty() || next.is_empty() {
        return false;
    }

    let normalized_existing = normalize_for_match(existing);
    let normalized_next = normalize_for_match(next);

    if normalized_next.chars().count() < 8 {
        return false;
    }

    normalized_existing.chars().count() >= normalized_next.chars().count()
        && normalized_existing.contains(&normalized_next)
}

fn normalize_for_match(text: &str) -> String {
    text.chars().filter_map(normalize_match_char).collect()
}

fn normalize_match_char(ch: char) -> Option<char> {
    if ch.is_whitespace()
        || ch.is_ascii_punctuation()
        || matches!(
            ch,
            '，' | '。' | '！' | '？' | '、' | '；' | '：' | '“' | '”' | '‘' | '’' | '（' | '）'
                | '《' | '》' | '【' | '】' | '…' | '—' | '－' | '～'
        )
    {
        return None;
    }

    if matches!(
        ch,
        '0'
            | '1'
            | '2'
            | '3'
            | '4'
            | '5'
            | '6'
            | '7'
            | '8'
            | '9'
            | '零'
            | '〇'
            | '一'
            | '二'
            | '两'
            | '三'
            | '四'
            | '五'
            | '六'
            | '七'
            | '八'
            | '九'
            | '十'
    ) {
        return Some('#');
    }

    if ch.is_ascii_alphabetic() {
        return Some(ch.to_ascii_lowercase());
    }

    Some(ch)
}

#[derive(Debug, Deserialize)]
struct RtasrResultPayload {
    #[serde(default)]
    seg_id: i64,
    cn: Option<RtasrCnPayload>,
}

impl RtasrResultPayload {
    fn is_final(&self) -> bool {
        self.cn
            .as_ref()
            .and_then(|cn| cn.st.as_ref())
            .is_some_and(RtasrSentence::is_final)
    }

    fn tracked_segment(&self) -> Option<TrackedSegment> {
        let sentence = self.cn.as_ref()?.st.as_ref()?;
        let text = sentence.text();
        if text.is_empty() {
            return None;
        }

        Some(TrackedSegment {
            seg_id: self.seg_id,
            bg: sentence.bg_ms(),
            ed: sentence.ed_ms(),
            text,
        })
    }
}

#[derive(Debug, Deserialize)]
struct RtasrCnPayload {
    st: Option<RtasrSentence>,
}

#[derive(Debug, Deserialize)]
struct RtasrSentence {
    #[serde(default)]
    bg: Option<String>,
    #[serde(default)]
    ed: Option<String>,
    #[serde(rename = "type", default)]
    result_type: Option<String>,
    #[serde(default)]
    rt: Vec<RtasrResultTrack>,
}

impl RtasrSentence {
    fn text(&self) -> String {
        self.rt
            .iter()
            .flat_map(|track| track.ws.iter())
            .filter_map(|word| word.cw.first())
            .map(|candidate| candidate.w.as_str())
            .collect::<String>()
    }

    fn is_final(&self) -> bool {
        self.result_type.as_deref() == Some("0")
    }

    fn bg_ms(&self) -> Option<i64> {
        self.bg.as_deref()?.parse().ok()
    }

    fn ed_ms(&self) -> Option<i64> {
        self.ed.as_deref()?.parse().ok()
    }
}

#[derive(Debug, Deserialize)]
struct RtasrResultTrack {
    #[serde(default)]
    ws: Vec<RtasrWord>,
}

#[derive(Debug, Deserialize)]
struct RtasrWord {
    #[serde(default)]
    cw: Vec<RtasrCandidate>,
}

#[derive(Debug, Deserialize)]
struct RtasrCandidate {
    w: String,
}

fn resample_chunk_to_16khz_bytes(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let pcm = if sample_rate == 16_000 || samples.is_empty() {
        samples.to_vec()
    } else {
        let source_len = samples.len();
        let target_len = ((source_len as u64) * 16_000).div_ceil(sample_rate as u64) as usize;
        let mut resampled = Vec::with_capacity(target_len);
        let last_index = source_len.saturating_sub(1);

        for index in 0..target_len {
            let source_position = index as f64 * sample_rate as f64 / 16_000.0;
            let left_index = source_position.floor() as usize;
            let left_index = left_index.min(last_index);
            let right_index = (left_index + 1).min(last_index);
            let fraction = source_position - left_index as f64;
            let left = samples[left_index] as f64;
            let right = samples[right_index] as f64;
            let interpolated = left + (right - left) * fraction;
            resampled.push(interpolated.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16);
        }

        resampled
    };

    pcm.into_iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect()
}

fn map_transport_error_to_asr(error: tungstenite::Error) -> AppError {
    AppError::AsrServiceUnavailable(normalize_transport_error_message(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_url_contains_required_query_parts_without_plain_key() {
        let credentials = RtasrCredentials {
            app_id: "appid".to_string(),
            api_key: "apikey".to_string(),
        };
        let settings = AppSettings::default();

        let url = build_auth_url(&credentials, &settings, SystemTime::UNIX_EPOCH).unwrap();

        assert!(url.starts_with(RTASR_ENDPOINT));
        assert!(url.contains("appid=appid"));
        assert!(url.contains("ts=0"));
        assert!(url.contains("signa="));
        assert!(url.contains("lang=cn"));
        assert!(!url.contains("apikey"));
    }

    #[test]
    fn english_mode_uses_en_lang_without_chinese_flags() {
        let credentials = RtasrCredentials {
            app_id: "appid".to_string(),
            api_key: "apikey".to_string(),
        };
        let mut settings = AppSettings::default();
        settings.rtasr_language = RtasrLanguage::EnUs;

        let url = build_auth_url(&credentials, &settings, SystemTime::UNIX_EPOCH).unwrap();

        assert!(url.contains("lang=en"));
        assert!(!url.contains("engLangType"));
    }

    #[test]
    fn parses_rtasr_result_payload_text() {
        let raw = r#"{
            "action":"result",
            "code":"0",
            "data":"{\"cn\":{\"st\":{\"rt\":[{\"ws\":[{\"cw\":[{\"w\":\"你好\"}]},{\"cw\":[{\"w\":\"世界\"}]}]}]}},\"seg_id\":3}",
            "desc":"success"
        }"#;
        let mut accumulator = ResultAccumulator::default();

        let text = process_server_message(raw, &mut accumulator).unwrap();

        assert_eq!(text.as_deref(), Some("你好世界"));
    }

    #[test]
    fn accumulator_replaces_same_segment_with_latest_partial() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 1,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"你"}]}]}]}}"#).ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 1,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"你好"}]}]}]}}"#).ok(),
        });

        assert_eq!(text, "你好");
    }

    #[test]
    fn final_segment_replaces_live_segment_without_duplication() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 5,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"为什么什么"}]}]}]}}"#).ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 5,
            cn: serde_json::from_str(r#"{"st":{"type":"0","rt":[{"ws":[{"cw":[{"w":"为什么什么"}]}]}]}}"#).ok(),
        });

        assert_eq!(text, "为什么什么");
    }

    #[test]
    fn sequential_segments_are_merged_in_seg_id_order_without_duplication() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 5,
            cn: serde_json::from_str(r#"{"st":{"type":"0","rt":[{"ws":[{"cw":[{"w":"为什么什么"}]}]}]}}"#).ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 6,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"什么原因啊兄弟们"}]}]}]}}"#).ok(),
        });

        assert_eq!(text, "为什么什么原因啊兄弟们");
    }

    #[test]
    fn later_updates_with_same_segment_id_replace_previous_revision() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 5,
            cn: serde_json::from_str(r#"{"st":{"type":"0","rt":[{"ws":[{"cw":[{"w":"第一句"}]}]}]}}"#).ok(),
        });
        accumulator.apply(RtasrResultPayload {
            seg_id: 6,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"第二"}]}]}]}}"#).ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 6,
            cn: serde_json::from_str(r#"{"st":{"type":"0","rt":[{"ws":[{"cw":[{"w":"第二句话"}]}]}]}}"#).ok(),
        });

        assert_eq!(text, "第一句第二句话");
    }

    #[test]
    fn newer_interim_snapshots_replace_older_ones_even_with_new_seg_id() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 1,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"康神"}]}]}]}}"#).ok(),
        });
        accumulator.apply(RtasrResultPayload {
            seg_id: 2,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"康神开播"}]}]}]}}"#).ok(),
        });
        accumulator.apply(RtasrResultPayload {
            seg_id: 3,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"康神开播了"}]}]}]}}"#).ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 4,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"真的假的"}]}]}]}}"#).ok(),
        });

        assert_eq!(text, "康神开播了真的假的");
    }

    #[test]
    fn interim_snapshots_with_same_bg_replace_previous_draft() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 10,
            cn: serde_json::from_str(
                r#"{"st":{"bg":"820","ed":"1200","type":"1","rt":[{"ws":[{"cw":[{"w":"康神开播"}]}]}]}}"#,
            )
            .ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 11,
            cn: serde_json::from_str(
                r#"{"st":{"bg":"820","ed":"1560","type":"1","rt":[{"ws":[{"cw":[{"w":"康神开播了"}]}]}]}}"#,
            )
            .ok(),
        });

        assert_eq!(text, "康神开播了");
    }

    #[test]
    fn final_segment_replaces_matching_interim_draft() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 5,
            cn: serde_json::from_str(
                r#"{"st":{"bg":"820","ed":"1200","type":"1","rt":[{"ws":[{"cw":[{"w":"康神开播"}]}]}]}}"#,
            )
            .ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 5,
            cn: serde_json::from_str(
                r#"{"st":{"bg":"820","ed":"1560","type":"0","rt":[{"ws":[{"cw":[{"w":"康神开播了"}]}]}]}}"#,
            )
            .ok(),
        });

        assert_eq!(text, "康神开播了");
    }

    #[test]
    fn single_character_overlap_is_merged_once() {
        assert_eq!(merge_with_overlap("康神开播了", "了真的假的"), "康神开播了真的假的");
    }

    #[test]
    fn longer_snapshot_replaces_previous_combined_text() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 1,
            cn: serde_json::from_str(
                r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"风格这一块实现主要是由skill实现的，主要涉及一些应用场景是什么样"}]}]}]}}"#,
            )
            .ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 2,
            cn: serde_json::from_str(
                r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"风格这一块实现主要是由skill实现的，主要涉及一些应用场景是什么呀"}]}]}]}}"#,
            )
            .ok(),
        });

        assert_eq!(text, "风格这一块实现主要是由skill实现的，主要涉及一些应用场景是什么呀");
    }

    #[test]
    fn accumulated_interim_text_is_replaced_when_new_snapshot_restarts_from_beginning() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 1,
            cn: serde_json::from_str(
                r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"该功能主要借助可扩展的那个skill模块，实现对语音识别所输出的文字进行智能化"}]}]}]}}"#,
            )
            .ok(),
        });
        accumulator.apply(RtasrResultPayload {
            seg_id: 2,
            cn: serde_json::from_str(
                r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"呃就是核心就是将一些口语化碎片化的识别文本，按照不同的场景"}]}]}]}}"#,
            )
            .ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 3,
            cn: serde_json::from_str(
                r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"该功能主要借助可扩展的那个skill模块，实现对语音识别所输出的文字进行智能化风格的改写，其核心就是将一些口语化碎片化的识别文本，按照不同的场景转换为更规范的文本"}]}]}]}}"#,
            )
            .ok(),
        });

        assert_eq!(
            text,
            "该功能主要借助可扩展的那个skill模块，实现对语音识别所输出的文字进行智能化风格的改写，其核心就是将一些口语化碎片化的识别文本，按照不同的场景转换为更规范的文本"
        );
    }

    #[test]
    fn redundant_tail_inside_live_snapshot_is_ignored() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 1,
            cn: serde_json::from_str(
                r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"该功能主要是借助可扩展的sqw模块实现对语音识别所输入的文字智能化风格改写，就是将一些口语化碎片化的识别文本转换成规范的文本，然后已经内置了4种典型的应用场景，比如通用润色呀邮件撰写问候消息，专业回复，后续还将支持用户自定义的skill来实现不同的应用场景。"}]}]}]}}"#,
            )
            .ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 2,
            cn: serde_json::from_str(
                r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"已经内置了四种典型的应用场景，比如通用润色呀、邮件撰写问候消息专业回复，后续还将支持用户自定义的skill来实现不同"}]}]}]}}"#,
            )
            .ok(),
        });

        assert_eq!(
            text,
            "该功能主要是借助可扩展的sqw模块实现对语音识别所输入的文字智能化风格改写，就是将一些口语化碎片化的识别文本转换成规范的文本，然后已经内置了4种典型的应用场景，比如通用润色呀邮件撰写问候消息，专业回复，后续还将支持用户自定义的skill来实现不同的应用场景。"
        );
    }

    #[test]
    fn timeout_is_clamped_to_five_seconds() {
        let mut settings = AppSettings::default();
        settings.rtasr_timeout_ms = 1_000;

        assert_eq!(request_timeout(&settings), Duration::from_millis(5_000));
    }

    #[test]
    fn startup_read_timeout_uses_full_request_timeout() {
        let timeout = Duration::from_secs(8);

        assert_eq!(startup_read_timeout(timeout), timeout);
    }

    #[test]
    fn streaming_read_timeout_keeps_short_polling_interval() {
        let timeout = Duration::from_secs(8);

        assert_eq!(streaming_read_timeout(timeout), READ_POLL_INTERVAL);
    }

    #[test]
    fn startup_retry_classifier_accepts_transient_transport_errors() {
        let error = AppError::AsrServiceUnavailable(
            "Timed out while connecting to the RTASR gateway. Please check network, proxy, or firewall access to rtasr.xfyun.cn:443.".to_string(),
        );

        assert!(is_retryable_session_start_error(&error));
    }

    #[test]
    fn startup_retry_classifier_rejects_auth_errors() {
        let error = AppError::AsrServiceUnavailable(
            "RTASR authentication failed. Please verify that AppID and APIKey match the realtime transcription service.".to_string(),
        );

        assert!(!is_retryable_session_start_error(&error));
    }

    #[test]
    fn resampling_upsamples_with_linear_interpolation() {
        let bytes = resample_chunk_to_16khz_bytes(&[0, 1_000, 2_000], 8_000);
        let samples = bytes
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();

        assert_eq!(samples, vec![0, 500, 1_000, 1_500, 2_000, 2_000]);
    }

    #[test]
    fn resampling_keeps_16khz_audio_unchanged() {
        let original = [120, -340, 560, -780];
        let bytes = resample_chunk_to_16khz_bytes(&original, 16_000);
        let samples = bytes
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();

        assert_eq!(samples, original);
    }

    #[test]
    fn unchanged_partial_snapshot_is_not_emitted_again() {
        let raw = r#"{
            "action":"result",
            "code":"0",
            "data":"{\"cn\":{\"st\":{\"type\":\"1\",\"rt\":[{\"ws\":[{\"cw\":[{\"w\":\"一二三\"}]}]}]}},\"seg_id\":1}",
            "desc":"success"
        }"#;
        let mut accumulator = ResultAccumulator::default();

        assert_eq!(process_server_message(raw, &mut accumulator).unwrap().as_deref(), Some("一二三"));
        assert_eq!(process_server_message(raw, &mut accumulator).unwrap(), None);
    }

    #[test]
    fn committing_previous_live_segment_keeps_only_new_tail() {
        let mut accumulator = ResultAccumulator::default();
        accumulator.apply(RtasrResultPayload {
            seg_id: 1,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"一二三"}]}]}]}}"#).ok(),
        });

        let text = accumulator.apply(RtasrResultPayload {
            seg_id: 2,
            cn: serde_json::from_str(r#"{"st":{"type":"1","rt":[{"ws":[{"cw":[{"w":"二三四"}]}]}]}}"#).ok(),
        });

        assert_eq!(text, "一二三四");
        assert_eq!(accumulator.final_segments.get(&1).map(String::as_str), Some("一二三"));
    }
}
