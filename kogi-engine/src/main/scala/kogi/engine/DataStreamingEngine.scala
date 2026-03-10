package kogi.engine

final class DataStreamingEngine(maxEvents: Int = 10000) {
  private var stream: Vector[StreamEvent] = Vector.empty
  private var subscribers: Map[Int, StreamEvent => Unit] = Map.empty
  private var nextSubscriberId: Int = 1

  def ingest(event: StreamEvent): Unit = {
    stream = (stream :+ event).takeRight(maxEvents)
    subscribers.values.foreach(handler => handler(event))
  }

  def ingestBatch(events: Seq[StreamEvent]): Unit = events.foreach(ingest)

  def recent(limit: Int): Vector[StreamEvent] = stream.takeRight(limit.max(0))

  def all: Vector[StreamEvent] = stream

  def eventsByProfile(profileId: String, limit: Int): Vector[StreamEvent] = {
    stream.filter(_.profileId == profileId).takeRight(limit.max(0))
  }

  def eventsByModule(module: String, limit: Int): Vector[StreamEvent] = {
    stream.filter(_.module == module).takeRight(limit.max(0))
  }

  def eventsSince(sinceMs: Long): Vector[StreamEvent] =
    stream.filter(_.timestampMs >= sinceMs)

  def eventsByModuleSince(module: String, sinceMs: Long): Vector[StreamEvent] =
    stream.filter(e => e.module == module && e.timestampMs >= sinceMs)

  def subscribe(handler: StreamEvent => Unit): () => Unit = {
    val id = nextSubscriberId
    nextSubscriberId += 1
    subscribers = subscribers.updated(id, handler)
    () => subscribers = subscribers - id
  }

  def size: Int = stream.size
}
