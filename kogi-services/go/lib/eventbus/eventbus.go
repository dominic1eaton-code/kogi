package eventbus

import (
	"fmt"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

type Event struct {
	ID        string            `json:"id"`
	Topic     string            `json:"topic"`
	Payload   string            `json:"payload"`
	Source    string            `json:"source"`
	Target    string            `json:"target,omitempty"`
	Metadata  map[string]string `json:"metadata,omitempty"`
	Timestamp string            `json:"timestamp"`
}

type PublishOptions struct {
	Source   string
	Target   string
	Metadata map[string]string
}

type subscriber struct {
	Consumer string
	Handler  func(Event)
}

type Bus struct {
	mu          sync.RWMutex
	subscribers map[string][]subscriber
	wildcard    []subscriber
	history     []Event
	maxHistory  int
	sequence    uint64
	topicCounts map[string]uint64
}

func New() *Bus {
	return NewWithHistory(10000)
}

func NewWithHistory(maxHistory int) *Bus {
	if maxHistory <= 0 {
		maxHistory = 1000
	}
	return &Bus{
		subscribers: map[string][]subscriber{},
		wildcard:    []subscriber{},
		history:     []Event{},
		maxHistory:  maxHistory,
		topicCounts: map[string]uint64{},
	}
}

func (b *Bus) Subscribe(topic string, fn func(Event)) {
	_ = b.SubscribeNamed(topic, "anonymous", fn)
}

func (b *Bus) SubscribeNamed(topic, consumer string, fn func(Event)) string {
	b.mu.Lock()
	defer b.mu.Unlock()

	if consumer == "" {
		consumer = "anonymous"
	}

	id := fmt.Sprintf("%s:%s:%d", topic, consumer, atomic.AddUint64(&b.sequence, 1))
	b.subscribers[topic] = append(b.subscribers[topic], subscriber{
		Consumer: consumer,
		Handler:  fn,
	})
	return id
}

func (b *Bus) SubscribeAll(consumer string, fn func(Event)) string {
	b.mu.Lock()
	defer b.mu.Unlock()

	if consumer == "" {
		consumer = "anonymous"
	}

	id := fmt.Sprintf("*:%s:%d", consumer, atomic.AddUint64(&b.sequence, 1))
	b.wildcard = append(b.wildcard, subscriber{
		Consumer: consumer,
		Handler:  fn,
	})
	return id
}

func (b *Bus) Publish(topic, payload string) Event {
	return b.PublishWith(topic, payload, PublishOptions{})
}

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
	handlers := cloneSubscribers(b.subscribers[topic])
	handlers = append(handlers, cloneSubscribers(b.wildcard)...)

	b.history = append(b.history, event)
	if len(b.history) > b.maxHistory {
		start := len(b.history) - b.maxHistory
		b.history = append([]Event(nil), b.history[start:]...)
	}
	b.topicCounts[topic]++
	b.mu.Unlock()

	for _, handler := range handlers {
		handler.Handler(event)
	}

	return event
}

func (b *Bus) History(limit int, topic string) []Event {
	b.mu.RLock()
	defer b.mu.RUnlock()

	if limit <= 0 {
		limit = len(b.history)
	}

	buffer := make([]Event, 0, min(limit, len(b.history)))
	for i := len(b.history) - 1; i >= 0 && len(buffer) < limit; i-- {
		event := b.history[i]
		if topic != "" && event.Topic != topic {
			continue
		}
		buffer = append(buffer, event)
	}

	for i, j := 0, len(buffer)-1; i < j; i, j = i+1, j-1 {
		buffer[i], buffer[j] = buffer[j], buffer[i]
	}

	return buffer
}

func (b *Bus) TopicStats() map[string]uint64 {
	b.mu.RLock()
	defer b.mu.RUnlock()

	stats := make(map[string]uint64, len(b.topicCounts))
	for topic, count := range b.topicCounts {
		stats[topic] = count
	}
	return stats
}

func (b *Bus) Subscriptions() map[string][]string {
	b.mu.RLock()
	defer b.mu.RUnlock()

	result := make(map[string][]string, len(b.subscribers)+1)
	for topic, subs := range b.subscribers {
		names := make([]string, 0, len(subs))
		for _, sub := range subs {
			names = append(names, sub.Consumer)
		}
		sort.Strings(names)
		result[topic] = names
	}

	if len(b.wildcard) > 0 {
		names := make([]string, 0, len(b.wildcard))
		for _, sub := range b.wildcard {
			names = append(names, sub.Consumer)
		}
		sort.Strings(names)
		result["*"] = names
	}

	return result
}

func cloneSubscribers(input []subscriber) []subscriber {
	output := make([]subscriber, len(input))
	copy(output, input)
	return output
}

func cloneMap(input map[string]string) map[string]string {
	if len(input) == 0 {
		return map[string]string{}
	}

	output := make(map[string]string, len(input))
	for key, value := range input {
		output[key] = value
	}
	return output
}

func defaultIfEmpty(value, fallback string) string {
	if value == "" {
		return fallback
	}
	return value
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
