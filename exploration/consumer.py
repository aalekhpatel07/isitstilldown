from kafka import KafkaConsumer
import msgpack


if __name__ == '__main__':
    consumer = KafkaConsumer('pingevents', value_deserializer=msgpack.loads)
    for msg in consumer:
        print(msg.value)
