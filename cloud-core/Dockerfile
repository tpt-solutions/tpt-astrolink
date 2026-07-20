# syntax=docker/dockerfile:1
# TPT Cloud Core (Go)
FROM golang:1.23 AS builder
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /out/cloud-core ./cmd/cloud-core

FROM gcr.io/distroless/static-debian12
WORKDIR /app
COPY --from=builder /out/cloud-core /app/cloud-core
EXPOSE 8080
ENTRYPOINT ["/app/cloud-core"]
