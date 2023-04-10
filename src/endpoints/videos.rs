use super::general::ApiState;
use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use querystring::querify;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use sqlx::{Error, FromRow};
use url::Url;

#[derive(Debug, FromRow, Serialize)]
pub struct Video {
    pub id: i32,
    pub channel_id: i32,
    pub title: String,
    pub url: String,
    #[serde(with = "ts_seconds_option")]
    pub upload_datetime: Option<DateTime<Utc>>,
    pub views: i64,
    pub length: i32,
}

#[get("/video/all")]
pub async fn get_videos(state: &State<ApiState>) -> Json<Option<Vec<Video>>> {
    let videos = sqlx::query_as::<_, Video>("select * from videos limit 50;")
        .fetch_all(&state.pool)
        .await;

    dbg!(&videos);

    match videos {
        Ok(v) => Json(Some(v)),
        Err(_) => Json(None),
    }
}

#[derive(Debug, Deserialize)]
pub struct NewVideoUrl {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeVideoResponse {
    items: Vec<YouTubeVideoItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeVideoItem {
    id: String,
    snippet: YouTubeVideoSnippet,
    statistics: YouTubeVideoStatistics,
    #[serde(rename = "contentDetails")]
    content_details: YouTubeVideoContentDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeVideoSnippet {
    #[serde(rename = "publishedAt")]
    published_at: String,
    #[serde(rename = "channelId")]
    channel_id: String,
    title: String,
    thumbnails: YouTubeVideoThumbnailTypes,
    #[serde(rename = "channelTitle")]
    channel_title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeVideoContentDetails {
    duration: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeVideoThumbnailTypes {
    default: YouTubeVideoThumbnail,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeVideoThumbnail {
    url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeVideoStatistics {
    #[serde(rename = "viewCount")]
    view_count: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateVideoResponse {
    pub success: bool,
    pub id: i32,
}

#[post("/video", data = "<video_url>")]
pub async fn create_video(
    video_url: Json<NewVideoUrl>,
    state: &State<ApiState>,
) -> Json<CreateVideoResponse> {
    // let mut video_id = "".to_string();

    // {
    //     let dog = "cat".to_string();
    //     video_id = dog.clone();
    // }

    // dbg!(&video_id);

    // Json(CreateVideoResponse {
    //     success: true,
    //     id: 24,
    // })

    let url = video_url.url.clone();
    let mut video_id = "".to_string();

    // Grab the id out of the url string Urls can be in either of the following formats:
    // (1) https://www.youtube.com/watch?v=TTjYjSEGHek
    // (2) https://youtu.be/TTjYjSEGHek
    if url.contains("youtu.be") {
        todo!();
    } else {
        let parsed_url = Url::parse(&url).unwrap();
        let qs = parsed_url.query().unwrap();
        let qs_parts = querify(qs);
        let temp = qs_parts
            .iter()
            .find(|&&q| q.0 == "v")
            .unwrap()
            .1
            .to_string();
        video_id = temp;
    }

    dbg!(&video_id);

    let youtube_api_url = format!("https://www.googleapis.com/youtube/v3/videos?key=AIzaSyAJER_goEuZNztE5XRitR-roJfHvSsUO9Q&part=id,snippet,statistics,contentDetails&id={video_id}");
    let video = reqwest::get(youtube_api_url)
        .await
        .unwrap()
        .json::<YouTubeVideoResponse>()
        .await
        .unwrap();

    let video_to_insert: YouTubeVideoItem = video.items[0].clone();

    let result: Result<i32, Error> =
        sqlx::query_scalar("insert into videos (channel_id, title, url, upload_datetime, views, length, thumbnail) values ($1, $2, $3, $4, $5, $6, $7) returning id")
            .bind(1)
            // .bind(video_to_insert.snippet.channel_id)
            .bind(video_to_insert.snippet.channel_title)
            .bind(video_url.url.clone())
            // .bind(video_to_insert.snippet.published_at)
            .bind(Utc::now())
            .bind(video_to_insert.statistics.view_count.parse::<i64>().unwrap())
            .bind(30)
            // .bind(video_to_insert.content_details.duration)
            .bind(video_to_insert.snippet.thumbnails.default.url)
            .fetch_one(&state.pool)
            .await;

    Json(CreateVideoResponse {
        success: true,
        id: result.unwrap(),
    })
}
