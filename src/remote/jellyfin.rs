use jellyfin_sdk_rust::JellyfinSDK;

pub struct JellyfinRemote {
    client: JellyfinSDK,
}

impl JellyfinRemote {
    pub fn new(base_url: &str, api_key: Option<&str>) -> Self {
        let mut client = JellyfinSDK::new();
        client.create_api(base_url, api_key);

        Self { client }
    }

    pub async fn get_library_items(&self) -> Result<Vec<String>, String> {
        unimplemented!()
    }
}
