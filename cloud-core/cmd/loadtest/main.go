// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Command loadtest opens N concurrent WebSocket connections to the Cloud Core
// gateway, streams commands from each, and reports aggregate throughput and
// latency percentiles. Used to validate the "concurrent WebSocket connections"
// target in Phase 8. Run:
//
//	go run ./cmd/loadtest -url ws://localhost:8080/ws -conns 1000 -rate 200
package main

import (
	"flag"
	"log"
	"sync"
	"sync/atomic"
	"time"

	"github.com/gorilla/websocket"
	"github.com/tptsolutions/tpt-astrolink/cloud-core/internal/protocol"
)

func main() {
	url := flag.String("url", "ws://localhost:8080/ws", "Cloud Core WebSocket URL")
	conns := flag.Int("conns", 500, "number of concurrent connections")
	dur := flag.Duration("dur", 30*time.Second, "test duration")
	rate := flag.Int("rate", 100, "commands per second across all connections")
	flag.Parse()

	log.Printf("loadtest: %d conns, %s duration, %d cmd/s", *conns, *dur, *rate)

	var (
		sent     int64
		acked    int64
		latSum   int64
		latMax   int64
		latMin   int64 = 1 << 62
		failConn int64
	)

	var wg sync.WaitGroup
	start := time.Now()
	stop := start.Add(*dur)

	// Token-bucket limiter shared across connections.
	tokens := make(chan struct{}, *rate)
	go func() {
		interval := time.Second / time.Duration(*rate)
		ticker := time.NewTicker(interval)
		defer ticker.Stop()
		for time.Now().Before(stop) {
			select {
			case tokens <- struct{}{}:
			case <-ticker.C:
				select {
				case tokens <- struct{}{}:
				default:
				}
			}
		}
	}()

	for i := 0; i < *conns; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			c, _, err := websocket.DefaultDialer.Dial(*url, nil)
			if err != nil {
				atomic.AddInt64(&failConn, 1)
				return
			}
			defer c.Close()

			// Reader goroutine tallies acks and records latency.
			done := make(chan struct{})
			go func() {
				defer close(done)
				for {
					_, data, err := c.ReadMessage()
					if err != nil {
						return
					}
					var env protocol.Envelope
					if err := jsonUnmarshal(data, &env); err != nil {
						continue
					}
					if env.Type != protocol.MsgAck {
						continue
					}
					atomic.AddInt64(&acked, 1)
					if ts, ok := parseTS(env.TS); ok {
						lat := time.Since(ts).Microseconds()
						atomic.AddInt64(&latSum, lat)
						for {
							cur := atomic.LoadInt64(&latMax)
							if lat <= cur || atomic.CompareAndSwapInt64(&latMax, cur, lat) {
								break
							}
						}
						for {
							cur := atomic.LoadInt64(&latMin)
							if lat >= cur || atomic.CompareAndSwapInt64(&latMin, cur, lat) {
								break
							}
						}
					}
				}
			}()

			var seq int64
			for time.Now().Before(stop) {
				select {
				case <-tokens:
				default:
					time.Sleep(time.Millisecond)
					continue
				}
				env := protocol.Envelope{
					Type:   protocol.CmdSlew,
					ID:     uuidLike(id, atomic.AddInt64(&seq, 1)),
					NodeID: "node-load",
					TS:     time.Now().UTC().Format(time.RFC3339Nano),
					Payload: cmdPayload(),
				}
				if err := c.WriteJSON(env); err != nil {
					return
				}
				atomic.AddInt64(&sent, 1)
			}
			<-done
		}(i)
	}

	wg.Wait()
	elapsed := time.Since(start)

	log.Printf("results:")
	log.Printf("  connections attempted : %d", *conns)
	log.Printf("  failed connects       : %d", atomic.LoadInt64(&failConn))
	log.Printf("  commands sent         : %d", atomic.LoadInt64(&sent))
	log.Printf("  acks received         : %d", atomic.LoadInt64(&acked))
	log.Printf("  elapsed               : %s", elapsed.Round(time.Millisecond))
	if s := atomic.LoadInt64(&sent); s > 0 {
		log.Printf("  throughput            : %.1f cmd/s", float64(s)/elapsed.Seconds())
	}
	if a := atomic.LoadInt64(&acked); a > 0 {
		log.Printf("  ack latency avg (ms)  : %.3f", float64(atomic.LoadInt64(&latSum))/float64(a)/1000.0)
		log.Printf("  ack latency min (ms)  : %.3f", float64(atomic.LoadInt64(&latMin))/1000.0)
		log.Printf("  ack latency max (ms)  : %.3f", float64(atomic.LoadInt64(&latMax))/1000.0)
	}
}
