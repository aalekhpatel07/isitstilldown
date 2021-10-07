// MIT License
//
// Copyright (c) 2019 Ankur Srivastava
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// Apart from some minor modifications to refactor it to my use case,
/// this kafka module is a courtesy of [Kafka-rust-example](https://github.com/ansrinivas/kafka-rust-example.git).

mod consumer;
mod producer;
pub use consumer::KafkaConsumer;
pub use producer::KafkaProducer;

use rdkafka::{
	config::{ClientConfig, RDKafkaLogLevel},
	consumer::stream_consumer::StreamConsumer,
};

use prost::bytes::BytesMut;
use uuid::Uuid;

use std::sync::Arc;


use std::env;
use log::info;
use serde::Deserialize;

const DEFAULT_CONFIG_ENV_KEY: &str = "DOMAIN_WATCHER_CONFIG_PATH";
const CONFIG_PREFIX: &str = "DOMAIN_WATCHER_";

struct ConfigFn {}

#[allow(dead_code)]
impl ConfigFn {
	fn fn_false() -> bool {
		false
	}
	fn fn_true() -> bool {
		true
	}
	fn fn_default_host() -> String {
		"0.0.0.0".into()
	}
	fn fn_default_port() -> String {
		"8080".into()
	}
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
	/// Run the app in debug mode
	#[serde(default = "ConfigFn::fn_true")]
	pub debug: bool,

	/// Set the address to bind the webserver on
	/// defaults to 0.0.0.0:8080
	#[serde(default = "ConfigFn::fn_default_host")]
	pub host: String,

	/// Default port, soon deprecated.
	#[serde(default = "ConfigFn::fn_default_port")]
	pub port: String,

	/// Kafka topic on which we want to publish the data.
	pub kafka_topic: String,

	/// Kafka brokers to connect to.
	pub kafka_brokers: String,

	/// Kafka username for sasl authentication.
	pub kafka_username: Option<String>,

	/// Kafka password for sasl authentication.
	pub kafka_password: Option<String>,

	/// Kafka ca-cert path for sasl authentication.
	pub kafka_ca_cert_path: Option<String>,

	/// Postgres database url
	pub postgres_database_url: Option<String>,

	/// Postgres path to cert.
	pub postgres_cert_path: Option<String>,
}

impl Config {
	// Create a new Config instance by reading from
	// environment variables
	pub fn new() -> Config {
		// Check if there is an environment variable DEVICE_REGISTRY_CONFIG_PATH
		// then read it from there else fallback to .env
		let filename = match env::var(DEFAULT_CONFIG_ENV_KEY) {
			Ok(filepath) => filepath,
			Err(_) => ".env".into(),
		};
		info!("Trying to read the config file from [{}]", &filename);

		dotenv::from_filename(&filename).ok();
		match envy::prefixed(CONFIG_PREFIX).from_env::<Config>() {
			Ok(config) => config,
			Err(e) => panic!("Config file being read: {}. And error {:?}", &filename, e),
		}
	}
}


/// Create a consumer based on the given configuration.
///
/// In case certificate path etc is provided then a sasl enabled client
/// is created else a normal client.
pub fn create_consumer(conf: Arc<Config>) -> KafkaConsumer {
	let is_tls = conf.kafka_ca_cert_path.is_some()
		&& conf.kafka_password.is_some()
		&& conf.kafka_username.is_some();

	if is_tls {
		info!("TLS is enabled. Will try to create a secure client");
		let username = conf
			.kafka_username
			.as_deref()
			.expect("Kafka username is required.");
		let password = conf
			.kafka_password
			.as_deref()
			.expect("Kafka password is required.");
		let ca_path = conf
			.kafka_ca_cert_path
			.as_deref()
			.expect("Kafka ca certificate is required.");
		let consumer: StreamConsumer = ClientConfig::new()
			.set("group.id", "some-random-id")
			.set("bootstrap.servers", &conf.kafka_brokers)
			.set("enable.partition.eof", "false")
			.set("session.timeout.ms", "6000")
			.set("enable.auto.commit", "true")
			.set("sasl.mechanisms", "PLAIN")
			.set("security.protocol", "SASL_SSL")
			.set("sasl.username", username)
			.set("sasl.password", password)
			.set("ssl.ca.location", ca_path)
			.set_log_level(RDKafkaLogLevel::Debug)
			.create()
			.expect("Consumer creation failed");
		return KafkaConsumer::new_with_consumer(consumer, &[&conf.kafka_topic]);
	}

	let group_id = Uuid::new_v4();
	KafkaConsumer::new(
		&conf.kafka_brokers,
		&group_id.to_string(),
		&[&conf.kafka_topic],
	)
}

/// Create a producer based on the given configuration.
///
/// In case certificate path etc is provided then a sasl enabled client
/// is created else a normal client.
pub fn create_producer(conf: Arc<Config>) -> KafkaProducer {
	let is_tls = conf.kafka_ca_cert_path.is_some()
		&& conf.kafka_password.is_some()
		&& conf.kafka_username.is_some();

	if is_tls {
		let username = conf
			.kafka_username
			.as_deref()
			.expect("Kafka username is required.");
		let password = conf
			.kafka_password
			.as_deref()
			.expect("Kafka password is required.");
		let ca_path = conf
			.kafka_ca_cert_path
			.as_deref()
			.expect("Kafka ca certificate is required.");
		let producer = ClientConfig::new()
			.set("bootstrap.servers", &conf.kafka_brokers)
			.set("message.timeout.ms", "10000")
			.set("sasl.mechanisms", "PLAIN")
			.set("security.protocol", "SASL_SSL")
			.set("sasl.username", username)
			.set("sasl.password", password)
			.set("ssl.ca.location", ca_path)
			.create()
			.expect("Producer creation error");
		return KafkaProducer::new_with_producer(producer);
	}

	KafkaProducer::new(&conf.kafka_brokers)
}
