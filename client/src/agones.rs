use std::net::SocketAddr;
use reqwest::{Client, Request, Url};
use reqwest::Method;

pub enum AgonesApi {
    AllocateRoom,
}

impl AgonesApi {
    pub fn path(&self) -> &str {
        match self {
            AgonesApi::AllocateRoom => "/room/allocate",
        }
    }

    pub fn url(&self, base_url: &str) -> Url {
        format!("{}{}", base_url, self.path()).parse().unwrap()
    }
}

struct AgonesUrlBuf {
    pub allocate_room: Url,
}

impl AgonesUrlBuf {
    pub fn create(base_url: &str) -> Self {
        AgonesUrlBuf {
            allocate_room: AgonesApi::AllocateRoom.url(base_url),
        }
    }
}


pub struct AgonesClient {
    pub client: Client,
    pub base_url: String,

    url_buf: AgonesUrlBuf,
}

impl AgonesClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::new();
        let url_buf = AgonesUrlBuf::create(&base_url);

        AgonesClient {
            client,
            base_url,
            url_buf,
        }
    }

    pub async fn allocate(&self) -> Result<SocketAddr, reqwest::Error> {
        let req = Request::new(Method::POST, self.url_buf.allocate_room.clone());
        let resp = self.client.execute(req).await?;
        let resp = resp.json::<serde_json::Value>().await?;

        let ip = resp["status"]["address"].as_str().unwrap();
        let port = resp["status"]["port"].as_u64().unwrap() as u16;

        Ok(SocketAddr::new(ip.parse().unwrap(), port))
    }
}
