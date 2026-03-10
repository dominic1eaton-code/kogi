package eventbus

import "sync"

type Event struct {
    Topic   string
    Payload string
}

type Bus struct {
    mu          sync.RWMutex
    subscribers map[string][]func(Event)
}

func New() *Bus {
    return &Bus{subscribers: map[string][]func(Event){}}
}

func (b *Bus) Subscribe(topic string, fn func(Event)) {
    b.mu.Lock()
    defer b.mu.Unlock()
    b.subscribers[topic] = append(b.subscribers[topic], fn)
}

func (b *Bus) Publish(topic, payload string) {
    b.mu.RLock()
    handlers := append([]func(Event){}, b.subscribers[topic]...)
    b.mu.RUnlock()

    evt := Event{Topic: topic, Payload: payload}
    for _, h := range handlers {
        h(evt)
    }
}