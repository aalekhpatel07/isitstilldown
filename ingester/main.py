import json
import logging
import socket
import os

from kafka import KafkaConsumer

from models import PingEvent

logging.basicConfig()
logging.getLogger(__name__).setLevel(logging.DEBUG)
logger = logging.getLogger(__name__)

brokers = os.getenv('KAFKA_BROKERS') or 'kafka:9092'

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

def convert_ping_event_to_influxdb_line_protocol(ping_event: PingEvent):
    # logger.debug('Before tags.')
    tags = {
        "url": ping_event.resource.url,
        "last_ping_at": ping_event.resource.last_ping_at.as_str() if ping_event.resource.last_ping_at else -1,
        "request_init": ping_event.request_init.as_str() if ping_event.request_init else -1,
        "response_code": ping_event.response_code
    }

    # logger.debug('Before fields.')
    
    fields = {
        "response_time": ping_event.response_time.as_str() if ping_event.response_time else -1
    }
    measurement = "ping_event"
    
    # logger.debug("Before tags, and fields str")

    tags_str = ",".join(map(lambda item: f"{item[0]}={item[1]}", tags.items()))
    fields_str = ",".join(map(lambda item: f"{item[0]}={item[1]}", fields.items()))
    
    result = measurement + "," + tags_str + " " + fields_str
    logger.debug("Influx Line Protocol: %s", result)
    return result


def handle_multiple_write(data_buffer):
    try:
        s = setup_socket()
        data = "\n".join(map(convert_ping_event_to_influxdb_line_protocol, data_buffer))
        data += "\n"
        # logger.debug("DATA: %s", data)
        s.sendall(data.encode())
        logger.info("Number of datapoints sent: %s", len(data_buffer))
        s.close()
    except socket.error as e:
        logger.error("Socket error: %s", e)


def setup_socket():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    host, port = os.getenv('DB_HOST'), os.getenv('DB_PORT')
    logger.debug("Establishing new socket connection to %s:%s", host, port)
    try:
        s.connect((str(host), int(port)))
        logger.debug("Socket connection is fine!")
    except socket.error as e:
        logger.error("Socket connection error: %s", e)
    return s


def main():
    consumer = kafka_consumer_blocking(brokers=brokers)
    data_buffer = []

    for msg in consumer:
        data = PingEvent.from_dict(msg.value)
        data_buffer.append(data)
        if len(data_buffer) % 10 == 0:
            try:
                logger.debug("len(data_buffer): %s", len(data_buffer))
                handle_multiple_write(data_buffer)
                data_buffer = []
            except Exception as e:
                logger.warning("Could not write %s datapoints.", len(data_buffer))
                logger.error("Write error: %s", e)

    if len(data_buffer):
        handle_multiple_write(data_buffer)
        data_buffer = []


if __name__ == '__main__':
    main()
