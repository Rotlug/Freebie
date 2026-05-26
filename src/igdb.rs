//! Fetches metadata about video games, such as rating, title, description and cover art
//! from IGDB.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use tokio::sync::Mutex;

/// The token returned from igdb after requesting access
#[serde_as]
#[derive(Deserialize, Debug)]
struct AccessToken {
    #[serde_as(as = "DurationSeconds<u64>")]
    expires_in: Duration,
    access_token: String,
}

/// The credentials used to get an `AccessToken` from the api
#[derive(Default)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

/// `MetadataManager` is used to request metadata about multiple video games at once
/// from igdb, and prevents the access to the api from expiring
#[derive(Default)]
pub struct MetadataManager {
    /// Used to make network requests
    reqwest_client: reqwest::Client,

    /// The access token returned by the api, or `None` if access hasn't been requested yet.
    access_token: Arc<Mutex<Option<AccessToken>>>,
    /// The `Instant` when the `access_token` has last been renewed, or `None` if
    /// The `access_token` hasn't been renewed yet.
    access_last_renewed: Arc<Mutex<Option<Instant>>>,

    credentials: Credentials,
}

impl MetadataManager {
    pub fn new(credentials: Credentials) -> Self {
        let reqwest_client = reqwest::Client::new();

        Self {
            reqwest_client,
            credentials,
            ..Default::default()
        }
    }

    /// Checks if the authentication token has expired.
    pub async fn expired(&self) -> bool {
        if let Some(ref access_token) = *self.access_token.lock().await
            && let Some(access_last_renewed) = *self.access_last_renewed.lock().await
        {
            access_last_renewed.elapsed() > access_token.expires_in
        } else {
            true
        }
    }

    /// Requests a new authentication token
    pub async fn authenticate(&self) -> anyhow::Result<()> {
        let url = format!(
            "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials",
            self.credentials.client_id, self.credentials.client_secret
        );

        let resp = self.reqwest_client.post(url).send().await?.text().await?;
        let token: AccessToken = serde_json::from_str(&resp)?;

        *self.access_token.lock().await = Some(token);
        *self.access_last_renewed.lock().await = Some(Instant::now());

        Ok(())
    }

    /// Given a list of game names, returns relevant metadata about said games.
    pub async fn get_games(
        &self,
        slugs: &[impl AsRef<str>],
    ) -> anyhow::Result<HashMap<String, Metadata>> {
        let mut metadatas = HashMap::new();

        let mut futures = vec![];

        // igdb allows for maximmum 20 games at a time
        for chunk in slugs.chunks(20) {
            let data = format!(
                "fields cover.url,name,summary,slug,aggregated_rating; where slug = {} & cover.url != null; limit 20;",
                Self::combine(chunk)
            );

            futures.push(async {
                let resp = self.fetch("https://api.igdb.com/v4/games", data).await?;
                let mut batch: Vec<Metadata> = serde_json::from_str(&resp)?;
                // Fix the cover.url
                for meta in &mut batch {
                    meta.cover.url =
                        format!("https:/{}", meta.cover.url.replace("thumb", "cover_big"));
                }

                Ok::<Vec<Metadata>, anyhow::Error>(batch)
            });
        }

        let results = futures_util::future::join_all(futures).await;
        for batch in results.into_iter().flatten() {
            for meta in batch {
                metadatas.insert(meta.slug.clone(), meta);
            }
        }

        Ok(metadatas)
    }

    /// General command to authenticate if needed and then fetch data from igdb
    async fn fetch(&self, url: &str, data: String) -> anyhow::Result<String> {
        if self.expired().await {
            self.authenticate().await?;
        }

        Ok(self
            .reqwest_client
            .post(url)
            .header("Client-ID", &self.credentials.client_id)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.access_token
                        .lock()
                        .await
                        .as_ref()
                        .unwrap()
                        .access_token
                ),
            )
            .body(data)
            .send()
            .await?
            .text()
            .await?)
    }

    /// Combine multiple strings into a single string that can be used in the api's `where` clauses.
    ///
    /// Example:
    /// \["minecraft", "terraria", "runescape"\] -> "("minecraft", "terraria", "runescape")"
    fn combine(strings: &[impl AsRef<str>]) -> String {
        let quoted_strings: Vec<String> = strings
            .iter()
            .map(|s| format!("\"{}\"", s.as_ref()))
            .collect();

        format!("({})", quoted_strings.join(","))
    }
}

/// Metadata about a video game
#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    /// The games title.
    pub name: String,
    /// The games url-friendly name.
    pub slug: String,
    /// A link to the games cover art.
    pub cover: Cover,
    /// A short description of the game.
    #[serde(rename = "summary")]
    pub description: Option<String>,
    /// The game's rating, based of reviews from igdb users and game critics.
    /// Represented as an `f32` between 0-100.
    #[serde(rename = "aggregated_rating")]
    pub rating: Option<f32>,
}

impl Hash for Metadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.slug.hash(state);
    }
}

/// The video game's cover art
#[derive(Debug, Deserialize, Serialize)]
pub struct Cover {
    #[serde(rename = "id")]
    _id: i32,
    pub url: String,

    #[serde(skip)]
    pub texture_cache: Mutex<Option<relm4::gtk::gdk::Texture>>,
}

impl Cover {
    pub async fn download(&self) -> reqwest::Result<Vec<u8>> {
        let bytes = reqwest::get(&self.url)
            .await?
            .bytes()
            .await?
            .into_iter()
            .collect();

        Ok(bytes)
    }
}
