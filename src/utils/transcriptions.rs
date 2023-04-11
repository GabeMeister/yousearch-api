use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TranscriptionSnippet {
    pub text: String,
    pub start: i32,
    pub duration: i32,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeHtmlCaptionData {
    pub playerCaptionsTracklistRenderer: YouTubeCaptionTracks,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeCaptionTracks {
    pub captionTracks: Vec<YouTubeCaptionTrack>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeCaptionTrack {
    pub baseUrl: String,
}

pub async fn fetch_transcription(video_id: String) -> Vec<TranscriptionSnippet> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");

    let html = reqwest::get(url).await.unwrap().text().await.unwrap();

    let data: Vec<&str> = html.split("\"captions\":").collect();

    if data.len() > 1 {
        let sub_snippet = data[1].to_string();
        let transcript_sub_snippet: Vec<&str> = sub_snippet.split(",\"videoDetails").collect();
        let transcript_json = transcript_sub_snippet[0];
        let transcript_data: YouTubeHtmlCaptionData =
            serde_json::from_str(transcript_json).unwrap();

        dbg!(transcript_data);
    } else {
        todo!("Havent handled this yet")
    }

    return vec![TranscriptionSnippet {
        text: "Welcome to Fleccas Talks, the best new podcast of all time!".to_string(),
        start: 5,
        duration: 3,
    }];
}

// https://github.com/jdepoix/youtube-transcript-api

// https://github.com/danielcliu/youtube-channel-transcript-api

// Step 1: Fetch html at 'https://www.youtube.com/watch?v={video_id}'
// 2) html.split('"captions":')

// def _extract_captions_json(self, html, video_id):
//         splitted_html = html.split('"captions":')

//         if len(splitted_html) <= 1:
//             if 'class="g-recaptcha"' in html:
//                 raise TooManyRequests(video_id)
//             if '"playabilityStatus":' not in html:
//                 raise VideoUnavailable(video_id)

//             raise TranscriptsDisabled(video_id)

//         captions_json = json.loads(
//             splitted_html[1].split(',"videoDetails')[0].replace('\n', '')
//         ).get('playerCaptionsTracklistRenderer')
//         if captions_json is None:
//             raise TranscriptsDisabled(video_id)

//         if 'captionTracks' not in captions_json:
//             raise NoTranscriptAvailable(video_id)

//         return captions_json
