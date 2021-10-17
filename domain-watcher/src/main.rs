mod kafka;

use std::time::{Duration, SystemTime};
use reqwest::{Url, StatusCode, Client as HTTPClient, Response};
use futures::{stream, StreamExt};
use serde::{Serialize, Deserialize};
use serde::ser::{SerializeStruct};
use log::{info, debug, error};
use serde_json::{Serializer, to_vec};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WatchConfig {
    ttl: Duration,
    max_attempts: Option<usize>,
    concurrent_requests: usize
}

#[derive(Debug, Clone, Eq)]
pub struct WatchableResource {
    url: Url,
    last_ping_at: Option<SystemTime>,
    active: bool
}


impl Serialize for WatchableResource {
    fn serialize<S: serde::Serializer >(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("WatchableResource", 3)?;
        s.serialize_field("url", &self.url.as_str())?;
        s.serialize_field("last_ping_at", &self.last_ping_at)?;
        s.serialize_field("active", &self.active)?;
        s.end()
    }
}


// impl<'de> Deserialize<'de> for WatchableResource {
//     fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
//         deserializer.deserialize_bool(visitor: V)
//     }
// }


impl PartialEq for WatchableResource {

    fn eq(&self, other: &Self) -> bool {
        self.url == other.url &&
        self.active == other.active &&
        self.last_ping_at.is_some() && other.last_ping_at.is_some() &&
        self.last_ping_at.unwrap() == other.last_ping_at.unwrap()
    }
}


#[derive(Debug, Serialize, Clone)]
pub struct PingEvent {
    pub(crate) resource: WatchableResource,
    pub(crate) request_init: Option<SystemTime>,
    pub(crate) response_time: Option<Duration>,
    pub(crate) response_code: Option<u16>,
    // pub(crate) error: Option<reqwest::Error>,
}


impl WatchableResource {
    pub fn new<S: AsRef<str>>(url: S, last_ping_at: Option<SystemTime>, active: bool) -> Self
    {
        Self {
            url: Url::parse(url.as_ref()).unwrap(),
            last_ping_at,
            active
        }
    }
}


impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs_f64(5.0),
            max_attempts: Some(2),
            concurrent_requests: 16
        }
    }
}


impl PingEvent {
    pub fn new(
        resource: WatchableResource,
        request_init: Option<SystemTime>,
        response_time: Option<Duration>,
        response_code: Option<u16>,
    ) -> Self {

        Self {
            resource,
            request_init,
            response_time,
            response_code,
        }
    }

    pub fn new_empty(resource: WatchableResource) -> Self {
        Self::new(resource, None, None, None)
    }

    pub fn now<S: AsRef<str>>(url: S) -> Self {
        let curr_time = Some(SystemTime::now());
        Self::new(
            WatchableResource::new(url, curr_time, true),
            curr_time,
            None,
            None
        )
    }
}


impl PartialEq for PingEvent {
    fn eq(&self, other: &Self) -> bool {

        // TODO: Figure out a better way to go about this.
        self.resource == other.resource &&

        self.request_init.is_some() && other.request_init.is_some() &&
        self.request_init.unwrap() == other.request_init.unwrap() &&

        self.response_time.is_some() && other.response_time.is_some() &&
        self.response_time.unwrap() == other.response_time.unwrap() &&

        self.response_code.is_some() && other.response_code.is_some() &&
        self.response_code.unwrap() == other.response_code.unwrap()
    }
}

impl Eq for PingEvent {}


#[derive(Debug)]
pub struct WatchPool {
    pub subjects: Vec<WatchableResource>,
    pub client: HTTPClient,
    pub(crate) config: WatchConfig
}

impl WatchPool {

    pub fn new<S: AsRef<str>>() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_subjects_and_config(&Vec::<S>::new(), WatchConfig::default())
    }

    pub fn default<S: AsRef<str>>() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new::<S>()
    }

    pub fn with_subjects<S: AsRef<str>>(subjects: &[S]) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_subjects_and_config(subjects, WatchConfig::default())
    }

    pub fn with_subjects_and_config<S: AsRef<str>>(
        subjects: &[S],
        config: WatchConfig
    ) -> Result<WatchPool, Box<dyn std::error::Error>> {

        let resources = subjects.iter()
            .map(|s| WatchableResource::new(s, None, true))
            .collect::<Vec<WatchableResource>>();
        
        Ok(Self {
            subjects: resources,
            client: HTTPClient::builder().build()?,
            config
        })
    }

    pub fn add_subject<S: AsRef<str>>(&mut self, subject: S) -> Result<WatchableResource, Box<dyn std::error::Error>> {
        
        let resource = WatchableResource::new(subject, None, true);
        self.subjects.push(resource.clone());

        Ok(
            resource    
        )
    }

    pub fn add_subjects<S: AsRef<str>>(&mut self, subjects: &[S]) -> Result<Vec<WatchableResource>, Box<dyn std::error::Error>> {
        Ok(
            subjects
            .iter()
            .map(|s| self.add_subject(s).unwrap())
            .collect::<Vec<WatchableResource>>()
        )
    }
}


const CONCURRENT_REQUESTS: usize = 16;
const WAIT_SECONDS_BEFORE_NEXT_BATCH: u64 = 5;


#[tokio::main]
async fn main() -> std::io::Result<()> {

    env_logger::init();
    
    let brokers: &str = &std::env::var("DOMAIN_WATCHER_KAFKA_BROKERS").unwrap_or("kafka:9092".to_string());
	let kproducer = kafka::create_producer(brokers);

    let urls = std::fs::read_to_string(std::env::var("DOMAIN_WATCHER_DOMAIN_LIST").unwrap_or("domains.txt".to_string()))?;

    let urls = urls
        .split_whitespace()
        .map(|x| {
            format!("https://{}", x)
        })
        .collect::<Vec<String>>();
    
    let client = reqwest::Client::builder().build().unwrap();

    let mut watch_pool = WatchPool::with_subjects_and_config(
        &urls,
        WatchConfig::default()
    ).unwrap();



    loop {

        // TODO: Add ability to add/remove a domain from the watch pool.
        // let (socket, _) = listener.accept().await?;
        // tokio::spawn(async move {
        //     watch_pool.add_subject().unwrap()
        // }).await?;

        stream::iter(&mut watch_pool.subjects)
            // .map(|url| url.url.as_str())
            .map(|url| {
                let client = &client;

                async move {

                    let start_time: SystemTime = SystemTime::now();
                    url.last_ping_at = Some(start_time);

                    let resp: Response = client.get(url.url.as_str()).send().await?;
                    let time_taken: Duration = start_time.elapsed().unwrap();

                    Ok((resp, start_time, time_taken, url))
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS.min(urls.len()))
            .for_each(
                |pr: std::result::Result<(reqwest::Response, std::time::SystemTime, std::time::Duration, &mut WatchableResource), reqwest::Error>| async {
                    
                    match pr {
                        Ok((resp, start_time, time_taken, url)) => {

                            info!("{:#?}", resp);

                            let mut buf = Vec::new();
                            let event = PingEvent::new(
                                url.clone(),
                                Some(start_time),
                                Some(time_taken),
                                Some(resp.status().as_u16())
                            );
                            
                            event.serialize(&mut Serializer::new(&mut buf)).unwrap();

                            kproducer.produce(
                                bytes::BytesMut::from(buf.as_slice()),
                                "pingevents"
                            ).await;
                        },
                        Err(e) => {
                            error!("{:#?}", e);
                            let url = e.url().expect("Url should exist.");
                            let full_url = format!(
                                "{}://{}{}",
                                url.scheme(),
                                url.host().expect("Host should exist"),
                                url.path()
                            );

                            let event = PingEvent::now(full_url);
                            let mut buf = Vec::new();
                            event.serialize(&mut Serializer::new(&mut buf)).unwrap();
                            kproducer.produce(
                                bytes::BytesMut::from(buf.as_slice()),
                                "pingevents"
                            ).await;
                        }
                    }
            })
            .await;
            
            tokio::time::sleep(Duration::from_secs(WAIT_SECONDS_BEFORE_NEXT_BATCH)).await;
    }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
// }
