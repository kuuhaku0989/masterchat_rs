use base64::{engine::general_purpose, Engine as _};
use core::panic;
use reqwest::{self, header::HeaderMap, Response};
use serde_json;
use sha1_smol::Sha1;
use std::error;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};
use types::*;
use urlencoding;

pub mod types;

const EP_MOD: &str = "/youtubei/v1/live_chat/moderate?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const EP_SM: &str =
    "/youtubei/v1/live_chat/send_message?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const EP_GLC: &str =
    "/youtubei/v1/live_chat/get_live_chat?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const EP_GLCR: &str =
    "/youtubei/v1/live_chat/get_live_chat_replay?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const EP_LCA: &str =
    "/youtubei/v1/live_chat/live_chat_action?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const DO: &str = "https://www.youtube.com";
const SASH: &str = "SAPISIDHASH ";
const XO: &str = "X-Origin";
const XGAU: &str = "X-Goog-AuthUser";
const XGPID: &str = "X-Goog-PageId";
const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36";

// TODO:
//       postWithRetry function
//       proper response handeling and interpreting, and useful Result Errors
//       implement rest of Masterchat

enum B64Type {
    B1,
    B2,
}

enum Payload<'a> {
    StringPl(&'a str),
    ByteArrayPl(Vec<u8>),
    MultiArrayPl(Vec<Vec<u8>>),
}

pub struct ChannelVideoPair {
    channel_id: String,
    video_id: String,
}

pub enum CredentialsType {
    DelegatedCredentials(Credentials),
    None,
}

pub enum ContinuationToken {
    First(bool),
    Continuation(String),
}
impl ContinuationToken {
    pub fn token(&self, cv_pair: &ChannelVideoPair, is_live: &bool) -> String {
        match self {
            ContinuationToken::First(top_chat) => {
                if *is_live {
                    live_reload_continuation(cv_pair, &top_chat)
                } else {
                    replay_timed_continuation(cv_pair, &top_chat)
                }
            }
            ContinuationToken::Continuation(token) => token.to_owned(),
        }
    }
}

pub struct Masterchat<'a> {
    cv_pair: ChannelVideoPair,
    credentials: CredentialsType,
    is_live: bool,
    pub callback: &'a dyn Fn(Vec<ActionType>),
}

impl<'a> Masterchat<'a> {
    pub fn new(channel_id: &str, video_id: &str, callback: &'a dyn Fn(Vec<ActionType>)) -> Self {
        Self {
            cv_pair: ChannelVideoPair {
                channel_id: channel_id.to_owned(),
                video_id: video_id.to_owned(),
            },
            credentials: CredentialsType::None,
            is_live: true,
            callback,
        }
    }

    pub async fn fetch(
        &self,
        token_or_options: ContinuationToken,
    ) -> Result<String, Box<dyn error::Error>> {
        let target = &self.cv_pair;
        let request_url = if self.is_live { EP_GLC } else { EP_GLCR };
        let token = token_or_options.token(target, &self.is_live);

        let mut request_body = chat_context(token);

        while self.is_live {
            let now = time::Instant::now();

            let res = self.post(request_url.to_owned(), request_body).await?;

            let text = &res.text().await?;

            let res: LiveChatResponse = match serde_json::from_str(&text) {
                Ok(t) => t,
                Err(e) => {
                    fs::write("./error_log.txt", text).expect("Failed to write error file");
                    println!("{}", e);
                    panic!();
                }
            };

            if let Some(actions) = &res.continuation_contents.live_chat_continuation.actions {
                (self.callback)(actions.to_owned());
            }

            let token_or_options = ContinuationToken::Continuation(res.continuation_token());
            //println!("{}", res.continuation_token());

            // return Ok(test.continuation_token());

            let token = token_or_options.token(target, &self.is_live);

            request_body = chat_context(token);

            let passed = now.elapsed();
            let timeout_ms = time::Duration::from_millis(res.timeout_ms());

            if passed < timeout_ms {
                let timeout_ms = timeout_ms - passed;
                thread::sleep(timeout_ms);
            }
        }

        Ok(String::from("")) // TODO
    }

    pub async fn send_message(&self, message: &str) -> Result<String, Box<dyn error::Error>> {
        let params = send_message_params(&self.cv_pair);
        let body = message_context(params, message.to_owned());

        let response = self.post(String::from(EP_SM), body).await?;
        // implement the following, gonna require me making network code and errors tho
        /* const res = await this.postWithRetry(EP_SM, body);
        if (res.timeoutDurationUsec) {
            // You are timeouted
            const timeoutSec = usecToSeconds(res.timeoutDurationUsec);
            throw new Error(`You have been placed in timeout for ${timeoutSec} seconds`);
        }
        const item = res.actions?.[0].addChatItemAction?.item;
        if (!(item && "liveChatTextMessageRenderer" in item)) {
            throw new Error(`Invalid response: ` + item);
        }
        return item.liveChatTextMessageRenderer; */

        Ok(response.text().await?) // TODO
    }

    pub async fn remove_message(&self, chat_id: &str) -> Result<String, Box<dyn error::Error>> {
        let params = remove_message_params(chat_id, &self.cv_pair, true);
        let res = self.post(EP_MOD.to_owned(), context(params)).await?;
        // response interpreting and call action handeling and return action
        /*const payload = res.actions[0].markChatItemAsDeletedAction;
        if (!payload) {
            throw new Error(`Invalid response when removing chat: ${JSON.stringify(res)}`);
        }
        return parseMarkChatItemAsDeletedAction(payload);*/
        Ok(res.text().await?) // TODO
    }

    pub async fn timeout(&self, target_channel_id: &str) -> Result<String, Box<dyn error::Error>> {
        let params = timeout_params(target_channel_id, &self.cv_pair);
        let response = self.post(EP_MOD.to_owned(), context(params)).await.unwrap();

        Ok(response.text().await.unwrap()) // TODO
    }

    pub async fn hide(&self, target_channel_id: &str) -> Result<String, Box<dyn error::Error>> {
        let params = hide_params(target_channel_id, &self.cv_pair, false);
        let body = context(params);

        let response = self.post(String::from(EP_MOD), body).await?;

        Ok(response.text().await?) // TODO
    }

    pub async fn pin(&self, chat_id: &str) -> Result<String, Box<dyn error::Error>> {
        let params = pin_params(&chat_id, &self.cv_pair);
        let body = context(params);
        let response = self.post(String::from(EP_LCA), body).await?;
        /*if (!res.success) {
            throw new Error(`Failed to pin chat: ` + JSON.stringify(res));
        }*/
        Ok(response.text().await?) // TODO
    }

    pub async fn unpin(&self, action_id: &str) -> Result<String, Box<dyn error::Error>> {
        let params = unpin_params(&action_id, &self.cv_pair);
        let res = self.post(EP_LCA.to_owned(), context(params)).await?;
        Ok(res.text().await?) // TODO
    }

    async fn post(
        &self,
        mut input: String,
        body: String,
    ) -> Result<Response, Box<dyn error::Error>> {
        let mut default_header = HeaderMap::new();
        default_header.insert("Accept-Language", "en".parse().unwrap());
        default_header.insert("User-Agent", UA.parse().unwrap());

        let creds = build_auth_headers(&self.credentials);

        if !input.starts_with("http") {
            input = DO.to_owned() + &input;
        }

        let client = reqwest::Client::new();
        let res = client
            .post(input)
            .body(body)
            .header("Content", "application/json")
            .headers(default_header)
            .headers(creds)
            .send()
            .await?;

        Ok(res)
    }

    pub fn set_credentials(&mut self, credentials: &str) {
        let credentials = String::from_utf8(b64tou8(credentials)).unwrap();
        let creds: Credentials =
            serde_json::from_str(&credentials).expect("Could not convert Credentials correctly!");
        self.credentials = CredentialsType::DelegatedCredentials(creds);
    }
}

fn send_message_params(cv_pair: &ChannelVideoPair) -> String {
    b64e(
        concatu8(vec![
            ld(1, Payload::ByteArrayPl(cv_token(cv_pair))),
            vt(2, 2),
            vt(3, 4),
        ]),
        B64Type::B2,
    )
}

fn remove_message_params(chat_id: &str, cv_pair: &ChannelVideoPair, retract: bool) -> String {
    b64e(
        concatu8(vec![
            ld(1, Payload::ByteArrayPl(cv_token(cv_pair))),
            ld(
                2,
                Payload::ByteArrayPl(ld(1, Payload::ByteArrayPl(chat_token(chat_id)))),
            ),
            vt(10, if retract { 1 } else { 2 }),
            vt(11, 1),
        ]),
        B64Type::B2,
    )
}

fn timeout_params(channel_id: &str, cv_pair: &ChannelVideoPair) -> String {
    b64e(
        concatu8(vec![
            ld(1, Payload::ByteArrayPl(cv_token(cv_pair))),
            ld(
                6,
                Payload::ByteArrayPl(ld(1, Payload::StringPl(&truc(channel_id.to_owned())))),
            ),
            vt(10, 2),
            vt(11, 1),
        ]),
        B64Type::B2,
    )
}

fn hide_params(channel_id: &str, origin: &ChannelVideoPair, undo: bool) -> String {
    let op = if undo { 5 } else { 4 };

    return b64e(
        concatu8(vec![
            ld(1, Payload::ByteArrayPl(cv_token(origin))),
            ld(
                op,
                Payload::ByteArrayPl(ld(1, Payload::StringPl(&truc(channel_id.to_owned())))),
            ),
            vt(10, 2),
            vt(11, 1),
        ]),
        B64Type::B2,
    );
}

fn pin_params(chat_id: &str, cv_pair: &ChannelVideoPair) -> String {
    b64e(
        ld(
            1,
            Payload::MultiArrayPl(vec![
                ld(1, Payload::ByteArrayPl(cv_token(cv_pair))),
                ld(2, Payload::ByteArrayPl(chat_token(&chat_id))),
                vt(3, 1),
                vt(10, 2),
                vt(11, 1),
            ]),
        ),
        B64Type::B2,
    )
}

fn unpin_params(action_id: &str, cv_pair: &ChannelVideoPair) -> String {
    b64e(
        ld(
            1,
            Payload::MultiArrayPl(vec![
                ld(1, Payload::ByteArrayPl(cv_token(cv_pair))),
                ld(
                    2,
                    Payload::ByteArrayPl(ld(
                        1,
                        Payload::ByteArrayPl(ld(1, Payload::StringPl(action_id))),
                    )),
                ),
                vt(3, 2),
                vt(10, 2),
                vt(11, 1),
            ]),
        ),
        B64Type::B2,
    )
}

fn live_reload_continuation(cv_pair: &ChannelVideoPair, top_chat: &bool) -> String {
    let chat_type = if *top_chat { 4 } else { 1 };
    b64e(
        ld(
            119693434,
            Payload::MultiArrayPl(vec![
                ld(3, Payload::StringPl(&hdt(cv_pair))),
                vt(6, 1),
                ld(16, Payload::ByteArrayPl(vt(1, chat_type))),
            ]),
        ),
        B64Type::B1,
    )
}

fn replay_timed_continuation(_cv_pair: &ChannelVideoPair, _top_chat: &bool) -> String {
    todo!()
}

fn context(params: String) -> String {
    serde_json::to_string(&YoutubeRequestBody::new(params))
        .expect("Could not serialize Context into Json")
}

fn chat_context(continuation: String) -> String {
    serde_json::to_string(&YoutubeChatRequestBody::new(continuation))
        .expect("Could not serialize Context into Json")
}

fn message_context(params: String, message: String) -> String {
    serde_json::to_string(&YoutubeMessageRequestBody::new(params, message))
        .expect("Could not serialize Context into Json")
}

pub fn build_auth_headers(credentials: &CredentialsType) -> HeaderMap {
    let credentials = match credentials {
        CredentialsType::DelegatedCredentials(creds) => creds,
        CredentialsType::None => return HeaderMap::new(),
    };

    let mut auth_header = HeaderMap::new();
    auth_header.insert(
        "Authorization",
        gen_auth_token(&credentials.sapisid, DO).parse().unwrap(),
    );
    auth_header.insert("Cookie", gen_cookie_string(&credentials).parse().unwrap());
    auth_header.insert(XGAU, "0".parse().unwrap());
    auth_header.insert(XGPID, credentials.delegated_session_id.parse().unwrap());
    auth_header.insert(XO, DO.parse().unwrap());
    auth_header
}

fn gen_cookie_string(creds: &Credentials) -> String {
    format!(
        "SID={}; HSID={}; SSID={}; APISID={}; SAPISID={}; DELEGATED_SESSION_ID={};",
        creds.sid, creds.hsid, creds.ssid, creds.apisid, creds.sapisid, creds.delegated_session_id
    )
}

fn gen_auth_token(sid: &str, origin: &str) -> String {
    SASH.to_owned() + &gen_sash(&sid, origin)
}

fn gen_sash(sid: &str, origin: &str) -> String {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    let payload = format!("{} {} {}", &now, sid, origin);
    let digest = sha1_digest(&payload);
    format!("{}_{}", now, digest)
}

fn sha1_digest(payload: &str) -> String {
    Sha1::from(payload).digest().to_string()
}

// Base 64 encoder
fn b64e(payload: Vec<u8>, b64type: B64Type) -> String {
    match b64type {
        B64Type::B1 => urlsafe_b64e(payload),
        B64Type::B2 => {
            let urlsafe = urlsafe_b64e(payload);
            let encoded = encode_utf8(urlsafe);
            u8tob64(encoded)
        }
    }
}

fn b64d(payload: &str, b64type: B64Type) -> Vec<u8> {
    match b64type {
        B64Type::B1 => urlsafe_b64d(payload),
        B64Type::B2 => {
            let b64 = b64tou8(payload);
            let decoded = String::from_utf8(b64).expect("Unable to decode UTF-8 String");
            urlsafe_b64d(&decoded)
        }
    }
}

// urlsafe Base64 encoder
fn urlsafe_b64e(payload: Vec<u8>) -> String {
    encoded_uricomponent(&u8tob64(payload))
}

fn urlsafe_b64d(payload: &str) -> Vec<u8> {
    b64tou8(&decode_uricomponent(payload))
}

fn encoded_uricomponent(uri_component: &str) -> String {
    urlencoding::encode(uri_component).to_string()
}

fn decode_uricomponent(encoded_uricomponent: &str) -> String {
    urlencoding::decode(encoded_uricomponent)
        .expect("Unable to decode URI Component")
        .to_string()
}

// uint 8 to Base 64 converter
fn u8tob64(data: Vec<u8>) -> String {
    general_purpose::STANDARD.encode(data)
}

fn b64tou8(data: &str) -> Vec<u8> {
    general_purpose::STANDARD.decode(data).unwrap()
}

fn cv_token(p: &ChannelVideoPair) -> Vec<u8> {
    ld(
        5,
        Payload::MultiArrayPl(vec![
            ld(1, Payload::StringPl(&p.channel_id)),
            ld(2, Payload::StringPl(&p.video_id)),
        ]),
    )
}

fn chat_token(chat_id: &str) -> Vec<u8> {
    return b64d(chat_id, B64Type::B1);
    // const i = parse(b64d(chatId, B64Type.B1)) as PBToken[];
    // const j = i[0].v as PBToken[];
    // const k = j.map((pbv) => pbv.v) as [string, string];
    // return [ld(1, k[0]), ld(2, k[1])];
}

fn ld(fid: u64, payload: Payload) -> Vec<u8> {
    let b: Vec<u8> = match payload {
        Payload::StringPl(input) => encode_utf8(input.to_owned()),
        Payload::ByteArrayPl(byte_array) => byte_array,
        Payload::MultiArrayPl(multi_array) => concatu8(multi_array),
    };
    let b_length = b.len();
    concatu8(vec![
        bitou8(pbh(fid, 2)),
        bitou8(encv(b_length.try_into().unwrap())),
        b,
    ])
}

fn vt(fid: u64, payload: u64) -> Vec<u8> {
    concatu8(vec![bitou8(pbh(fid, 0)), bitou8(payload)])
}

// probably protobuf encoder?
fn pbh(fid: u64, pbh_type: u64) -> u64 {
    encv((fid << 3) | pbh_type)
}

fn encv(mut n: u64) -> u64 {
    // everything in here also bigint
    let mut s = 0;
    while (n >> 7) != 0 {
        s = (s << 8) | 0x80 | (n & 0x7f);
        n >>= 7;
    }
    s = (s << 8) | n;
    return s;
}

// converts bigint to hex to u8 array
fn bitou8(n: u64) -> Vec<u8> {
    let mut bytes = n.to_be_bytes().to_vec();
    let leading_zeros = bytes.iter().position(|byte| *byte > 0);
    match leading_zeros {
        Some(amount) => {
            bytes.drain(0..amount);
            bytes
        }
        None => bytes,
    }
}

// concats u8 arrays
// args array of u8 arrays
fn concatu8(mut args: Vec<Vec<u8>>) -> Vec<u8> {
    let mut out = args[0].clone();
    let mut iterator = args.iter_mut();
    iterator.next();

    for arg in iterator {
        out.append(arg);
    }

    out
}

fn hdt(tgt: &ChannelVideoPair) -> String {
    u8tob64(concatu8(vec![
        ld(1, Payload::ByteArrayPl(cv_token(tgt))),
        ld(
            3,
            Payload::ByteArrayPl(ld(
                48687757,
                Payload::ByteArrayPl(ld(1, Payload::StringPl(&tgt.video_id))),
            )),
        ),
        vt(4, 1),
    ]))
}

fn truc(mut i: String) -> String {
    if i.starts_with("UC") {
        i.drain(0..2);
    }

    i
}

//Convert String to UTF-8 uint8 array
fn encode_utf8(input: String) -> Vec<u8> {
    input.into_bytes()
}

#[cfg(test)]
mod tests {}
