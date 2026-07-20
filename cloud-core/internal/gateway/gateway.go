// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package gateway implements the WebSocket gateway that bridges the Client UI
// to the MQTT bridge (Cloud Core <-> Edge Agent). See docs/architecture.md.
package gateway

import (
	"encoding/json"
	"log"
	"net/http"
	"sync"

	"github.com/gorilla/websocket"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/metrics"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true }, // TODO: restrict origin in prod.
}

// Hub fans out telemetry/events to subscribed clients and routes commands to
// the MQTT bridge.
type Hub struct {
	mu       sync.RWMutex
	clients  map[*Client]struct{}
	bridge   CommandSink
	register chan *Client
}

// CommandSink forwards a command envelope to the MQTT bridge.
type CommandSink interface {
	SendCommand(env protocol.Envelope) error
}

func NewHub(bridge CommandSink) *Hub {
	return &Hub{
		clients:  make(map[*Client]struct{}),
		bridge:   bridge,
		register: make(chan *Client),
	}
}

func (h *Hub) Run() {
	for c := range h.register {
		h.mu.Lock()
		h.clients[c] = struct{}{}
		h.mu.Unlock()
	}
}

// Broadcast delivers a telemetry/event envelope to all connected clients.
func (h *Hub) Broadcast(env protocol.Envelope) {
	metrics.TelemetryPublished.Inc()
	h.mu.RLock()
	defer h.mu.RUnlock()
	for c := range h.clients {
		c.send <- env
	}
}

func (h *Hub) ServeWS(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("ws upgrade: %v", err)
		return
	}
	c := &Client{conn: conn, send: make(chan protocol.Envelope, 64), hub: h}
	h.register <- c
	go c.readLoop()
	go c.writeLoop()
}

// Client is a single WebSocket connection.
type Client struct {
	conn *websocket.Conn
	send chan protocol.Envelope
	hub  *Hub
}

func (c *Client) readLoop() {
	defer c.conn.Close()
	for {
		_, data, err := c.conn.ReadMessage()
		if err != nil {
			return
		}
		var env protocol.Envelope
		if err := json.Unmarshal(data, &env); err != nil {
			continue
		}
		if isCommand(env.Type) {
			metrics.CommandsReceived.WithLabelValues(env.Type).Inc()
			if err := c.hub.bridge.SendCommand(env); err != nil {
				c.ack(env.ID, err)
			}
		}
	}
}

func (c *Client) writeLoop() {
	defer c.conn.Close()
	for env := range c.send {
		b, _ := json.Marshal(env)
		_ = c.conn.WriteMessage(websocket.TextMessage, b)
	}
}

func (c *Client) ack(id string, err error) {
	env := protocol.Envelope{Type: protocol.MsgAck, ID: id}
	b, _ := json.Marshal(env)
	_ = c.conn.WriteMessage(websocket.TextMessage, b)
}

func isCommand(t string) bool {
	switch t {
	case protocol.CmdSlew, protocol.CmdSlewStop, protocol.CmdFocus,
		protocol.CmdFocusRel, protocol.CmdWeather, protocol.CmdImagingStart,
		protocol.CmdImagingStop, protocol.CmdSubscribe:
		return true
	}
	return false
}
