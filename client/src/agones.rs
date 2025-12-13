use reqwest::{Client, Url};

pub enum AgonesApi {
    AllocateRoom,
}

impl AgonesApi {
    pub fn path(&self) -> &str {
        match self {
            AgonesApi::AllocateRoom => "/room/allocate",
        }
    }

    pub fn url(&self, base_url: &Url) -> Url {
        base_url.join(self.path()).unwrap()
    }
}

#[derive(Debug)]
struct AgonesUrlBuf {
    pub allocate_room: Url,
}

impl AgonesUrlBuf {
    pub fn create(base_url: &Url) -> Self {
        AgonesUrlBuf {
            allocate_room: AgonesApi::AllocateRoom.url(base_url),
        }
    }
}

#[derive(Debug)]
pub struct AgonesClient {
    pub client: Client,
    pub base_url: Url,

    url_buf: AgonesUrlBuf,
}

impl AgonesClient {
    pub fn new(base_url: Url) -> Self {
        let client = Client::new();
        let url_buf = AgonesUrlBuf::create(&base_url);

        AgonesClient {
            client,
            base_url,
            url_buf,
        }
    }

    #[cfg(not(feature = "agones"))]
    pub async fn allocate(&self) -> Result<Url, reqwest::Error> {
        Ok("ws://localhost:55555/".parse().unwrap())
    }

    #[cfg(feature = "agones")]
    pub async fn allocate(&self) -> Result<Url, reqwest::Error> {
        let req = Request::new(Method::POST, self.url_buf.allocate_room.clone());
        let resp = self.client.execute(req).await?;
        let resp = resp.json::<serde_json::Value>().await?;

        // let ip = resp["status"]["address"].as_str().unwrap();
        // let port = resp["status"]["port"].as_u64().unwrap() as u16;
        let url = resp["status"]["url"].as_str().unwrap();

        Ok(url.parse().unwrap())
    }
}
