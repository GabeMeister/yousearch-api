use html_entities::decode_html_entities;
use rocket::serde::{Deserialize, Serialize};
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Serialize)]
pub struct CaptionSnippet {
    pub text: String,
    pub start: i32,
    pub duration: i32,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeHtmlCaptionData {
    #[serde(rename = "playerCaptionsTracklistRenderer")]
    pub player_captions_tracklist_renderer: YouTubeCaptionTracks,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeCaptionTracks {
    #[serde(rename = "captionTracks")]
    pub caption_tracks: Vec<YouTubeCaptionTrack>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeCaptionTrack {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct YouTubeCaptionTextSnippet {
    pub text: String,
    pub start: f32,
    pub duration: f32,
}

pub async fn fetch_captions(video_id: String) -> Vec<CaptionSnippet> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");

    let html = reqwest::get(url).await.unwrap().text().await.unwrap();

    let data: Vec<&str> = html.split("\"captions\":").collect();

    if data.len() > 1 {
        let sub_snippet = data[1].to_string();
        let transcript_sub_snippet: Vec<&str> = sub_snippet.split(",\"videoDetails").collect();
        let transcript_json = transcript_sub_snippet[0];
        let transcript_data: YouTubeHtmlCaptionData =
            serde_json::from_str(transcript_json).unwrap();

        let transcript_url = transcript_data
            .player_captions_tracklist_renderer
            .caption_tracks[1]
            .base_url
            .clone();

        let data = reqwest::get(&transcript_url)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        // dbg!(&data);

        let reader = EventReader::new(BufReader::new(data.as_bytes()));

        let mut captions_list: Vec<YouTubeCaptionTextSnippet> = vec![];
        let mut temp_caption = YouTubeCaptionTextSnippet {
            text: String::new(),
            start: 0.0,
            duration: 0.0,
        };

        for event in reader {
            match event {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    if name.local_name == "text" {
                        for attr in attributes {
                            if attr.name.to_string() == "start".to_string() {
                                temp_caption.start = attr.value.parse::<f32>().unwrap();
                            } else if attr.name.to_string() == "dur".to_string() {
                                temp_caption.duration = attr.value.parse::<f32>().unwrap();
                            }
                        }
                    }
                }
                Ok(XmlEvent::EndElement { name, .. }) => {
                    if name.local_name == "text" {
                        captions_list.push(temp_caption);
                        temp_caption = YouTubeCaptionTextSnippet {
                            text: String::new(),
                            start: 0.0,
                            duration: 0.0,
                        };
                    }
                }
                Ok(XmlEvent::Characters(text)) => {
                    temp_caption.text = decode_html_entities(&text.replace("\n", ""))
                        .unwrap()
                        .to_owned();
                }
                Err(e) => {
                    println!("error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }

        dbg!(captions_list);
    } else {
        todo!("Havent handled this yet");
    }

    return vec![CaptionSnippet {
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
