pub fn jellyfin_image_url(
    server_url: &str,
    item_id: &str,
    image_tag: Option<&str>,
    access_token: Option<&str>,
    max_width: u32,
    quality: u32,
) -> String {
    let mut params = Vec::new();
    params.push(format!("maxWidth={}", max_width));
    params.push(format!("quality={}", quality));

    if let Some(tag) = image_tag {
        params.push(format!("tag={}", tag));
    }
    if let Some(token) = access_token {
        params.push(format!("api_key={}", token));
    }

    let mut url = format!("{}/Items/{}/Images/Primary", server_url, item_id);
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
}

pub fn parse_jellyfin_path(path_str: &str) -> Option<(&str, Option<&str>)> {
    let parts: Vec<&str> = path_str.split(':').collect();
    if parts.len() >= 2 {
        let id = parts[1];
        let tag = if parts.len() >= 3 {
            Some(parts[2])
        } else {
            None
        };
        Some((id, tag))
    } else {
        None
    }
}

pub fn jellyfin_image_url_from_path(
    path_str: &str,
    server_url: &str,
    access_token: Option<&str>,
    max_width: u32,
    quality: u32,
) -> Option<String> {
    let (id, tag) = parse_jellyfin_path(path_str)?;
    Some(jellyfin_image_url(
        server_url,
        id,
        tag,
        access_token,
        max_width,
        quality,
    ))
}

pub fn track_cover_url_with_album_fallback(
    track_path_str: &str,
    album_id_str: &str,
    server_url: &str,
    access_token: Option<&str>,
    max_width: u32,
    quality: u32,
) -> Option<String> {
    if let Some((id, Some(tag))) = parse_jellyfin_path(track_path_str) {
        return Some(jellyfin_image_url(
            server_url,
            id,
            Some(tag),
            access_token,
            max_width,
            quality,
        ));
    }

    if !album_id_str.is_empty() {
        if let Some((album_item_id, album_tag)) = parse_jellyfin_path(album_id_str) {
            return Some(jellyfin_image_url(
                server_url,
                album_item_id,
                album_tag,
                access_token,
                max_width,
                quality,
            ));
        }
    }

    if let Some((id, _)) = parse_jellyfin_path(track_path_str) {
        return Some(jellyfin_image_url(
            server_url,
            id,
            None,
            access_token,
            max_width,
            quality,
        ));
    }

    None
}
