# syntax=docker/dockerfile:1
# TPT Relay (Go)
FROM golang:1.23 AS builder
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /out/relay ./cmd/relay

FROM gcr.io/distroless/static-debian12
WORKDIR /app
COPY --from=builder /out/relay /app/relay
ENTRYPOINT ["/app/relay"]
