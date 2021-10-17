import json
import logging
import os

from influxdb_client import InfluxDBClient, Point
from influxdb_client.client.write_api import SYNCHRONOUS

from kafka import KafkaConsumer

from models import PingEvent

logging.basicConfig()
logging.getLogger(__name__).setLevel(logging.DEBUG)
logger = logging.getLogger(__name__)


bucket = os.getenv('INFLUX_BUCKET') or 'domain-watcher'
host = os.getenv('INFLUX_HOST') or 'influx'
port = os.getenv('INFLUX_PORT') or 8086
token = os.getenv('INFLUX_TOKEN')
org = os.getenv('INFLUX_ORG') or 'is-it-still-down?'

brokers = os.getenv('KAFKA_BROKERS') or 'kafka:9092'


client = InfluxDBClient(url=f'http://{host}:{port}', token=token, org=org)
# query_api = client.query_api()



def kafka_consumer_blocking(brokers=None):
    if brokers is None:
        brokers = "kafka:9092"
    while True:
        try:
            consumer = KafkaConsumer('pingevents', value_deserializer=json.loads, bootstrap_servers=brokers)
            logger.debug('Connected to broker.')
            break
        except Exception as e:
            logger.error('Could not connect to broker %s', e)
    return consumer

def create_point(ping_event: PingEvent):
    data_point = (
        Point("domain-ping")
        .tag("url", ping_event.resource.url)
        .tag("last_ping_at", int(ping_event.resource.last_ping_at.as_str() if ping_event.resource.last_ping_at else -1))
        .tag("request_init", int(ping_event.request_init.as_str() if ping_event.request_init else -1))
        .tag("response_code", ping_event.response_code)
        .field("response_time", int(ping_event.response_time.as_str() if ping_event.response_time else -1))
    )
    return data_point


def main():
    consumer = kafka_consumer_blocking(brokers=brokers)
    write_api = client.write_api(write_options=SYNCHRONOUS)

    # p = Point("my_measurement").tag("location", "Prague").field("temperature", 25.3)
    # write_api.write(bucket=bucket, record=p)
    # data_buffer = []

    for msg in consumer:
        data = PingEvent.from_dict(msg.value)
        point = create_point(data)
        write_api.write(bucket=bucket, record=[point])
        logger.debug(f'Wrote a point: {point}')
    #     data_buffer.append(data.to_dict())
    #     if len(data_buffer) % 10 == 0:
    #         try:
    #             # s.write_points(data_buffer)
    #             logger.info("Written %2d points to influx.", len(data_buffer))
    #             data_buffer = []
    #         except Exception as e:
    #             logger.warn("Could not write %s datapoints to influx.", len(data_buffer))


if __name__ == '__main__':
    main()

