// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Command tpt-cloud-core is the entrypoint for the Cloud Core microservice.
// It wires the WebSocket gateway, MQTT bridge, and HTTP health endpoints.
package main

import (
	"log"
	"net/http"
	"os"

	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/gateway"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/mqttbridge"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/transport"
)

// hubSink adapts the WebSocket hub to the mqttbridge.TelemetrySink interface.
type hubSink struct{ hub *gateway.Hub }

func (s hubSink) Broadcast(env protocol.Envelope) error {
	s.hub.Broadcast(env)
	return nil
}

func main() {
	broker := os.Getenv("TPT_MQTT_BROKER")
	if broker == "" {
		broker = "tcp://localhost:1883"
	}

	var bridge *mqttbridge.Bridge

	mqttClient, err := transport.New(transport.Config{
		Broker:   broker,
		ClientID: "cloud-core",
	}, func(topic string, payload []byte) {
		bridge.OnMessage(topic, payload)
	})
	if err != nil {
		log.Printf("mqtt init: %v (continuing without broker)", err)
	}

	bridge = mqttbridge.NewBridge(mqttClient.Publish, nil)
	hub := gateway.NewHub(bridge)
	bridge.SetSink(hubSink{hub: hub})
	go hub.Run()

	http.HandleFunc("/ws", hub.ServeWS)
	http.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	http.Handle("/metrics", promhttp.Handler())

	addr := envOr("TPT_HTTP_ADDR", ":8080")
	log.Printf("cloud-core listening on %s (mqtt %s)", addr, broker)
	if err := http.ListenAndServe(addr, nil); err != nil {
		log.Fatal(err)
	}
}

func envOr(k, def string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return def
}
