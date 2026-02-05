use jellyfin_sdk_rust::JellyfinSDK;
use serde::{Deserialize, Serialize};

pub struct JellyfinRemote {
    client: JellyfinSDK,
    base_url: String,
    device_id: String,
    user_id: Option<String>,
    access_token: Option<String>,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    #[serde(rename = "Username")]
    username: &'a str,
    #[serde(rename = "Pw")]
    password: &'a str,
}

#[derive(Deserialize)]
struct LoginResponse {
    #[serde(rename = "AccessToken")]
    access_token: String,
    #[serde(rename = "User")]
    #[allow(dead_code)]
    user: UserObj,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct UserObj {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ViewItemsResponse {
    pub items: Vec<ViewItem>,
    pub total_record_count: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ViewItem {
    pub name: String,
    pub id: String,
    pub collection_type: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ItemsResponse {
    pub items: Vec<Item>,
    pub total_record_count: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub name: String,
    pub id: String,
    #[serde(rename = "Type")]
    pub item_type: String,
    pub run_time_ticks: Option<u64>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub artists: Option<Vec<String>>,
    pub album_artist: Option<String>,
    pub image_tags: Option<std::collections::HashMap<String, String>>,
    pub index_number: Option<u32>,
    pub parent_index_number: Option<u32>,
    pub production_year: Option<u16>,
    pub genres: Option<Vec<String>>,
    pub container: Option<String>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AlbumItem {
    pub name: String,
    pub id: String,
    pub album_artist: Option<String>,
    pub artists: Option<Vec<String>>,
    pub production_year: Option<u16>,
    pub genres: Option<Vec<String>>,
    pub image_tags: Option<std::collections::HashMap<String, String>>,
    pub child_count: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AlbumsResponse {
    pub items: Vec<AlbumItem>,
    pub total_record_count: u32,
}

impl JellyfinRemote {
    pub fn new(
        base_url: &str,
        api_key: Option<&str>,
        device_id: &str,
        user_id: Option<&str>,
    ) -> Self {
        let mut client = JellyfinSDK::new();
        let clean_base_url = base_url.trim_end_matches('/');
        client.create_api(clean_base_url, api_key);

        Self {
            client,
            base_url: clean_base_url.to_string(),
            device_id: device_id.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            access_token: api_key.map(|s| s.to_string()),
        }
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(String, String), String> {
        let url = format!("{}/Users/AuthenticateByName", self.base_url);

        let client = reqwest::Client::new();
        let body = LoginRequest { username, password };

        let auth_header = format!(
            "MediaBrowser Client=\"Rusic\", Device=\"Rusic\", DeviceId=\"{}\", Version=\"0.1.0\"",
            self.device_id
        );

        let resp = client
            .post(&url)
            .header("X-Emby-Authorization", auth_header)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Login failed with status: {} - {}", status, text));
        }

        let login_resp: LoginResponse = resp.json().await.map_err(|e| e.to_string())?;

        self.access_token = Some(login_resp.access_token.clone());
        self.user_id = Some(login_resp.user.id.clone());

        self.client
            .create_api(&self.base_url, Some(&login_resp.access_token));

        Ok((login_resp.access_token, login_resp.user.id))
    }

    pub async fn get_views(&self) -> Result<Vec<ViewItem>, String> {
        let user_id = self.user_id.as_ref().ok_or("No user ID available")?;
        let token = self
            .access_token
            .as_ref()
            .ok_or("No access token available")?;

        let url = format!("{}/Users/{}/Views", self.base_url, user_id);
        let client = reqwest::Client::new();

        let auth_header = format!(
            "MediaBrowser Client=\"Rusic\", Device=\"Rusic\", DeviceId=\"{}\", Version=\"0.1.0\", Token=\"{}\"",
            self.device_id, token
        );

        let resp = client
            .get(&url)
            .header("X-Emby-Authorization", auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Failed to get views: {}", resp.status()));
        }

        let views_resp: ViewItemsResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok(views_resp.items)
    }

    pub async fn get_music_libraries(&self) -> Result<Vec<ViewItem>, String> {
        let views = self.get_views().await?;
        let music_libs = views
            .into_iter()
            .filter(|v| v.collection_type.as_deref() == Some("music"))
            .collect();
        Ok(music_libs)
    }

    pub async fn get_library_items(&self, parent_id: &str) -> Result<Vec<Item>, String> {
        let user_id = self.user_id.as_ref().ok_or("No user ID available")?;
        let token = self
            .access_token
            .as_ref()
            .ok_or("No access token available")?;

        let url = format!("{}/Users/{}/Items", self.base_url, user_id);
        let client = reqwest::Client::new();

        let auth_header = format!(
            "MediaBrowser Client=\"Rusic\", Device=\"Rusic\", DeviceId=\"{}\", Version=\"0.1.0\", Token=\"{}\"",
            self.device_id, token
        );

        let resp = client
            .get(&url)
            .query(&[("ParentId", parent_id)])
            .header("X-Emby-Authorization", auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Failed to get library items: {}", resp.status()));
        }

        let items_resp: ItemsResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok(items_resp.items)
    }

    pub async fn get_music_library_items(&self, parent_id: &str) -> Result<Vec<Item>, String> {
        let user_id = self.user_id.as_ref().ok_or("No user ID available")?;
        let token = self
            .access_token
            .as_ref()
            .ok_or("No access token available")?;

        let url = format!("{}/Users/{}/Items", self.base_url, user_id);
        let client = reqwest::Client::new();

        let auth_header = format!(
            "MediaBrowser Client=\"Rusic\", Device=\"Rusic\", DeviceId=\"{}\", Version=\"0.1.0\", Token=\"{}\"",
            self.device_id, token
        );

        let resp = client
            .get(&url)
            .query(&[
                ("ParentId", parent_id),
                ("Recursive", "true"),
                ("IncludeItemTypes", "Audio"),
                (
                    "Fields",
                    "DateCreated,DateLastMediaAdded,MediaSources,ImageTags,Genres,ParentIndexNumber,IndexNumber,AlbumId,AlbumArtist,ProductionYear,Container",
                ),
            ])
            .header("X-Emby-Authorization", auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Failed to get music items: {}", resp.status()));
        }

        let items_resp: ItemsResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok(items_resp.items)
    }

    pub async fn get_music_library_items_paginated(
        &self,
        parent_id: &str,
        start_index: usize,
        limit: usize,
    ) -> Result<Vec<Item>, String> {
        let user_id = self.user_id.as_ref().ok_or("No user ID available")?;
        let token = self
            .access_token
            .as_ref()
            .ok_or("No access token available")?;

        let url = format!("{}/Users/{}/Items", self.base_url, user_id);
        let client = reqwest::Client::new();

        let auth_header = format!(
            "MediaBrowser Client=\"Rusic\", Device=\"Rusic\", DeviceId=\"{}\", Version=\"0.1.0\", Token=\"{}\"",
            self.device_id, token
        );

        let start = start_index.to_string();
        let limit_val = limit.to_string();

        let resp = client
            .get(&url)
            .query(&[
                ("ParentId", parent_id),
                ("Recursive", "true"),
                ("IncludeItemTypes", "Audio"),
                (
                    "Fields",
                    "DateCreated,DateLastMediaAdded,MediaSources,ImageTags,Genres,ParentIndexNumber,IndexNumber,AlbumId,AlbumArtist,ProductionYear,Container",
                ),
                ("StartIndex", start.as_str()),
                ("Limit", limit_val.as_str()),
            ])
            .header("X-Emby-Authorization", auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Failed to get music items: {}", resp.status()));
        }

        let items_resp: ItemsResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok(items_resp.items)
    }

    pub async fn get_albums(&self, parent_id: &str) -> Result<Vec<AlbumItem>, String> {
        let user_id = self.user_id.as_ref().ok_or("No user ID available")?;
        let token = self
            .access_token
            .as_ref()
            .ok_or("No access token available")?;

        let url = format!("{}/Users/{}/Items", self.base_url, user_id);
        let client = reqwest::Client::new();

        let auth_header = format!(
            "MediaBrowser Client=\"Rusic\", Device=\"Rusic\", DeviceId=\"{}\", Version=\"0.1.0\", Token=\"{}\"",
            self.device_id, token
        );

        let resp = client
            .get(&url)
            .query(&[
                ("ParentId", parent_id),
                ("Recursive", "true"),
                ("IncludeItemTypes", "MusicAlbum"),
                (
                    "Fields",
                    "ImageTags,Genres,ProductionYear,AlbumArtist,ChildCount",
                ),
                ("SortBy", "SortName"),
                ("SortOrder", "Ascending"),
            ])
            .header("X-Emby-Authorization", auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Failed to get albums: {}", resp.status()));
        }

        let albums_resp: AlbumsResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok(albums_resp.items)
    }

    pub async fn get_albums_paginated(
        &self,
        parent_id: &str,
        start_index: usize,
        limit: usize,
    ) -> Result<(Vec<AlbumItem>, u32), String> {
        let user_id = self.user_id.as_ref().ok_or("No user ID available")?;
        let token = self
            .access_token
            .as_ref()
            .ok_or("No access token available")?;

        let url = format!("{}/Users/{}/Items", self.base_url, user_id);
        let client = reqwest::Client::new();

        let auth_header = format!(
            "MediaBrowser Client=\"Rusic\", Device=\"Rusic\", DeviceId=\"{}\", Version=\"0.1.0\", Token=\"{}\"",
            self.device_id, token
        );

        let start = start_index.to_string();
        let limit_val = limit.to_string();

        let resp = client
            .get(&url)
            .query(&[
                ("ParentId", parent_id),
                ("Recursive", "true"),
                ("IncludeItemTypes", "MusicAlbum"),
                (
                    "Fields",
                    "ImageTags,Genres,ProductionYear,AlbumArtist,ChildCount",
                ),
                ("SortBy", "SortName"),
                ("SortOrder", "Ascending"),
                ("StartIndex", start.as_str()),
                ("Limit", limit_val.as_str()),
            ])
            .header("X-Emby-Authorization", auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Failed to get albums: {}", resp.status()));
        }

        let albums_resp: AlbumsResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok((albums_resp.items, albums_resp.total_record_count))
    }

    pub fn build_image_url(&self, item_id: &str, image_tag: Option<&str>) -> String {
        let mut url = format!("{}/Items/{}/Images/Primary", self.base_url, item_id);
        let mut params = Vec::new();

        if let Some(tag) = image_tag {
            params.push(format!("tag={}", tag));
        }
        if let Some(token) = &self.access_token {
            params.push(format!("api_key={}", token));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        url
    }
}
