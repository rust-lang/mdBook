package serve

import (
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

// reloadMessage is the literal payload the front-end book.js expects. The
// Rust implementation sends the same string over a tokio broadcast
// channel; we hard-code it here to keep the contract obvious. Changing
// it would require a matching change in the theme's book.js.
const reloadMessage = "reload"

// upgrader configures the WebSocket handshake. We allow any origin
// because the served book is expected to be opened via a local browser;
// if a user really wants to lock this down they can put mdbook behind
// a reverse proxy. The Rust CLI uses axum's default which is similarly
// permissive.
var upgrader = websocket.Upgrader{
	CheckOrigin:     func(r *http.Request) bool { return true },
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

// ReloadHub tracks active WebSocket clients and broadcasts a single
// "reload" message to all of them when a build finishes. It is the Go
// counterpart of the `tokio::sync::broadcast::channel::<Message>(100)`
// in src/cmd/serve.rs.
//
// The hub keeps one goroutine per client that pumps outgoing messages.
// Incoming messages are not consumed (the front-end doesn't send
// anything); the read loop exists only to detect closed connections.
type ReloadHub struct {
	mu      sync.Mutex
	clients map[*hubClient]struct{}
	logger  *log.Logger
	closed  bool
}

type hubClient struct {
	conn *websocket.Conn
	send chan string
}

// newReloadHub constructs a hub that logs to logger (or the default log).
func newReloadHub(logger *log.Logger) *ReloadHub {
	if logger == nil {
		logger = log.Default()
	}
	return &ReloadHub{
		clients: map[*hubClient]struct{}{},
		logger:  logger,
	}
}

// handle is the http.HandlerFunc mounted at /<LiveReloadEndpoint>. It
// upgrades the connection, registers the client, and starts its pump
// goroutine. The handler returns when the client disconnects.
func (h *ReloadHub) handle(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		// upgrader.Upgrade already wrote a response; we just log.
		h.logger.Printf("serve: ws upgrade: %v", err)
		return
	}
	client := &hubClient{conn: conn, send: make(chan string, 4)}
	h.add(client)
	defer h.remove(client)

	// Reader goroutine: discard everything, but exit when the conn
	// closes so the handler returns and we unregister.
	closed := make(chan struct{})
	go func() {
		defer close(closed)
		for {
			if _, _, err := conn.NextReader(); err != nil {
				return
			}
		}
	}()

	for {
		select {
		case <-closed:
			return
		case msg, ok := <-client.send:
			if !ok {
				return
			}
			// 5s write deadline so a wedged client can't pin the
			// hub goroutine forever.
			_ = conn.SetWriteDeadline(time.Now().Add(5 * time.Second))
			if err := conn.WriteMessage(websocket.TextMessage, []byte(msg)); err != nil {
				h.logger.Printf("serve: ws write: %v", err)
				return
			}
		}
	}
}

// broadcast enqueues a reload message on every connected client. The
// send is non-blocking — clients that are slow to drain their buffer
// simply miss this notification and will pick up the next one.
func (h *ReloadHub) broadcast() {
	h.mu.Lock()
	defer h.mu.Unlock()
	if h.closed {
		return
	}
	for c := range h.clients {
		select {
		case c.send <- reloadMessage:
		default:
			// Drop on the floor; the next broadcast will catch them.
		}
	}
}

func (h *ReloadHub) add(c *hubClient) {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.clients[c] = struct{}{}
}

func (h *ReloadHub) remove(c *hubClient) {
	h.mu.Lock()
	defer h.mu.Unlock()
	if _, ok := h.clients[c]; !ok {
		return
	}
	delete(h.clients, c)
	close(c.send)
	_ = c.conn.Close()
}

func (h *ReloadHub) close() {
	h.mu.Lock()
	defer h.mu.Unlock()
	if h.closed {
		return
	}
	h.closed = true
	for c := range h.clients {
		_ = c.conn.Close()
		delete(h.clients, c)
	}
}
