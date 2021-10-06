use std::time::{Duration, Instant};
use reqwest::{Url, StatusCode, Client as HTTPClient};
use tokio::runtime::Runtime;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WatchConfig {
    ttl: Duration,
    max_attempts: Option<usize>,

}

#[derive(Debug, Clone, Eq)]
pub struct WatchableResource {
    url: Url,
    last_ping_at: Option<Instant>,
}

impl PartialEq for WatchableResource {

    fn eq(&self, other: &Self) -> bool {
        self.url == other.url &&
        self.last_ping_at.is_some() && other.last_ping_at.is_some() &&
        self.last_ping_at.unwrap() == other.last_ping_at.unwrap()
    }
}


#[derive(Debug)]
pub struct PingEvent {
    pub(crate) resource: WatchableResource,
    pub(crate) request_init: Option<Instant>,
    pub(crate) response_time: Option<Duration>,
    pub(crate) response_code: Option<StatusCode>,
    pub(crate) error: Option<reqwest::Error>,
}


impl WatchableResource {
    pub fn new(url: &str, last_ping_at: Option<Instant>) -> Self
    {
        Self {
            url: Url::parse(url).unwrap(),
            last_ping_at
        }
    }

    pub fn ping(&mut self, client: &HTTPClient, rt: &Runtime) -> PingEvent {
        rt.block_on(
            async {

                let start_time = Instant::now();
                let response = client.get(self.url.clone()).send().await;

                match response {
                    Ok(response) => {
                        // Note: If we want json, we need to enable the json feature of reqwest.
                        
                        // println!("{:#?}", response);
                        //     .json::<HashMap<String, String>>()
                        //     .await?;
                        //     println!("{:#?}", response);

                        self.last_ping_at = Some(start_time);
                        PingEvent::new(
                            self.clone(),
                            Some(start_time),
                            Some(start_time.elapsed()),
                            Some(response.status()),
                            None
                        )
                    },
                    Err(error) => {
                        self.last_ping_at = Some(start_time);
                        PingEvent::new(
                            self.clone(),
                            Some(start_time),
                            Some(start_time.elapsed()),
                            None,
                            Some(error)
                        )
                    }
                }
            }
        )
    }
}


impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs_f64(5.0),
            max_attempts: Some(2)
        }
    }
}


impl PingEvent {
    pub fn new(
        resource: WatchableResource,
        request_init: Option<Instant>,
        response_time: Option<Duration>,
        response_code: Option<StatusCode>,
        error: Option<reqwest::Error>
    ) -> Self {

        Self {
            resource: resource.clone(),
            request_init,
            response_time,
            response_code,
            error
        }
    }

    pub fn new_empty(resource: WatchableResource) -> Self {
        Self::new(resource, None, None, None, None)
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
    rt: Runtime,
    pub(crate) config: WatchConfig
}

impl WatchPool {

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_subjects_and_config(&vec![], WatchConfig::default())
    }

    pub fn default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new()
    }

    pub fn with_subjects(subjects: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_subjects_and_config(subjects, WatchConfig::default())
    }

    pub fn with_subjects_and_config(
        subjects: &[&str],
        config: WatchConfig
    ) -> Result<WatchPool, Box<dyn std::error::Error>> {

        let resources = subjects.iter()
            .map(|s| WatchableResource::new(s, None))
            .collect::<Vec<WatchableResource>>();
        
        let runtime = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()?;
        
        Ok(Self {
            subjects: resources,
            client: HTTPClient::builder().build()?,
            rt: runtime,
            config
        })
    }

    pub fn add_subject(&mut self, subject: &str) -> Result<WatchableResource, Box<dyn std::error::Error>> {
        
        let resource = WatchableResource::new(subject, None);
        self.subjects.push(resource.clone());

        Ok(
            resource    
        )
    }

    pub fn add_subjects(&mut self, subjects: &[&str]) -> Result<Vec<WatchableResource>, Box<dyn std::error::Error>> {
        Ok(
            subjects
            .iter()
            .map(|s| self.add_subject(s).unwrap())
            .collect::<Vec<WatchableResource>>()
        )
    }

    pub fn ping_all(&mut self) -> Vec<PingEvent> {
        
        let mut events = vec![];

        for subject in self.subjects.iter_mut() {
            events.push((*subject).ping(&self.client, &self.rt))
        }
        events
    }
}


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let response = reqwest::get("https://httpbin.org/ip")
//     .await?
//     .json::<HashMap<String, String>>()
//     .await?;
//     println!("{:#?}", response);
//     Ok(())
// }


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_google_watchable_resource() {
        let google_resource = WatchableResource::new("https://www.google.com/", None);

        assert_eq!(google_resource.url.as_str(), "https://www.google.com/");
        assert_eq!(google_resource.last_ping_at, None);
    }

    #[test]
    fn create_google_ping_event_empty() {
        let google_resource = WatchableResource::new("https://www.google.com/", None);
        let ping_event_empty = PingEvent::new_empty(google_resource);
        assert_eq!(ping_event_empty.request_init, None);
        assert_eq!(ping_event_empty.response_code, None);
        assert_eq!(ping_event_empty.response_time, None);
    }

    #[test]
    fn create_dummy_google_ping_event() {
        let google_resource = WatchableResource::new("https://www.google.com/", None);

        let _ping_event = PingEvent::new(
            google_resource,
            Some(Instant::now()),
            Some(Duration::from_secs(2)),
            Some(StatusCode::OK),
            None
        );
    }

    #[test]
    fn create_watcher() {

        let domains = [
            "https://www.google.com/",
            "https://www.amazon.com/",
        ];

        let watcher = WatchPool::with_subjects(&domains);
        assert!(watcher.is_ok());
    }

    #[test]
    fn ping_google_and_amazon() {
        let domains = [
            "https://www.google.com/",
            "https://www.amazon.com/",
            "http://www.thisdomainisdefinitelynottaken.com/"
        ];

        let watcher = WatchPool::with_subjects(&domains);
        assert!(watcher.is_ok());

        let ping_events = watcher.unwrap().ping_all();

        println!("{:#?}", ping_events);
    }

}
