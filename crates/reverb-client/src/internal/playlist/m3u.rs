use std::{collections::HashMap, fs::File, io::BufRead, time::Duration};

use reverb_core::failure::failure::{Failure, FailureType};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{external::{self, external::{ExternalSong, ExternalSongTrait, ExternalType}}, internal::{playlist::playlist::Playlist, song::{Song, SongInfo}}};

static REVERB_PLAYLIST_INFO: &str = "#REVERBPLAYLISTINFO:";
static REVERB_SONG_INFO: &str = "#REVERBSONGINFO:";
static EXTINF: &str = "#EXTINF:";
static EXT_M3U: &str = "#EXTM3U";
static NAME: &str = "name";
static EXTERNAL_TYPE: &str = "external_type";
static ADDITIONAL_PROPERTIES: &str = "additional_properties";
static EQUALS: &str = "=";
static COMMA: &str = ",";
static SPACE: &str = " ";

/// Represents a song in an M3U playlist. Used for serialization and deserialization of M3U playlists.
pub struct M3USongInfo {
    pub duration: Duration,
    pub title: String,
    pub artists: Vec<String>,
    pub uri: String,
    pub external_type: ExternalType,
    pub additional_properties: Option<HashMap<String, AdditionalProperty>>,
}

pub enum SongCreationInfo {
    FromM3U(M3USongInfo),
    FromURI(String),
}

pub struct M3UPlaylist {
    pub name: String,
    pub external_type: Option<ExternalType>,
    pub songs: Vec<SongCreationInfo>,
}


#[derive(Serialize, Deserialize, Debug)]
pub enum  AdditionalProperty {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<AdditionalProperty>),
    Object(HashMap<String, AdditionalProperty>),
}

pub(super) fn playlist_to_m3u_string(playlist: &Playlist) -> Result<String, Failure> {
    // mandatory tag for extm3u files
    let mut m3u8 = format!("{EXT_M3U}\n");
    m3u8.push_str(&format!("{REVERB_PLAYLIST_INFO}{NAME}{EQUALS}{}{COMMA}{SPACE}{EXTERNAL_TYPE}{EQUALS}{}\n", playlist.name, playlist.external_type.as_ref().map_or("None".to_string(), |et| et.to_string())));
    for song in &playlist.songs {
        let m3u_song_info = song.to_m3u_song_info()?;
        let duration = m3u_song_info.duration.as_secs() as i64;
        let title = &m3u_song_info.title;
        let artists = m3u_song_info.artists.join(", ");
        let uri = &m3u_song_info.uri;
        let external_type = &m3u_song_info.external_type;
        let additional_properties = &m3u_song_info.additional_properties;

        // add the EXTINF tag with the duration and title
        m3u8.push_str(&format!("{EXTINF}:{}, {} - {}\n", duration, artists, title));
        // add the external type of the song and additional properties
        m3u8.push_str(&format!("{REVERB_SONG_INFO}{EXTERNAL_TYPE}{EQUALS}{}{COMMA}{SPACE}{ADDITIONAL_PROPERTIES}{EQUALS}{}\n", 
            external_type, 
            serde_json::to_string(&additional_properties).map_err(|e| Failure::from((e.into(), &*format!("failed to serialize additional properties for {:?}", song), FailureType::Warning)))?));
        // add the URI of the song
        m3u8.push_str(&format!("{}\n", uri));
    }
    Ok(m3u8)
}

pub(super) fn file_to_m3u(file: File) -> Result<M3UPlaylist, Failure> {
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();
    let mut reverb_playlist = false;
    let mut type_determined = false;

    // read to the first line that referes to a song, gathering info along the way!
    while !type_determined && {line.clear(); reader.read_line(&mut line).is_ok()} {
        if line.trim().is_empty() || line.starts_with(EXT_M3U) { // ignore empty lines and the extm3u tag
            continue;
        } else if line.starts_with(REVERB_PLAYLIST_INFO) { // reerb playlist
            reverb_playlist = true;
            type_determined = true;
        } else if line.starts_with(EXTINF) { // standard m3u playlist
            type_determined = true;
        } else if line.starts_with('#') { // other comment, ignore
            // ignore other comments
            continue;
        } else {
            type_determined = true; // assume it's a standard m3u playlist
        }
    }
    if !type_determined {
        return Err(Failure::from((anyhow!("Failed to determine M3U playlist type the playlist is likely malformed"), reverb_core::failure::failure::FailureType::Warning)));
    }

    let external_type;
    let name;
    let mut m3u_song_infos: Vec<SongCreationInfo> = Vec::new();

    if reverb_playlist {
        // parse playlist info
        // this is definatly a bad way to do it it ccurrently relies on the specfic order of the feilds, need better parser but not right now... TODO: this
        let name_start;
        let name_end;
        let external_type_start;
        let external_type_end = line.len() -1;
        match line.find(&format!("{NAME}{EQUALS}")) {
            Some(index) => name_start = index + NAME.len() + EQUALS.len(),
            None => {
                return Err(Failure::from((
                    anyhow!("malformed reverb playlist"),
                    FailureType::Warning,
                )));
            }
        };
        match line.find(&format!("{EXTERNAL_TYPE}{EQUALS}")) {
            Some(index) => {
                name_end = index - 2;
                external_type_start = index + EXTERNAL_TYPE.len() + EQUALS.len();
            },
            None => {
                return Err(Failure::from((
                    anyhow!("malformed reverb playlist"),
                    FailureType::Warning,
                )));
            }
        }
        name = &line[name_start..name_end];
        let external_type_str = &line[external_type_start..external_type_end];
        if external_type_str.trim() == "None" {
            external_type = None;
        } else {
            external_type = Some(ExternalType::get_from_str(external_type_str.trim())?);
        }

        // parse song info
        let mut current_line = String::new();
        while {
            current_line.clear();
            match reader.read_line(&mut current_line) {
                Ok(0) => false, // EOF
                Ok(_) => true,
                Err(e) => {
                    return Err(Failure::from((e.into(), FailureType::Warning)));
                }
            }
        } {
            if !current_line.starts_with(EXTINF) {
                continue; // keep reading till start of next song
            }

            let extinf = current_line.trim_start_matches(EXTINF).trim();
            let mut parts = extinf.splitn(2, ',');
            let duration = parts
                .next()
                .and_then(|v| v.trim().parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or_default();
            let title_and_artists = parts.next().unwrap_or_default().trim();
            let (artists, title) = title_and_artists
                .rsplit_once(" - ")
                .map(|(artists, title)| {
                    (
                        artists
                            .split(", ")
                            .filter(|artist| !artist.is_empty())
                            .map(|artist| artist.to_string())
                            .collect::<Vec<_>>(),
                        title.to_string(),
                    )
                })
                .unwrap_or_default();

            let mut song_info_line = String::new();
            let mut uri_line = String::from('#');
            while !song_info_line.starts_with(REVERB_SONG_INFO) {
                song_info_line.clear();
                if !reader.read_line(&mut song_info_line).is_ok() {
                    break;
                }
            }
            while uri_line.starts_with('#') {
                uri_line.clear();
                if !reader.read_line(&mut uri_line).is_ok() {
                    break;
                }
            }
            if song_info_line.is_empty() || uri_line.is_empty() {
                break;
            } else {
                let external_type;
                let additional_properties: HashMap<String, AdditionalProperty>;

                if let Some(external_type_start) = song_info_line.find(&format!("{EXTERNAL_TYPE}{EQUALS}")) {
                    let external_type_start = external_type_start + EXTERNAL_TYPE.len() + EQUALS.len();
                    let additional_properties_start = song_info_line
                        .find(&format!("{ADDITIONAL_PROPERTIES}{EQUALS}"))
                        .unwrap_or(song_info_line.len());
                    let external_type_str = song_info_line[external_type_start..additional_properties_start]
                        .trim()
                        .trim_end_matches(COMMA)
                        .trim();
                    external_type = ExternalType::get_from_str(external_type_str)?;
                } else {
                    return Err(Failure::from((
                        anyhow!("malformed reverb playlist"),
                        FailureType::Warning,
                    )));
                }

                if let Some(additional_properties_start) = song_info_line.find(&format!("{ADDITIONAL_PROPERTIES}{EQUALS}")) {
                    let additional_properties_start = additional_properties_start + ADDITIONAL_PROPERTIES.len() + EQUALS.len();
                    additional_properties = serde_json::from_str(song_info_line[additional_properties_start..].trim()).unwrap_or_default();
                } else {
                    return Err(Failure::from((
                        anyhow!("malformed reverb playlist"),
                        FailureType::Warning,
                    )));
                }


                m3u_song_infos.push(SongCreationInfo::FromM3U(M3USongInfo {
                    duration,
                    title,
                    artists,
                    uri: uri_line.trim().to_string(),
                    external_type,
                    additional_properties: if additional_properties.is_empty() {
                        None
                    } else {
                        Some(additional_properties)
                    },
                }));
            }
        }
    } else {
        todo!()
    }

    Ok(M3UPlaylist {
        name: String::from(name),
        external_type: external_type,
        songs: m3u_song_infos,
    })
}

pub(super) fn m3u_to_playlist(m3u_playlist: M3UPlaylist) -> Result<Playlist, Failure> {
    let songs = m3u_playlist.songs.iter().map(|song_creation_info| -> Result<Song, Failure> {
        match song_creation_info {
            SongCreationInfo::FromM3U(m3u_song_info) => Ok(Song {
                song_external: ExternalSong::from_m3u_song_info(m3u_song_info)?,
                info: SongInfo {
                    title: m3u_song_info.title.clone(),
                    artists: m3u_song_info.artists.clone(),
                },
            }),
            SongCreationInfo::FromURI(uri) => Song::new(uri),
        }
    }).collect::<Result<Vec<_>, _>>()?;
    let playlist = Playlist {
        name: m3u_playlist.name,
        external_type: m3u_playlist.external_type,
        songs,
    };
    Ok(playlist)
}