package eventbus

// eventbus.go
//
// In-process pub/sub bus used by the Kogi gateway and all Go services.
//
// New capabilities added to support the full PortfolioSystem event surface:
//
//   Prefix subscriptions
//     SubscribePrefix(prefix, consumer, fn) — fires on any topic that starts
//     with prefix (e.g. "portfolio." catches all 19 portfolio topics).
//
//   One-shot subscriptions
//     SubscribeOnce(topic, consumer, fn) — automatically unsubscribes after
//     the first matching event is delivered.
//
//   Unsubscribe by ID
//     Unsubscribe(subscriptionID) — remove any subscription returned by
//     SubscribeNamed / SubscribePrefix / SubscribeAll.
//
//   Dead-letter queue
//     Events published to topics that have zero subscribers are appended to a
//     separate dead-letter ring (capacity: maxHistory / 4, min 256).
//     DeadLetters(limit) returns them newest-first.
//
//   Replay
//     Replay(topic, from, to) returns the slice of history events for a topic
//     within an inclusive timestamp window (RFC3339Nano strings).
//     ReplayPrefix(prefix, from, to) does the same for prefix-matched topics.
//
//   Filtered history
//     HistoryPrefix(prefix, limit) — like History but matches all topics that
//     begin with prefix.
//     HistorySource(source, limit) — filters by event source.
//
//   Pause / Resume
//     Pause() / Resume() — stop/restart event delivery to subscribers without
//     losing events; they are queued internally and flushed on Resume.
//
//   Metrics
//     Metrics() returns a snapshot of per-topic publish counts, subscriber
//     counts, dead-letter count, and total events published.
//
//   PublishBatch
//     PublishBatch(events []BatchItem) publishes multiple events atomically
//     under a single lock acquisition for the history append; handlers are
//     called outside the lock as normal.

import (
	"fmt"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

// ── Public types ──────────────────────────────────────────────────────────────

// Event is an immutable record of one published message.
type Event struct {
	ID        string            `json:"id"`
	Topic     string            `json:"topic"`
	Payload   string            `json:"payload"`
	Source    string            `json:"source"`
	Target    string            `json:"target,omitempty"`
	Metadata  map[string]string `json:"metadata,omitempty"`
	Timestamp string            `json:"timestamp"`
}

// PublishOptions controls optional fields on a published event.
type PublishOptions struct {
	Source   string
	Target   string
	Metadata map[string]string
}

// BatchItem is a single entry in a PublishBatch call.
type BatchItem struct {
	Topic   string
	Payload string
	Options PublishOptions
}

// BusMetrics is a read-only snapshot of bus activity.
type BusMetrics struct {
	TotalPublished   uint64            `json:"total_published"`
	DeadLetterCount  int               `json:"dead_letter_count"`
	TopicCounts      map[string]uint64 `json:"topic_counts"`
	SubscriberCounts map[string]int    `json:"subscriber_counts"`
	PrefixSubCount   int               `json:"prefix_subscriber_count"`
	WildcardSubCount int               `json:"wildcard_subscriber_count"`
	Paused           bool              `json:"paused"`
}

// ── Internal types ────────────────────────────────────────────────────────────

type subscriber struct {
	ID       string
	Consumer string
	Handler  func(Event)
	once     bool // true → remove after first delivery
}

type prefixSubscriber struct {
	ID       string
	Prefix   string
	Consumer string
	Handler  func(Event)
}

// ── Bus ───────────────────────────────────────────────────────────────────────

// Bus is a concurrent in-process event bus.  All methods are safe for
// concurrent use from multiple goroutines.
type Bus struct {
	mu          sync.RWMutex
	subscribers map[string][]subscriber  // topic → []subscriber
	prefixSubs  []prefixSubscriber        // prefix-matched subscribers
	wildcard    []subscriber              // "*" catch-all subscribers
	history     []Event
	deadLetters []Event
	maxHistory  int
	maxDead     int
	sequence    uint64
	topicCounts map[string]uint64
	totalPub    uint64

	// pause / resume
	pauseMu sync.Mutex
	paused  bool
	pending []pendingDelivery
}

type pendingDelivery struct {
	handlers []subscriber
	event    Event
}

// New returns a Bus with 10 000-event history.
func New() *Bus {
	return NewWithHistory(10000)
}

// NewWithHistory returns a Bus with the given history capacity.
func NewWithHistory(maxHistory int) *Bus {
	if maxHistory <= 0 {
		maxHistory = 1000
	}
	maxDead := maxHistory / 4
	if maxDead < 256 {
		maxDead = 256
	}
	return &Bus{
		subscribers: map[string][]subscriber{},
		prefixSubs:  []prefixSubscriber{},
		wildcard:    []subscriber{},
		history:     []Event{},
		deadLetters: []Event{},
		maxHistory:  maxHistory,
		maxDead:     maxDead,
		topicCounts: map[string]uint64{},
	}
}

// ── Subscribe ─────────────────────────────────────────────────────────────────

// Subscribe registers an anonymous handler for an exact topic.
func (b *Bus) Subscribe(topic string, fn func(Event)) {
	_ = b.SubscribeNamed(topic, "anonymous", fn)
}

// SubscribeNamed registers a named handler for an exact topic and returns a
// subscription ID that can be passed to Unsubscribe.
func (b *Bus) SubscribeNamed(topic, consumer string, fn func(Event)) string {
	b.mu.Lock()
	defer b.mu.Unlock()
	if consumer == "" {
		consumer = "anonymous"
	}
	id := fmt.Sprintf("sub:%s:%s:%d", topic, consumer, atomic.AddUint64(&b.sequence, 1))
	b.subscribers[topic] = append(b.subscribers[topic], subscriber{
		ID: id, Consumer: consumer, Handler: fn,
	})
	return id
}

// SubscribeOnce registers a handler that fires exactly once then is removed.
func (b *Bus) SubscribeOnce(topic, consumer string, fn func(Event)) string {
	b.mu.Lock()
	defer b.mu.Unlock()
	if consumer == "" {
		consumer = "anonymous"
	}
	id := fmt.Sprintf("once:%s:%s:%d", topic, consumer, atomic.AddUint64(&b.sequence, 1))
	b.subscribers[topic] = append(b.subscribers[topic], subscriber{
		ID: id, Consumer: consumer, Handler: fn, once: true,
	})
	return id
}

// SubscribePrefix registers a handler for all topics whose name begins with
// prefix (e.g. "portfolio." matches all portfolio system events).
func (b *Bus) SubscribePrefix(prefix, consumer string, fn func(Event)) string {
	b.mu.Lock()
	defer b.mu.Unlock()
	if consumer == "" {
		consumer = "anonymous"
	}
	id := fmt.Sprintf("pfx:%s:%s:%d", prefix, consumer, atomic.AddUint64(&b.sequence, 1))
	b.prefixSubs = append(b.prefixSubs, prefixSubscriber{
		ID: id, Prefix: prefix, Consumer: consumer, Handler: fn,
	})
	return id
}

// SubscribeAll registers a handler that receives every published event.
func (b *Bus) SubscribeAll(consumer string, fn func(Event)) string {
	b.mu.Lock()
	defer b.mu.Unlock()
	if consumer == "" {
		consumer = "anonymous"
	}
	id := fmt.Sprintf("wild:*:%s:%d", consumer, atomic.AddUint64(&b.sequence, 1))
	b.wildcard = append(b.wildcard, subscriber{
		ID: id, Consumer: consumer, Handler: fn,
	})
	return id
}

// Unsubscribe removes a subscription by its ID (returned by SubscribeNamed /
// SubscribePrefix / SubscribeAll / SubscribeOnce).
// Returns true if the subscription was found and removed.
func (b *Bus) Unsubscribe(id string) bool {
	b.mu.Lock()
	defer b.mu.Unlock()

	// Exact-topic subscribers
	for topic, subs := range b.subscribers {
		for i, s := range subs {
			if s.ID == id {
				b.subscribers[topic] = append(subs[:i], subs[i+1:]...)
				return true
			}
		}
	}
	// Prefix subscribers
	for i, ps := range b.prefixSubs {
		if ps.ID == id {
			b.prefixSubs = append(b.prefixSubs[:i], b.prefixSubs[i+1:]...)
			return true
		}
	}
	// Wildcard subscribers
	for i, w := range b.wildcard {
		if w.ID == id {
			b.wildcard = append(b.wildcard[:i], b.wildcard[i+1:]...)
			return true
		}
	}
	return false
}

// ── Publish ───────────────────────────────────────────────────────────────────

// Publish publishes an event with no options.
func (b *Bus) Publish(topic, payload string) Event {
	return b.PublishWith(topic, payload, PublishOptions{})
}

// PublishWith publishes an event with full options.
func (b *Bus) PublishWith(topic, payload string, options PublishOptions) Event {
	event := Event{
		ID:        fmt.Sprintf("evt-%d", atomic.AddUint64(&b.sequence, 1)),
		Topic:     topic,
		Payload:   payload,
		Source:    defaultIfEmpty(options.Source, "gateway"),
		Target:    options.Target,
		Metadata:  cloneMap(options.Metadata),
		Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
	}

	b.mu.Lock()

	// Collect exact-topic handlers; remove once-subscribers
	exact := cloneSubscribers(b.subscribers[topic])
	remaining := b.subscribers[topic][:0]
	for _, s := range b.subscribers[topic] {
		if !s.once {
			remaining = append(remaining, s)
		}
	}
	b.subscribers[topic] = remaining

	// Collect prefix handlers
	var prefixHandlers []subscriber
	for _, ps := range b.prefixSubs {
		if strings.HasPrefix(topic, ps.Prefix) {
			prefixHandlers = append(prefixHandlers, subscriber{
				ID: ps.ID, Consumer: ps.Consumer, Handler: ps.Handler,
			})
		}
	}

	wildcards := cloneSubscribers(b.wildcard)

	// Append to history
	b.history = append(b.history, event)
	if len(b.history) > b.maxHistory {
		start := len(b.history) - b.maxHistory
		b.history = append([]Event(nil), b.history[start:]...)
	}
	b.topicCounts[topic]++
	atomic.AddUint64(&b.totalPub, 1)

	// Dead-letter if no subscribers at all
	totalHandlers := len(exact) + len(prefixHandlers) + len(wildcards)
	if totalHandlers == 0 {
		b.deadLetters = append(b.deadLetters, event)
		if len(b.deadLetters) > b.maxDead {
			start := len(b.deadLetters) - b.maxDead
			b.deadLetters = append([]Event(nil), b.deadLetters[start:]...)
		}
	}

	// Check pause state
	paused := b.paused
	if paused {
		allHandlers := append(append(exact, prefixHandlers...), wildcards...)
		b.pending = append(b.pending, pendingDelivery{handlers: allHandlers, event: event})
		b.mu.Unlock()
		return event
	}
	b.mu.Unlock()

	// Deliver outside the lock
	allHandlers := append(append(exact, prefixHandlers...), wildcards...)
	for _, h := range allHandlers {
		h.Handler(event)
	}

	return event
}

// PublishBatch publishes multiple events.  History appends are batched under
// a single lock; handlers are called outside the lock per event.
func (b *Bus) PublishBatch(items []BatchItem) []Event {
	if len(items) == 0 {
		return nil
	}

	events := make([]Event, len(items))
	allDeliveries := make([]pendingDelivery, 0, len(items))

	b.mu.Lock()
	for i, item := range items {
		ev := Event{
			ID:        fmt.Sprintf("evt-%d", atomic.AddUint64(&b.sequence, 1)),
			Topic:     item.Topic,
			Payload:   item.Payload,
			Source:    defaultIfEmpty(item.Options.Source, "gateway"),
			Target:    item.Options.Target,
			Metadata:  cloneMap(item.Options.Metadata),
			Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
		}
		events[i] = ev

		exact := cloneSubscribers(b.subscribers[item.Topic])
		// Remove once-subscribers
		remaining := b.subscribers[item.Topic][:0]
		for _, s := range b.subscribers[item.Topic] {
			if !s.once {
				remaining = append(remaining, s)
			}
		}
		b.subscribers[item.Topic] = remaining

		var prefixHandlers []subscriber
		for _, ps := range b.prefixSubs {
			if strings.HasPrefix(item.Topic, ps.Prefix) {
				prefixHandlers = append(prefixHandlers, subscriber{
					ID: ps.ID, Consumer: ps.Consumer, Handler: ps.Handler,
				})
			}
		}
		wildcards := cloneSubscribers(b.wildcard)

		b.history = append(b.history, ev)
		b.topicCounts[item.Topic]++
		atomic.AddUint64(&b.totalPub, 1)

		if len(b.history) > b.maxHistory {
			start := len(b.history) - b.maxHistory
			b.history = append([]Event(nil), b.history[start:]...)
		}

		total := len(exact) + len(prefixHandlers) + len(wildcards)
		if total == 0 {
			b.deadLetters = append(b.deadLetters, ev)
			if len(b.deadLetters) > b.maxDead {
				start := len(b.deadLetters) - b.maxDead
				b.deadLetters = append([]Event(nil), b.deadLetters[start:]...)
			}
		}

		allDeliveries = append(allDeliveries, pendingDelivery{
			event:    ev,
			handlers: append(append(exact, prefixHandlers...), wildcards...),
		})
	}
	paused := b.paused
	if paused {
		b.pending = append(b.pending, allDeliveries...)
	}
	b.mu.Unlock()

	if !paused {
		for _, d := range allDeliveries {
			for _, h := range d.handlers {
				h.Handler(d.event)
			}
		}
	}
	return events
}

// ── Pause / Resume ────────────────────────────────────────────────────────────

// Pause suspends handler delivery.  Events are still recorded in history.
// Queued events are flushed when Resume is called.
func (b *Bus) Pause() {
	b.pauseMu.Lock()
	defer b.pauseMu.Unlock()
	b.mu.Lock()
	b.paused = true
	b.mu.Unlock()
}

// Resume re-enables handler delivery and flushes any pending events.
func (b *Bus) Resume() {
	b.pauseMu.Lock()
	defer b.pauseMu.Unlock()

	b.mu.Lock()
	b.paused = false
	pending := b.pending
	b.pending = nil
	b.mu.Unlock()

	for _, d := range pending {
		for _, h := range d.handlers {
			h.Handler(d.event)
		}
	}
}

// ── Query ─────────────────────────────────────────────────────────────────────

// History returns up to limit events for an exact topic (all if limit ≤ 0).
// Pass topic="" to retrieve across all topics.
func (b *Bus) History(limit int, topic string) []Event {
	b.mu.RLock()
	defer b.mu.RUnlock()

	if limit <= 0 {
		limit = len(b.history)
	}

	buffer := make([]Event, 0, min(limit, len(b.history)))
	for i := len(b.history) - 1; i >= 0 && len(buffer) < limit; i-- {
		ev := b.history[i]
		if topic != "" && ev.Topic != topic {
			continue
		}
		buffer = append(buffer, ev)
	}
	reverseEvents(buffer)
	return buffer
}

// HistoryPrefix returns up to limit events whose topic starts with prefix.
func (b *Bus) HistoryPrefix(prefix string, limit int) []Event {
	b.mu.RLock()
	defer b.mu.RUnlock()

	if limit <= 0 {
		limit = len(b.history)
	}

	buffer := make([]Event, 0, min(limit, len(b.history)))
	for i := len(b.history) - 1; i >= 0 && len(buffer) < limit; i-- {
		ev := b.history[i]
		if strings.HasPrefix(ev.Topic, prefix) {
			buffer = append(buffer, ev)
		}
	}
	reverseEvents(buffer)
	return buffer
}

// HistorySource returns up to limit events whose Source field matches source.
func (b *Bus) HistorySource(source string, limit int) []Event {
	b.mu.RLock()
	defer b.mu.RUnlock()

	if limit <= 0 {
		limit = len(b.history)
	}

	buffer := make([]Event, 0, min(limit, len(b.history)))
	for i := len(b.history) - 1; i >= 0 && len(buffer) < limit; i-- {
		ev := b.history[i]
		if ev.Source == source {
			buffer = append(buffer, ev)
		}
	}
	reverseEvents(buffer)
	return buffer
}

// Replay returns all history events for an exact topic whose timestamp falls
// within [from, to] (inclusive, RFC3339Nano strings; empty string = unbounded).
func (b *Bus) Replay(topic, from, to string) []Event {
	b.mu.RLock()
	defer b.mu.RUnlock()

	var result []Event
	for _, ev := range b.history {
		if topic != "" && ev.Topic != topic {
			continue
		}
		if from != "" && ev.Timestamp < from {
			continue
		}
		if to != "" && ev.Timestamp > to {
			continue
		}
		result = append(result, ev)
	}
	return result
}

// ReplayPrefix returns history events for topics starting with prefix,
// within the given timestamp window.
func (b *Bus) ReplayPrefix(prefix, from, to string) []Event {
	b.mu.RLock()
	defer b.mu.RUnlock()

	var result []Event
	for _, ev := range b.history {
		if !strings.HasPrefix(ev.Topic, prefix) {
			continue
		}
		if from != "" && ev.Timestamp < from {
			continue
		}
		if to != "" && ev.Timestamp > to {
			continue
		}
		result = append(result, ev)
	}
	return result
}

// DeadLetters returns up to limit events that had no subscribers at publish
// time (newest first).
func (b *Bus) DeadLetters(limit int) []Event {
	b.mu.RLock()
	defer b.mu.RUnlock()

	if limit <= 0 {
		limit = len(b.deadLetters)
	}

	n := len(b.deadLetters)
	start := n - limit
	if start < 0 {
		start = 0
	}
	result := make([]Event, n-start)
	copy(result, b.deadLetters[start:])
	reverseEvents(result)
	return result
}

// TopicStats returns a copy of per-topic publish counts.
func (b *Bus) TopicStats() map[string]uint64 {
	b.mu.RLock()
	defer b.mu.RUnlock()

	stats := make(map[string]uint64, len(b.topicCounts))
	for topic, count := range b.topicCounts {
		stats[topic] = count
	}
	return stats
}

// Subscriptions returns a map of topic → []consumerName for all exact-topic
// subscriptions.  Wildcard subscribers appear under "*".
// Prefix subscribers appear under "prefix:<prefix>".
func (b *Bus) Subscriptions() map[string][]string {
	b.mu.RLock()
	defer b.mu.RUnlock()

	result := make(map[string][]string)

	for topic, subs := range b.subscribers {
		names := make([]string, 0, len(subs))
		for _, s := range subs {
			names = append(names, s.Consumer)
		}
		sort.Strings(names)
		result[topic] = names
	}
	if len(b.wildcard) > 0 {
		names := make([]string, 0, len(b.wildcard))
		for _, s := range b.wildcard {
			names = append(names, s.Consumer)
		}
		sort.Strings(names)
		result["*"] = names
	}
	for _, ps := range b.prefixSubs {
		key := "prefix:" + ps.Prefix
		result[key] = append(result[key], ps.Consumer)
	}
	for key := range result {
		if strings.HasPrefix(key, "prefix:") {
			sort.Strings(result[key])
		}
	}
	return result
}

// Metrics returns a snapshot of bus activity counters.
func (b *Bus) Metrics() BusMetrics {
	b.mu.RLock()
	defer b.mu.RUnlock()

	topicCounts := make(map[string]uint64, len(b.topicCounts))
	for t, c := range b.topicCounts {
		topicCounts[t] = c
	}

	subCounts := make(map[string]int, len(b.subscribers))
	for t, subs := range b.subscribers {
		subCounts[t] = len(subs)
	}

	return BusMetrics{
		TotalPublished:   atomic.LoadUint64(&b.totalPub),
		DeadLetterCount:  len(b.deadLetters),
		TopicCounts:      topicCounts,
		SubscriberCounts: subCounts,
		PrefixSubCount:   len(b.prefixSubs),
		WildcardSubCount: len(b.wildcard),
		Paused:           b.paused,
	}
}

// ── Internal helpers ──────────────────────────────────────────────────────────

func cloneSubscribers(input []subscriber) []subscriber {
	if len(input) == 0 {
		return nil
	}
	output := make([]subscriber, len(input))
	copy(output, input)
	return output
}

func cloneMap(input map[string]string) map[string]string {
	if len(input) == 0 {
		return map[string]string{}
	}
	output := make(map[string]string, len(input))
	for k, v := range input {
		output[k] = v
	}
	return output
}

func defaultIfEmpty(value, fallback string) string {
	if value == "" {
		return fallback
	}
	return value
}

func reverseEvents(s []Event) {
	for i, j := 0, len(s)-1; i < j; i, j = i+1, j-1 {
		s[i], s[j] = s[j], s[i]
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
