use std::sync::{atomic::AtomicU8, Arc};

#[derive(Clone)]
pub struct Session {
    session_data: Arc<tokio::sync::RwLock<SessionData>>,
    connection_count: Arc<std::sync::Mutex<u8>>,
}

struct SessionData {
    session_id: String,
    username: String,
    password: String,
}

impl Session {
    pub async fn new(username: String, password: String) -> anyhow::Result<Self> {
        let session_id = get_session_id(&username, &password).await?;
        Ok(Session {
            session_data: Arc::new(tokio::sync::RwLock::new(SessionData {
                session_id,
                username,
                password,
            })),
            connection_count: Arc::default(),
        })
    }

    pub async fn query(&self, callsign: &str) -> anyhow::Result<Option<SearchData>> {
        let mut saved_id: Option<String> = None;
        loop {
            let client =
                reqwest::Client::new().get("https://www.hamqth.com/xml_latlong.php".to_owned());

            let session_id = self.session_data.read().await.session_id.clone();

            loop {
                {
                    let mut cc = self.connection_count.lock().unwrap();
                    if *cc < 16 {
                        *cc += 1;
                        break;
                    }
                }
                tokio::task::yield_now().await;
            }

            let client = client.query(&[
                ("id", session_id.as_str()),
                ("callsign", callsign),
                ("prg", "rust-dashboard"),
            ]);

            let body = client.send().await?.text().await?;

            *self.connection_count.lock().unwrap() -= 1;

            let response: HamQTH = quick_xml::de::from_str(&body)?;

            match response.response {
                ResponseTypes::Session(session_response) => {
                    match session_response.error.as_deref() {
                        Some("Session does not exist or expired") => {
                            // check to make sure we didn't already get an error with the same session_id
                            if let Some(ref si) = saved_id {
                                if si == &session_id {
                                    // got another error with same id, something else must be wrong
                                    anyhow::bail!(
                                        "HamQTH API error while querying: {:?}",
                                        session_response.error
                                    );
                                }
                            }

                            // check that we still need to request again with a read lock for efficiency
                            let current_data = self.session_data.read().await;
                            if session_id != current_data.session_id {
                                // reauthenticated in another thread, request again
                                saved_id = Some(current_data.session_id.clone());
                                continue;
                            }

                            // get write lock
                            let mut session_data = self.session_data.write().await;

                            // validate that another thread hasn't reauthenticated
                            if session_data.session_id.as_str() == session_id.as_str() {
                                // reauthenticate here, then request again
                                let new_session_id =
                                    get_session_id(&session_data.username, &session_data.password)
                                        .await?;
                                saved_id = Some(new_session_id.clone());
                                session_data.session_id = new_session_id;
                            } else {
                                // reauthenticated in another thread, request again
                                saved_id = Some(session_data.session_id.clone());
                            }
                        }
                        Some("Callsign not found") => {
                            return Ok(None);
                        }
                        Some(e) => anyhow::bail!("HamQTH unrecognized error: {}", e),
                        None => anyhow::bail!("HamQTH empty error"),
                    }
                }
                ResponseTypes::Search(ans) => {
                    return Ok(Some(ans));
                }
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct HamQTH {
    #[serde(rename = "$value")]
    response: ResponseTypes,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResponseTypes {
    Session(SessionResponse),
    Search(SearchData),
}

#[derive(Debug, serde::Deserialize)]
struct SessionResponse {
    session_id: Option<String>,
    error: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchData {
    callsign: String,
    pub latitude: f32,
    pub longitude: f32,
}

async fn get_session_id(username: &str, password: &str) -> anyhow::Result<String> {
    let body = reqwest::Client::new()
        .get("https://www.hamqth.com/xml.php".to_owned())
        .query(&[("u", &username), ("p", &password)])
        .send()
        .await?
        .text()
        .await?;

    let response: HamQTH = quick_xml::de::from_str(&body)?;
    let ResponseTypes::Session(session_data) = response.response else {
        anyhow::bail!("received an invalid response from HamQTH ({})", body);
    };

    match session_data.session_id {
        Some(session_id) => Ok(session_id),
        None => anyhow::bail!(
            "HamQTH API error while authenticating: {:?}",
            session_data.error
        ),
    }
}
