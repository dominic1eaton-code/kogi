package kogi.analytics

final case class StreamEvent(
    id: String,
    profileId: String,
    module: String,
    eventType: String,
    timestampMs: Long,
    productivityDelta: Double = 0.0,
    cashFlowDelta: Double = 0.0,
    collaborationDelta: Double = 0.0,
    riskDelta: Double = 0.0,
    metadata: Map[String, String] = Map.empty
)

final case class ModuleMetricSample(
    id: String,
    module: String,
    timestampMs: Long,
    latencyMs: Double,
    queueDepth: Int,
    errorCount: Int = 0,
    successCount: Int = 1,
    throughputUnits: Double = 1.0,
    cpuPct: Double = 0.0,
    memoryMb: Double = 0.0,
    metadata: Map[String, String] = Map.empty
)

final case class HostMetricSample(
    id: String,
    hostId: String,
    timestampMs: Long,
    cpuPct: Double,
    memoryPct: Double,
    diskPct: Double,
    processCount: Int,
    schedulerLoad: Double = 0.0,
    networkInKbps: Double = 0.0,
    networkOutKbps: Double = 0.0,
    metadata: Map[String, String] = Map.empty
)

final case class ModuleActivity(
    module: String,
    events: Int,
    netImpact: Double
)

final case class RecommendationCard(
    id: String,
    priority: String,
    message: String
)

final case class DiscoverCard(
    id: String,
    title: String,
    evidence: String
)

final case class ExploreCard(
    id: String,
    title: String,
    nextAction: String
)

final case class AnalyticsSnapshot(
    profileId: String,
    eventCount: Int,
    signal: PortfolioSignal,
    health: PortfolioHealthScore,
    modules: List[ModuleActivity],
    recommendations: List[RecommendationCard],
    discover: List[DiscoverCard],
    explore: List[ExploreCard],
    lastUpdatedMs: Long
)

final case class ModuleRealtimeSnapshot(
    module: String,
    eventCount: Int,
    metricCount: Int,
    throughputPerMin: Double,
    errorRate: Double,
    blockedRate: Double,
    avgLatencyMs: Double,
    p95LatencyMs: Double,
    avgQueueDepth: Double,
    avgCpuPct: Double,
    avgMemoryMb: Double,
    signal: PortfolioSignal,
    health: PortfolioHealthScore,
    recommendations: List[RecommendationCard],
    discover: List[DiscoverCard],
    explore: List[ExploreCard],
    anomalies: List[String],
    lastUpdatedMs: Long
)

final case class HostRealtimeSnapshot(
    hostId: String,
    sampleCount: Int,
    cpuAvgPct: Double,
    memoryAvgPct: Double,
    diskAvgPct: Double,
    processAvg: Double,
    schedulerLoadAvg: Double,
    netInAvgKbps: Double,
    netOutAvgKbps: Double,
    saturationScore: Double,
    status: String,
    anomalies: List[String],
    lastUpdatedMs: Long
)

final case class SystemRealtimeSnapshot(
    host: HostRealtimeSnapshot,
    modules: List[ModuleRealtimeSnapshot],
    recommendations: List[RecommendationCard],
    discover: List[DiscoverCard],
    explore: List[ExploreCard],
    generatedAtMs: Long
)

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

final class AnalyticsRecommendationDiscoverExploreEngine(
    streamEngine: DataStreamingEngine = new DataStreamingEngine(),
    defaultWindow: Int = 250,
    defaultRealtimeWindowMs: Long = 5L * 60L * 1000L,
    recommendationLimit: Int = 8,
    discoverLimit: Int = 8,
    exploreLimit: Int = 8,
    maxModuleMetrics: Int = 20000,
    maxHostMetrics: Int = 20000
) {
  private val stream = streamEngine
  private val knownModules = Set(
    "kogi-kernel",
    "kogi-host",
    "kogi-server",
    "gateway",
    "auth",
    "portfolio",
    "exchange",
    "ims",
    "office",
    "bank",
    "marketplace",
    "studio",
    "community",
    "developer",
    "profile",
    "organizations",
    "legal"
  )
  private var moduleMetrics: Vector[ModuleMetricSample] = Vector.empty
  private var hostMetrics: Vector[HostMetricSample] = Vector.empty

  def moduleCatalog(): List[String] = knownModules.toList.sorted

  def ingest(event: StreamEvent): AnalyticsSnapshot = {
    stream.ingest(event)
    snapshotForProfile(event.profileId, defaultWindow)
  }

  def ingestBatch(events: Seq[StreamEvent]): List[AnalyticsSnapshot] =
    events.toList.map(ingest)

  def ingestModuleMetric(sample: ModuleMetricSample): ModuleRealtimeSnapshot = {
    moduleMetrics = (moduleMetrics :+ sample).takeRight(maxModuleMetrics)
    moduleSnapshot(sample.module, defaultRealtimeWindowMs)
  }

  def ingestHostMetric(sample: HostMetricSample): HostRealtimeSnapshot = {
    hostMetrics = (hostMetrics :+ sample).takeRight(maxHostMetrics)
    hostSnapshot(sample.hostId, defaultRealtimeWindowMs)
  }

  def ingestRealtime(
      event: Option[StreamEvent] = None,
      moduleMetric: Option[ModuleMetricSample] = None,
      hostMetric: Option[HostMetricSample] = None,
      hostId: String = "kogi-host-001",
      windowMs: Long = defaultRealtimeWindowMs
  ): SystemRealtimeSnapshot = {
    event.foreach(stream.ingest)
    moduleMetric.foreach(sample => moduleMetrics = (moduleMetrics :+ sample).takeRight(maxModuleMetrics))
    hostMetric.foreach(sample => hostMetrics = (hostMetrics :+ sample).takeRight(maxHostMetrics))
    systemSnapshot(hostId, windowMs = windowMs)
  }

  def ingestBusEvent(
      topic: String,
      payload: Map[String, String],
      profileId: String = "system",
      hostId: String = "kogi-host-001",
      timestampMs: Long = System.currentTimeMillis(),
      windowMs: Long = defaultRealtimeWindowMs
  ): SystemRealtimeSnapshot = {
    val module = payload
      .get("module")
      .map(_.toLowerCase)
      .getOrElse(resolveModuleFromTopic(topic))
    val eventType = payload.getOrElse("event_type", topic.replace('.', '_'))

    val event = StreamEvent(
      id = payload.getOrElse("event_id", s"bus-${safeId(topic)}-$timestampMs"),
      profileId = payload.getOrElse("profile_id", profileId),
      module = module,
      eventType = eventType,
      timestampMs = timestampMs,
      productivityDelta = parseDouble(payload, "productivity_delta", 0.0),
      cashFlowDelta = parseDouble(payload, "cashflow_delta", 0.0),
      collaborationDelta = parseDouble(payload, "collaboration_delta", 0.0),
      riskDelta = parseDouble(payload, "risk_delta", 0.0),
      metadata = payload + ("topic" -> topic)
    )
    stream.ingest(event)

    val maybeModuleMetric =
      if (
        payload.contains("latency_ms") ||
        payload.contains("queue_depth") ||
        payload.contains("cpu_pct") ||
        payload.contains("memory_mb")
      ) {
        Some(
          ModuleMetricSample(
            id = payload.getOrElse("metric_id", s"mm-${safeId(topic)}-$timestampMs"),
            module = module,
            timestampMs = timestampMs,
            latencyMs = parseDouble(payload, "latency_ms", 0.0),
            queueDepth = parseInt(payload, "queue_depth", 0),
            errorCount = parseInt(payload, "error_count", 0),
            successCount = parseInt(payload, "success_count", 1),
            throughputUnits = parseDouble(payload, "throughput_units", 1.0),
            cpuPct = parseDouble(payload, "cpu_pct", 0.0),
            memoryMb = parseDouble(payload, "memory_mb", 0.0),
            metadata = payload
          )
        )
      } else None

    maybeModuleMetric.foreach(sample => moduleMetrics = (moduleMetrics :+ sample).takeRight(maxModuleMetrics))

    val maybeHostMetric =
      if (
        payload.contains("host_cpu_pct") ||
        payload.contains("host_memory_pct") ||
        payload.contains("host_disk_pct") ||
        payload.contains("host_process_count")
      ) {
        Some(
          HostMetricSample(
            id = payload.getOrElse("host_metric_id", s"hm-${safeId(topic)}-$timestampMs"),
            hostId = payload.getOrElse("host_id", hostId),
            timestampMs = timestampMs,
            cpuPct = parseDouble(payload, "host_cpu_pct", 0.0),
            memoryPct = parseDouble(payload, "host_memory_pct", 0.0),
            diskPct = parseDouble(payload, "host_disk_pct", 0.0),
            processCount = parseInt(payload, "host_process_count", 0),
            schedulerLoad = parseDouble(payload, "host_scheduler_load", 0.0),
            networkInKbps = parseDouble(payload, "host_net_in_kbps", 0.0),
            networkOutKbps = parseDouble(payload, "host_net_out_kbps", 0.0),
            metadata = payload
          )
        )
      } else None

    maybeHostMetric.foreach(sample => hostMetrics = (hostMetrics :+ sample).takeRight(maxHostMetrics))
    systemSnapshot(hostId = hostId, windowMs = windowMs)
  }

  def snapshotForProfile(profileId: String, window: Int = defaultWindow): AnalyticsSnapshot = {
    val events = stream.eventsByProfile(profileId, window)
    val signal = deriveSignal(events)
    val health = PortfolioHealthPipeline.score(signal)
    val modules = moduleActivity(events)
    val recommendations = buildRecommendations(signal, health, events, modules)
    val discover = buildDiscover(events, modules)
    val explore = buildExplore(events, modules, recommendations)

    AnalyticsSnapshot(
      profileId = profileId,
      eventCount = events.size,
      signal = signal,
      health = health,
      modules = modules,
      recommendations = recommendations.take(recommendationLimit),
      discover = discover.take(discoverLimit),
      explore = explore.take(exploreLimit),
      lastUpdatedMs = events.lastOption.map(_.timestampMs).getOrElse(System.currentTimeMillis())
    )
  }

  def globalModuleActivity(window: Int = defaultWindow): List[ModuleActivity] = {
    val events = stream.recent(window)
    moduleActivity(events)
  }

  def moduleSnapshot(
      module: String,
      windowMs: Long = defaultRealtimeWindowMs,
      nowMs: Long = System.currentTimeMillis()
  ): ModuleRealtimeSnapshot = {
    val sinceMs = nowMs - windowMs
    val events = stream.eventsByModuleSince(module, sinceMs)
    val metrics = moduleMetrics.filter(m => m.module == module && m.timestampMs >= sinceMs)

    val signal = deriveSignal(events)
    val health = PortfolioHealthPipeline.score(signal)
    val errorEvents = events.count(e => containsAny(e.eventType, "error", "failed", "incident"))
    val blockedEvents = events.count(e => containsAny(e.eventType, "blocked", "stalled", "overdue"))
    val eventCount = events.size
    val metricCount = metrics.size

    val errorRateEvents = ratio(errorEvents.toDouble, eventCount.toDouble)
    val blockedRate = ratio(blockedEvents.toDouble, eventCount.toDouble)

    val metricErrors = metrics.map(_.errorCount).sum.toDouble
    val metricTotal = metrics.map(m => m.errorCount + m.successCount).sum.toDouble
    val errorRateMetrics = ratio(metricErrors, metricTotal)
    val errorRate = if (metricTotal > 0) errorRateMetrics else errorRateEvents

    val avgLatencyMs = average(metrics.map(_.latencyMs))
    val p95LatencyMs = percentile(metrics.map(_.latencyMs), 95.0)
    val avgQueueDepth = average(metrics.map(_.queueDepth.toDouble))
    val avgCpuPct = average(metrics.map(_.cpuPct))
    val avgMemoryMb = average(metrics.map(_.memoryMb))

    val throughputFromEvents = perMinute(eventCount.toDouble, windowMs)
    val throughputFromMetrics = perMinute(metrics.map(_.throughputUnits).sum, windowMs)
    val throughputPerMin = if (metricCount > 0) throughputFromMetrics else throughputFromEvents

    val moduleActivityList = moduleActivity(events)
    val recommendations = (
      buildRecommendations(signal, health, events, moduleActivityList) ++
        moduleOperationalRecommendations(module, errorRate, p95LatencyMs, avgQueueDepth, avgCpuPct)
    )
      .groupBy(_.message)
      .values
      .map(_.head)
      .toList
      .sortBy(card => priorityRank(card.priority))
      .take(recommendationLimit)

    val discover = (
      buildDiscover(events, moduleActivityList) ++
        moduleDiscover(module, metrics)
    ).take(discoverLimit)

    val explore = (
      buildExplore(events, moduleActivityList, recommendations) ++
        moduleExplore(module, errorRate, blockedRate)
    ).take(exploreLimit)

    val anomalies = moduleAnomalies(module, errorRate, blockedRate, p95LatencyMs, avgQueueDepth, avgCpuPct, health)
    val lastUpdatedMs = (
      events.lastOption.map(_.timestampMs).toSeq ++
        metrics.lastOption.map(_.timestampMs).toSeq
    ).sorted.lastOption.getOrElse(nowMs)

    ModuleRealtimeSnapshot(
      module = module,
      eventCount = eventCount,
      metricCount = metricCount,
      throughputPerMin = throughputPerMin,
      errorRate = errorRate,
      blockedRate = blockedRate,
      avgLatencyMs = avgLatencyMs,
      p95LatencyMs = p95LatencyMs,
      avgQueueDepth = avgQueueDepth,
      avgCpuPct = avgCpuPct,
      avgMemoryMb = avgMemoryMb,
      signal = signal,
      health = health,
      recommendations = recommendations,
      discover = discover,
      explore = explore,
      anomalies = anomalies,
      lastUpdatedMs = lastUpdatedMs
    )
  }

  def hostSnapshot(
      hostId: String,
      windowMs: Long = defaultRealtimeWindowMs,
      nowMs: Long = System.currentTimeMillis()
  ): HostRealtimeSnapshot = {
    val sinceMs = nowMs - windowMs
    val samples = hostMetrics.filter(s => s.hostId == hostId && s.timestampMs >= sinceMs)

    val cpuAvg = average(samples.map(_.cpuPct))
    val memoryAvg = average(samples.map(_.memoryPct))
    val diskAvg = average(samples.map(_.diskPct))
    val processAvg = average(samples.map(_.processCount.toDouble))
    val schedulerAvg = average(samples.map(_.schedulerLoad))
    val netInAvg = average(samples.map(_.networkInKbps))
    val netOutAvg = average(samples.map(_.networkOutKbps))

    val schedulerPct = clamp(schedulerAvg * 100.0)
    val saturation = clamp(cpuAvg * 0.35 + memoryAvg * 0.30 + diskAvg * 0.15 + schedulerPct * 0.20)
    val status =
      if (saturation >= 80.0) "red"
      else if (saturation >= 60.0) "amber"
      else "green"

    val anomalies = List(
      if (cpuAvg >= 90.0) Some("host cpu saturation critical") else None,
      if (memoryAvg >= 90.0) Some("host memory saturation critical") else None,
      if (diskAvg >= 90.0) Some("host disk saturation critical") else None,
      if (schedulerAvg >= 0.95) Some("host scheduler load critical") else None,
      if (processAvg >= 5000) Some("host process count unusually high") else None
    ).flatten

    HostRealtimeSnapshot(
      hostId = hostId,
      sampleCount = samples.size,
      cpuAvgPct = cpuAvg,
      memoryAvgPct = memoryAvg,
      diskAvgPct = diskAvg,
      processAvg = processAvg,
      schedulerLoadAvg = schedulerAvg,
      netInAvgKbps = netInAvg,
      netOutAvgKbps = netOutAvg,
      saturationScore = saturation,
      status = status,
      anomalies = anomalies,
      lastUpdatedMs = samples.lastOption.map(_.timestampMs).getOrElse(nowMs)
    )
  }

  def systemSnapshot(
      hostId: String = "kogi-host-001",
      modules: Seq[String] = Seq.empty,
      windowMs: Long = defaultRealtimeWindowMs,
      nowMs: Long = System.currentTimeMillis()
  ): SystemRealtimeSnapshot = {
    val observed = observedModules(nowMs - windowMs)
    val targetModules = if (modules.nonEmpty) modules.distinct else observed

    val moduleSnapshots = targetModules
      .map(m => moduleSnapshot(m, windowMs, nowMs))
      .toList
      .sortBy(snapshot => (priorityRank(snapshot.health.status), -snapshot.eventCount))

    val host = hostSnapshot(hostId, windowMs, nowMs)

    val recommendations = buildSystemRecommendations(host, moduleSnapshots).take(recommendationLimit)
    val discover = buildSystemDiscover(host, moduleSnapshots).take(discoverLimit)
    val explore = buildSystemExplore(host, moduleSnapshots).take(exploreLimit)

    SystemRealtimeSnapshot(
      host = host,
      modules = moduleSnapshots,
      recommendations = recommendations,
      discover = discover,
      explore = explore,
      generatedAtMs = nowMs
    )
  }

  private def deriveSignal(events: Seq[StreamEvent]): PortfolioSignal = {
    val completedCount = events.count(e => containsAny(e.eventType, "complete", "closed", "done"))
    val blockedCount = events.count(e => containsAny(e.eventType, "blocked", "stalled", "overdue"))
    val errorCount = events.count(e => containsAny(e.eventType, "error", "failed", "incident"))
    val socialCount = events.count(e => containsAny(e.eventType, "message", "chat", "comment", "review"))
    val cashInCount = events.count(e => containsAny(e.eventType, "payment_received", "invoice_paid", "deal_won"))
    val cashOutCount = events.count(e => containsAny(e.eventType, "payment_failed", "chargeback", "refund"))

    val productivity = clamp(
      55.0 + events.map(_.productivityDelta).sum +
        completedCount * 1.6 -
        blockedCount * 2.2 -
        errorCount * 1.2
    )

    val cashFlow = clamp(
      55.0 + events.map(_.cashFlowDelta).sum +
        cashInCount * 2.0 -
        cashOutCount * 2.6
    )

    val collaboration = clamp(
      58.0 + events.map(_.collaborationDelta).sum + socialCount * 0.9
    )

    val risk = clamp(
      42.0 + events.map(_.riskDelta).sum +
        blockedCount * 1.3 +
        errorCount * 2.4 -
        completedCount * 0.5
    )

    PortfolioSignal(
      productivity = productivity,
      cashFlow = cashFlow,
      collaboration = collaboration,
      risk = risk
    )
  }

  private def moduleActivity(events: Seq[StreamEvent]): List[ModuleActivity] = {
    events
      .groupBy(_.module)
      .view
      .map { case (module, grouped) =>
        val netImpact = grouped.map(eventImpact).sum
        ModuleActivity(module = module, events = grouped.size, netImpact = netImpact)
      }
      .toList
      .sortBy(activity => (-activity.events, -activity.netImpact))
  }

  private def buildRecommendations(
      signal: PortfolioSignal,
      health: PortfolioHealthScore,
      events: Seq[StreamEvent],
      modules: List[ModuleActivity]
  ): List[RecommendationCard] = {
    val buffer = scala.collection.mutable.ListBuffer.empty[RecommendationCard]

    if (signal.risk > 65.0) {
      buffer += RecommendationCard(
        id = "rec-risk-001",
        priority = "high",
        message = "Reduce risk exposure by triaging blocked/failed events in office and exchange modules."
      )
    }

    if (signal.cashFlow < 50.0) {
      buffer += RecommendationCard(
        id = "rec-cashflow-001",
        priority = "high",
        message = "Prioritize paid backlog work and recovery actions for wallet/campaign pipelines."
      )
    }

    if (signal.productivity < 50.0) {
      buffer += RecommendationCard(
        id = "rec-productivity-001",
        priority = "medium",
        message = "Rebalance sprint workload and reduce WIP across active workspace lanes."
      )
    }

    if (signal.collaboration < 50.0) {
      buffer += RecommendationCard(
        id = "rec-collab-001",
        priority = "medium",
        message = "Increase team sync cadence and direct-message response coverage."
      )
    }

    val lowActivityModules = modules.filter(_.events < 2).map(_.module)
    if (lowActivityModules.nonEmpty) {
      buffer += RecommendationCard(
        id = "rec-coverage-001",
        priority = "low",
        message = s"Low activity detected in modules: ${lowActivityModules.mkString(", ")}; verify sync/integration coverage."
      )
    }

    health.recommendations.zipWithIndex.foreach { case (msg, idx) =>
      buffer += RecommendationCard(
        id = f"rec-health-${idx + 1}%03d",
        priority = "medium",
        message = msg
      )
    }

    val staleEventCount = events.count(e => containsAny(e.eventType, "stale", "timeout"))
    if (staleEventCount > 0) {
      buffer += RecommendationCard(
        id = "rec-stream-001",
        priority = "medium",
        message = s"Detected $staleEventCount stale/timeout events; inspect streaming ingest latency."
      )
    }

    buffer.toList
      .groupBy(_.message)
      .values
      .map(_.head)
      .toList
      .sortBy(card => priorityRank(card.priority))
  }

  private def buildDiscover(
      events: Seq[StreamEvent],
      modules: List[ModuleActivity]
  ): List[DiscoverCard] = {
    if (events.isEmpty) {
      return List(
        DiscoverCard(
          id = "discover-empty-001",
          title = "No Stream Activity",
          evidence = "No events in window. Start ingesting module telemetry."
        )
      )
    }

    val providers = events.flatMap(_.metadata.get("provider")).distinct.sorted
    val dominant = modules.headOption
    val topEventTypes = events.groupBy(_.eventType).toList.sortBy(-_._2.size).take(3)

    val cards = scala.collection.mutable.ListBuffer.empty[DiscoverCard]

    dominant.foreach { module =>
      cards += DiscoverCard(
        id = "discover-module-001",
        title = s"${module.module} is the dominant activity module",
        evidence = f"${module.events} events with net impact ${module.netImpact}%.2f"
      )
    }

    if (providers.nonEmpty) {
      cards += DiscoverCard(
        id = "discover-provider-001",
        title = "Connected providers discovered",
        evidence = providers.mkString(", ")
      )
    }

    topEventTypes.foreach { case (eventType, grouped) =>
      cards += DiscoverCard(
        id = s"discover-event-${safeId(eventType)}",
        title = s"Trending event: $eventType",
        evidence = s"${grouped.size} events in current window"
      )
    }

    cards.toList
  }

  private def buildExplore(
      events: Seq[StreamEvent],
      modules: List[ModuleActivity],
      recommendations: List[RecommendationCard]
  ): List[ExploreCard] = {
    val cards = scala.collection.mutable.ListBuffer.empty[ExploreCard]
    val activeModules = modules.map(_.module).toSet

    cards += ExploreCard(
      id = "explore-portfolio-001",
      title = "Explore portfolio item concentration",
      nextAction = "Compare top module activity against portfolio risk and capital allocation."
    )

    if (activeModules.contains("office") && !activeModules.contains("exchange")) {
      cards += ExploreCard(
        id = "explore-exchange-001",
        title = "Office-heavy, exchange-light activity",
        nextAction = "Explore exchange integrations for capital and deal flow automation."
      )
    }

    if (activeModules.contains("community") && !activeModules.contains("marketplace")) {
      cards += ExploreCard(
        id = "explore-marketplace-001",
        title = "Community activity can convert to marketplace",
        nextAction = "Explore publishing community demand signals as marketplace listings."
      )
    }

    if (recommendations.exists(_.priority == "high")) {
      cards += ExploreCard(
        id = "explore-stabilize-001",
        title = "High-priority risk or cashflow actions detected",
        nextAction = "Run stabilization playbook before expanding experimentation lanes."
      )
    }

    val innovationSignals = events.count(e => containsAny(e.eventType, "prototype", "experiment", "idea"))
    if (innovationSignals > 0) {
      cards += ExploreCard(
        id = "explore-innovation-001",
        title = "Innovation stream detected",
        nextAction = s"Promote top $innovationSignals prototype/idea events into studio-to-office execution pipeline."
      )
    }

    cards.toList
  }

  private def moduleOperationalRecommendations(
      module: String,
      errorRate: Double,
      p95LatencyMs: Double,
      avgQueueDepth: Double,
      avgCpuPct: Double
  ): List[RecommendationCard] = {
    List(
      if (errorRate >= 0.15)
        Some(
          RecommendationCard(
            id = s"rec-${safeId(module)}-error",
            priority = "high",
            message = s"$module error rate is elevated; trigger incident playbook and inspect failing routes."
          )
        )
      else None,
      if (p95LatencyMs >= 1200.0)
        Some(
          RecommendationCard(
            id = s"rec-${safeId(module)}-latency",
            priority = "high",
            message = s"$module latency p95 is above threshold; scale workers and optimize slow handlers."
          )
        )
      else None,
      if (avgQueueDepth >= 150.0)
        Some(
          RecommendationCard(
            id = s"rec-${safeId(module)}-queue",
            priority = "medium",
            message = s"$module queue depth is growing; increase consumer concurrency or apply backpressure."
          )
        )
      else None,
      if (avgCpuPct >= 80.0)
        Some(
          RecommendationCard(
            id = s"rec-${safeId(module)}-cpu",
            priority = "medium",
            message = s"$module compute utilization is high; evaluate horizontal scaling."
          )
        )
      else None
    ).flatten
  }

  private def moduleDiscover(
      module: String,
      metrics: Seq[ModuleMetricSample]
  ): List[DiscoverCard] = {
    if (metrics.isEmpty) return Nil

    val providers = metrics.flatMap(_.metadata.get("provider")).distinct
    val routes = metrics.flatMap(_.metadata.get("route")).groupBy(identity).view.mapValues(_.size).toMap
    val hottestRoute = routes.toList.sortBy(-_._2).headOption

    val cards = scala.collection.mutable.ListBuffer.empty[DiscoverCard]
    cards += DiscoverCard(
      id = s"discover-${safeId(module)}-perf",
      title = s"$module realtime telemetry available",
      evidence = f"${metrics.size} metric samples with avg latency ${average(metrics.map(_.latencyMs))}%.2f ms"
    )

    if (providers.nonEmpty) {
      cards += DiscoverCard(
        id = s"discover-${safeId(module)}-providers",
        title = s"$module provider coverage",
        evidence = providers.sorted.mkString(", ")
      )
    }

    hottestRoute.foreach { case (route, count) =>
      cards += DiscoverCard(
        id = s"discover-${safeId(module)}-route",
        title = s"$module hottest route",
        evidence = s"$route ($count samples)"
      )
    }

    cards.toList
  }

  private def moduleExplore(
      module: String,
      errorRate: Double,
      blockedRate: Double
  ): List[ExploreCard] = {
    val cards = scala.collection.mutable.ListBuffer.empty[ExploreCard]

    cards += ExploreCard(
      id = s"explore-${safeId(module)}-optimize",
      title = s"Explore $module optimization lane",
      nextAction = s"Open $module performance board and test one throughput optimization experiment."
    )

    if (errorRate >= 0.10 || blockedRate >= 0.12) {
      cards += ExploreCard(
        id = s"explore-${safeId(module)}-stability",
        title = s"Explore $module stabilization lane",
        nextAction = s"Correlate $module incidents with upstream dependencies (kernel/host/server/gateway)."
      )
    }

    cards.toList
  }

  private def buildSystemRecommendations(
      host: HostRealtimeSnapshot,
      modules: Seq[ModuleRealtimeSnapshot]
  ): List[RecommendationCard] = {
    val cards = scala.collection.mutable.ListBuffer.empty[RecommendationCard]

    if (host.status == "red") {
      cards += RecommendationCard(
        id = "sys-rec-host-red",
        priority = "high",
        message = "Host saturation is critical; scale host resources and rebalance module workloads."
      )
    } else if (host.status == "amber") {
      cards += RecommendationCard(
        id = "sys-rec-host-amber",
        priority = "medium",
        message = "Host is trending hot; tune scheduler/process limits and watch memory pressure."
      )
    }

    val redModules = modules.filter(_.health.status == "red").map(_.module)
    if (redModules.nonEmpty) {
      cards += RecommendationCard(
        id = "sys-rec-module-red",
        priority = "high",
        message = s"Critical module health detected in: ${redModules.mkString(", ")}."
      )
    }

    val latencyModules = modules.filter(_.p95LatencyMs >= 1200.0).map(_.module)
    if (latencyModules.nonEmpty) {
      cards += RecommendationCard(
        id = "sys-rec-latency",
        priority = "high",
        message = s"High p95 latency in modules: ${latencyModules.mkString(", ")}."
      )
    }

    val queueModules = modules.filter(_.avgQueueDepth >= 150.0).map(_.module)
    if (queueModules.nonEmpty) {
      cards += RecommendationCard(
        id = "sys-rec-queue",
        priority = "medium",
        message = s"Backpressure risk in modules: ${queueModules.mkString(", ")}."
      )
    }

    cards.toList
      .groupBy(_.message)
      .values
      .map(_.head)
      .toList
      .sortBy(card => priorityRank(card.priority))
  }

  private def buildSystemDiscover(
      host: HostRealtimeSnapshot,
      modules: Seq[ModuleRealtimeSnapshot]
  ): List[DiscoverCard] = {
    val cards = scala.collection.mutable.ListBuffer.empty[DiscoverCard]

    cards += DiscoverCard(
      id = "sys-discover-host",
      title = s"Host ${host.hostId} status ${host.status}",
      evidence = f"cpu=${host.cpuAvgPct}%.1f%% mem=${host.memoryAvgPct}%.1f%% disk=${host.diskAvgPct}%.1f%% saturation=${host.saturationScore}%.1f"
    )

    modules.sortBy(-_.throughputPerMin).headOption.foreach { top =>
      cards += DiscoverCard(
        id = "sys-discover-throughput",
        title = s"${top.module} is the throughput leader",
        evidence = f"${top.throughputPerMin}%.2f units/min in current window"
      )
    }

    modules.sortBy(-_.errorRate).headOption.filter(_.errorRate > 0).foreach { top =>
      cards += DiscoverCard(
        id = "sys-discover-error",
        title = s"${top.module} has highest error rate",
        evidence = f"${top.errorRate * 100.0}%.2f%% error rate"
      )
    }

    cards.toList
  }

  private def buildSystemExplore(
      host: HostRealtimeSnapshot,
      modules: Seq[ModuleRealtimeSnapshot]
  ): List[ExploreCard] = {
    val cards = scala.collection.mutable.ListBuffer.empty[ExploreCard]
    val moduleSet = modules.map(_.module).toSet

    cards += ExploreCard(
      id = "sys-explore-capacity",
      title = "Explore capacity planning model",
      nextAction = "Use module throughput and host saturation to simulate scale-up/scale-out plans."
    )

    if (moduleSet.contains("kogi-kernel") || moduleSet.contains("kernel")) {
      cards += ExploreCard(
        id = "sys-explore-kernel",
        title = "Explore kernel scheduler instrumentation",
        nextAction = "Correlate kernel queue depth and dispatch latency with module p95 latency."
      )
    }

    if (moduleSet.contains("kogi-host") || moduleSet.contains("host")) {
      cards += ExploreCard(
        id = "sys-explore-host",
        title = "Explore host orchestration optimization",
        nextAction = "Tune module isolation quotas based on observed cpu/memory contention."
      )
    }

    if (host.status != "green") {
      cards += ExploreCard(
        id = "sys-explore-resilience",
        title = "Explore resilience hardening",
        nextAction = "Enable failover routing and workload shedding policies for peak periods."
      )
    }

    cards.toList
  }

  private def moduleAnomalies(
      module: String,
      errorRate: Double,
      blockedRate: Double,
      p95LatencyMs: Double,
      avgQueueDepth: Double,
      avgCpuPct: Double,
      health: PortfolioHealthScore
  ): List[String] = {
    List(
      if (errorRate >= 0.20) Some(s"$module anomaly: error rate critical") else None,
      if (blockedRate >= 0.20) Some(s"$module anomaly: blocked flow critical") else None,
      if (p95LatencyMs >= 1500.0) Some(s"$module anomaly: latency critical") else None,
      if (avgQueueDepth >= 200.0) Some(s"$module anomaly: queue depth critical") else None,
      if (avgCpuPct >= 90.0) Some(s"$module anomaly: cpu saturation critical") else None,
      if (health.status == "red") Some(s"$module anomaly: health red") else None
    ).flatten
  }

  private def observedModules(sinceMs: Long): Seq[String] = {
    val streamModules = stream.eventsSince(sinceMs).map(_.module)
    val metricModules = moduleMetrics.filter(_.timestampMs >= sinceMs).map(_.module)
    (streamModules ++ metricModules).distinct.sorted
  }

  private def eventImpact(event: StreamEvent): Double =
    event.productivityDelta + event.cashFlowDelta + event.collaborationDelta - event.riskDelta

  private def containsAny(value: String, parts: String*): Boolean = {
    val normalized = value.toLowerCase
    parts.exists(normalized.contains)
  }

  private def clamp(value: Double): Double = math.max(0.0, math.min(100.0, value))

  private def ratio(numerator: Double, denominator: Double): Double =
    if (denominator <= 0.0) 0.0 else numerator / denominator

  private def average(values: Seq[Double]): Double =
    if (values.isEmpty) 0.0 else values.sum / values.size.toDouble

  private def percentile(values: Seq[Double], p: Double): Double = {
    if (values.isEmpty) return 0.0
    val sorted = values.sorted
    val rank = math.ceil((p / 100.0) * sorted.size.toDouble).toInt.max(1).min(sorted.size)
    sorted(rank - 1)
  }

  private def perMinute(units: Double, windowMs: Long): Double = {
    if (windowMs <= 0L) units
    else units * (60000.0 / windowMs.toDouble)
  }

  private def priorityRank(priority: String): Int = priority match {
    case "high" | "red"   => 0
    case "medium" | "amber" => 1
    case _                  => 2
  }

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")

  private def parseDouble(values: Map[String, String], key: String, default: Double): Double =
    values.get(key).flatMap(value => scala.util.Try(value.toDouble).toOption).getOrElse(default)

  private def parseInt(values: Map[String, String], key: String, default: Int): Int =
    values.get(key).flatMap(value => scala.util.Try(value.toInt).toOption).getOrElse(default)

  private def resolveModuleFromTopic(topic: String): String = {
    val normalized = topic.toLowerCase
    val tokens = normalized.split("[./:_-]").toList.filter(_.nonEmpty)
    val tokenModule = tokens.find(knownModules.contains)

    tokenModule
      .orElse {
        if (normalized.startsWith("gateway")) Some("gateway")
        else if (normalized.startsWith("kernel")) Some("kogi-kernel")
        else if (normalized.startsWith("host")) Some("kogi-host")
        else if (normalized.startsWith("server")) Some("kogi-server")
        else None
      }
      .getOrElse(tokens.headOption.getOrElse("system"))
  }
}
