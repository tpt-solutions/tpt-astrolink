// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package transport provides the Cloud Core MQTT client (paho) that connects
// to the broker, subscribes to edge telemetry/events, and publishes commands.
// Topic layout: docs/protocols/mqtt-contract.md.
package transport

import (
	"log"
	"time"

	mqtt "github.com/eclipse/paho.mqtt.golang"
)

// Config holds broker connection settings.
type Config struct {
	Broker   string // e.g. "tcp://localhost:1883"
	ClientID string
}

// Client wraps the paho MQTT client and routes inbound messages to a handler.
type Client struct {
	c mqtt.Client
}

// Handler receives inbound payloads with their topic.
type Handler func(topic string, payload []byte)

// New connects to the broker and returns a ready Client. On connect it
// subscribes to all edge telemetry and event topics.
func New(cfg Config, h Handler) (*Client, error) {
	opts := mqtt.NewClientOptions().
		AddBroker(cfg.Broker).
		SetClientID(cfg.ClientID).
		SetAutoReconnect(true).
		SetConnectRetry(true).
		SetConnectRetryInterval(2 * time.Second).
		SetOnConnectHandler(func(c mqtt.Client) {
			c.Subscribe("tpt/v1/+/tele/+", 0, func(_ mqtt.Client, m mqtt.Message) {
				h(m.Topic(), m.Payload())
			})
			c.Subscribe("tpt/v1/+/evt/+", 1, func(_ mqtt.Client, m mqtt.Message) {
				h(m.Topic(), m.Payload())
			})
			c.Subscribe("tpt/v1/+/status", 1, func(_ mqtt.Client, m mqtt.Message) {
				h(m.Topic(), m.Payload())
			})
		})

	c := mqtt.NewClient(opts)
	t := c.Connect()
	if !t.WaitTimeout(10 * time.Second) {
		log.Printf("mqtt: connect timeout to %s", cfg.Broker)
	} else if err := t.Error(); err != nil {
		log.Printf("mqtt: connect error: %v", err)
	}
	return &Client{c: c}, nil
}

// Publish sends a command payload to a node topic.
func (cl *Client) Publish(topic string, payload []byte) error {
	t := cl.c.Publish(topic, 1, false, payload)
	t.Wait()
	return t.Error()
}
