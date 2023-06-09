use crate::endpoints::general::ApiState;
use crate::utils::captions::fetch_captions;
use crate::utils::environment::get_env;
use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use querystring::querify;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket::{get, post};
use sqlx::{Error, FromRow};
use url::Url;

use super::general::SuccessFailResponse;

#[derive(Debug, Clone, Deserialize, FromRow, Serialize)]
pub struct Video {
    pub id: i32,
    pub channel_id: i32,
    pub channel_title: String,
    pub title: String,
    pub url: String,
    pub captions: String,
    #[serde(with = "ts_seconds_option")]
    pub upload_datetime: Option<DateTime<Utc>>,
    pub views: i64,
    pub length: i32,
    pub thumbnail: String,
    pub youtube_id: String,
}

#[get("/video/all")]
pub async fn get_videos(state: &State<ApiState>) -> Json<Option<Vec<Video>>> {
    let videos = sqlx::query_as::<_, Video>(
        "select v.id, v.channel_id, ch.title as channel_title, v.title, v.url, LEFT(ca.raw_text, 400) as captions, v.upload_datetime, v.views, v.length, v.thumbnail, v.youtube_id from videos v
        join captions ca on ca.video_id=v.id
        join channels ch on ch.id=v.channel_id
        limit 50",
    )
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeChannelResponse {
    items: Vec<YouTubeChannelItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeChannelItem {
    id: String,
    snippet: YouTubeChannelSnippet,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeChannelSnippet {
    title: String,
    thumbnails: YouTubeChannelThumbnailTypes,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeChannelThumbnailTypes {
    default: YouTubeChannelThumbnail,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct YouTubeChannelThumbnail {
    url: String,
}

#[derive(Debug, FromRow, Serialize)]
struct RowId {
    id: i32,
}

#[post("/video", data = "<video_url>")]
pub async fn create_video(
    video_url: Json<NewVideoUrl>,
    state: &State<ApiState>,
) -> Json<CreateVideoResponse> {
    let url = video_url.url.clone();
    let youtube_video_id: String;

    // Grab the id out of the url string Urls can be in either of the following formats:
    // (1) https://youtu.be/TTjYjSEGHek
    // (2) https://www.youtube.com/watch?v=TTjYjSEGHek
    if url.contains("youtu.be") {
        youtube_video_id = Url::parse(&url).unwrap().path().split_at(1).1.to_string();
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
        youtube_video_id = temp;
    }

    let youtube_api_key = get_env("YOUTUBE_API_KEY");

    let youtube_api_url = format!("https://www.googleapis.com/youtube/v3/videos?key={youtube_api_key}&part=id,snippet,statistics,contentDetails&id={youtube_video_id}");
    let video = reqwest::get(youtube_api_url)
        .await
        .unwrap()
        .json::<YouTubeVideoResponse>()
        .await
        .unwrap();

    let video_to_insert: YouTubeVideoItem = video.items[0].clone();
    let channel_youtube_id = video_to_insert.snippet.channel_id.clone();

    // Check channels table if channel id already exists
    let row = sqlx::query_as::<_, RowId>("select id from channels where youtube_id=$1")
        .bind(&channel_youtube_id)
        .fetch_one(&state.pool)
        .await;

    let mut channel_id: i32 = -1;
    let youtube_api_key = get_env("YOUTUBE_API_KEY");

    if let Err(_) = row {
        // Fetch the channel details, and insert them into the channels table
        let youtube_api_channel_url = format!("https://www.googleapis.com/youtube/v3/channels?key={youtube_api_key}&part=id,snippet,statistics,contentDetails&id={channel_youtube_id}");
        let channel = reqwest::get(&youtube_api_channel_url)
            .await
            .unwrap()
            .json::<YouTubeChannelResponse>()
            .await
            .unwrap();

        dbg!(&channel);
        let channel_to_insert = channel.items[0].clone();

        let result: Result<i32, Error> =
        sqlx::query_scalar("insert into channels (title, url, thumbnail, youtube_id) values ($1, $2, $3, $4) returning id")
            .bind(channel_to_insert.snippet.title)
            .bind(format!("https://youtube.com/channel/{}", channel_to_insert.id))
            .bind(channel_to_insert.snippet.thumbnails.default.url)
            .bind(channel_to_insert.id)
            .fetch_one(&state.pool)
            .await;

        channel_id = result.unwrap();
    } else if let Ok(r) = row {
        channel_id = r.id;
    }

    let result: Result<i32, Error> =
        sqlx::query_scalar("insert into videos (channel_id, title, url, upload_datetime, views, length, thumbnail, youtube_id) values ($1, $2, $3, $4, $5, $6, $7, $8) returning id")
            .bind(channel_id)
            .bind(video_to_insert.snippet.title)
            .bind(video_url.url.clone())
            // .bind(video_to_insert.snippet.published_at)
            .bind(Utc::now())
            .bind(video_to_insert.statistics.view_count.parse::<i64>().unwrap())
            .bind(30)
            // .bind(video_to_insert.content_details.duration)
            .bind(video_to_insert.snippet.thumbnails.default.url)
            .bind(youtube_video_id.clone())
            .fetch_one(&state.pool)
            .await;

    let video_id = result.unwrap();
    let video_captions = fetch_captions(youtube_video_id.clone()).await;
    let raw_text = video_captions
        .iter()
        .fold(String::new(), |acc, s| acc + &s.text + " ");
    let caption_id_result: Result<i32, Error> = sqlx::query_scalar(
        "insert into captions (video_id, raw_text, caption_json) values ($1, $2, $3) returning id",
    )
    .bind(video_id)
    .bind(raw_text)
    .bind(sqlx::types::Json(&video_captions))
    .fetch_one(&state.pool)
    .await;

    dbg!(&video_captions);

    let caption_id = caption_id_result.unwrap();
    dbg!(caption_id);

    let video_ids = video_captions
        .iter()
        .map(|_c| video_id)
        .collect::<Vec<i32>>();
    let caption_ids = video_captions
        .iter()
        .map(|_c| caption_id)
        .collect::<Vec<i32>>();
    let caption_texts = video_captions
        .iter()
        .map(|c| c.text.clone())
        .collect::<Vec<String>>();
    let caption_starts = video_captions.iter().map(|c| c.start).collect::<Vec<f32>>();
    let caption_durations = video_captions
        .iter()
        .map(|c| c.duration)
        .collect::<Vec<f32>>();

    let caption_timestamp_result: Result<i32, Error> = sqlx::query_scalar(
        "insert into caption_timestamps (video_id, caption_id, caption_text, start, duration) select * from unnest($1, $2, $3, $4, $5) returning id",
    )
        .bind(video_ids)
        .bind(caption_ids)
        .bind(caption_texts)
        .bind(caption_starts)
        .bind(caption_durations)
        .fetch_one(&state.pool)
        .await;

    dbg!(caption_timestamp_result.unwrap());

    Json(CreateVideoResponse {
        success: true,
        id: video_id,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CaptionTextSnippet {
    pub url: String,
    pub caption_text: String,
    pub start: f64,
}

// A struct for "bucketing" together caption snippets into the same video
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CaptionSearchResults {
    pub success: bool,
    pub videos: Vec<VideoCaptionsResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VideoCaptionsResult {
    pub video: Video,
    pub captions: Vec<CaptionTextSnippet>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VideoCaptionBuckets {
    pub video: Video,
    pub captions: Vec<CaptionTextSnippet>,
}

#[get("/video/caption/search?<text>")]
pub async fn search_video_captions(
    text: &str,
    state: &State<ApiState>,
) -> Json<Option<CaptionSearchResults>> {
    // If the user searches for text with spaces in it, such as "tennis match",
    // then we want to find any row that contains the text "tennis" AND "match".
    // To do this we put a `&` character inbetween every word.
    let search_text = text.replace(" ", " & ");

    let rows = sqlx::query!(
        "
        select
            v.id,
            v.channel_id,
            v.title,
            v.upload_datetime,
            v.views,
            v.length,
            v.thumbnail,
            v.youtube_id,
            CONCAT('https://www.youtube.com/watch?v=', v.youtube_id) as base_url,
            ch.title as channel_title,
            CONCAT('https://www.youtube.com/watch?v=', v.youtube_id, '&t=', GREATEST(ct.start::integer - 2, 0), 's') as url,
            ct.caption_text,
            ct.start
        from caption_timestamps ct
        join videos v on v.id = ct.video_id
        join channels ch on ch.id=v.channel_id
        where to_tsvector('english', caption_text) @@ to_tsquery('english', $1)
        order by v.upload_datetime desc, ct.start",
        search_text,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();

    let mut videos: Vec<VideoCaptionsResult> = vec![];
    let mut temp_video: Video = Video {
        id: 0,
        channel_id: 0,
        channel_title: String::new(),
        title: String::new(),
        url: String::new(),
        captions: String::new(),
        upload_datetime: None,
        views: 0,
        length: 0,
        thumbnail: String::new(),
        youtube_id: String::new(),
    };

    for row in rows {
        // Each row effectively stores all the data for each video already, so
        // we just create the video once
        if temp_video.id != row.id {
            temp_video = Video {
                id: row.id,
                channel_id: row.channel_id,
                channel_title: row.channel_title,
                title: row.title,
                url: row.base_url.unwrap(),
                captions: String::new(),
                upload_datetime: Some(row.upload_datetime),
                views: row.views,
                length: row.length,
                thumbnail: row.thumbnail,
                youtube_id: row.youtube_id,
            };

            let temp_captions: Vec<CaptionTextSnippet> = vec![];

            videos.push(VideoCaptionsResult {
                video: temp_video.clone(),
                captions: temp_captions,
            });
        }

        let temp_caption = CaptionTextSnippet {
            url: row.url.unwrap(),
            caption_text: row.caption_text,
            start: row.start,
        };
        videos.last_mut().unwrap().captions.push(temp_caption);
    }

    Json(Some(CaptionSearchResults {
        success: true,
        videos,
    }))
}

#[get("/video/test")]
pub async fn test_video(_state: &State<ApiState>) -> Json<SuccessFailResponse> {
    // let caption_id_result: Result<i32, Error> = sqlx::query_scalar(
    //     "insert into captions (video_id, raw_text, caption_json) values ($1, $2, $3) returning id",
    // )
    // .bind(video_id)
    // .bind(raw_text)
    // .bind(sqlx::types::Json(&video_captions))
    // .fetch_one(&state.pool)
    // .await;

    Json(SuccessFailResponse { success: true })
}
