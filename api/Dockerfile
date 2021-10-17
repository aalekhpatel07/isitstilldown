FROM python:3.9.7
RUN apt-get update && apt-get upgrade
RUN apt-get install -y vim iputils-ping

WORKDIR /app

COPY . .

RUN pip install -r requirements.txt

CMD ["python", "main.py"]
