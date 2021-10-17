FROM python:3

RUN apt-get update && apt-get upgrade
RUN apt-get install -y vim iputils-ping

WORKDIR /app

# copy all the files to the container
COPY . .

# install dependencies
RUN pip install --no-cache-dir -r requirements.txt

# run program
CMD ["python", "./main.py"]
