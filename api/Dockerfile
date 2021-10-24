FROM golang:1.10 AS build
WORKDIR /go/src
COPY go ./go
COPY main.go .

ENV CGO_ENABLED=0
RUN go get -d -v ./...

RUN go build -a -installsuffix cgo -o swagger .

FROM centos:8 AS runtime
RUN yum -y update && yum -y upgrade
COPY --from=build /go/src/swagger ./
EXPOSE 8080/tcp
CMD ["./swagger"]
