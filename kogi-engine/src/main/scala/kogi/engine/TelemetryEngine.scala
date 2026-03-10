package kogi.engine

final case class DataFlowHop(
    component: String,
    stage: String,
    timestampMs: Long,
    metadata: Map[String, String] = Map.empty
)

final case class PlatformDataEnvelope(
    id: String,
    flowId: String,
    source: String,
    target: String,
    topic: String,
    payload: Map[String, String],
    timestampMs: Long,
    hops: List[DataFlowHop] = Nil
)

final case class DataFlowLedgerEntry(
    flowId: String,
    envelopeId: String,
    source: String,
    target: String,
    topic: String,
    processedBy: String,
    status: String,
    notes: String,
    timestampMs: Long
)

final case class ComponentFlowStats(
    component: String,
    ingressCount: Int,
    egressCount: Int,
    lastTopic: String,
    lastSeenMs: Long,
    avgLatencyMs: Double,
    errorCount: Int
)

final case class EngineFlowSnapshot(
    system: SystemRealtimeSnapshot,
    components: List[ComponentFlowStats],
    recentLedger: List[DataFlowLedgerEntry],
    observedTopics: List[String],
    totalEnvelopes: Int,
    generatedAtMs: Long
)

final class TelemetryEngine(
    analyticsEngine: AnalyticsEngine = new AnalyticsEngine(),
    maxEnvelopes: Int = 50000,
    maxLedger: Int = 50000
) {
  private var envelopes: Vector[PlatformDataEnvelope] = Vector.empty
  private var ledger: Vector[DataFlowLedgerEntry] = Vector.empty

  def ingestEnvelope(envelope: PlatformDataEnvelope): EngineFlowSnapshot = {
    val normalized = normalizeEnvelope(envelope)
    envelopes = (envelopes :+ normalized).takeRight(maxEnvelopes)

    val payload = normalized.payload
    val timestampMs = normalized.timestampMs
    val profileId = payload.getOrElse("profile_id", "system")
    val module = payload.getOrElse("module", resolveModule(normalized.source, normalized.topic))
    val eventType = payload.getOrElse("event_type", normalized.topic.replace('.', '_'))

    analyticsEngine.ingest(
      StreamEvent(
        id = payload.getOrElse("event_id", normalized.id),
        profileId = profileId,
        module = module,
        eventType = eventType,
        timestampMs = timestampMs,
        productivityDelta = parseDouble(payload, "productivity_delta", 0.0),
        cashFlowDelta = parseDouble(payload, "cashflow_delta", 0.0),
        collaborationDelta = parseDouble(payload, "collaboration_delta", 0.0),
        riskDelta = parseDouble(payload, "risk_delta", 0.0),
        metadata = payload ++ Map(
          "flow_id" -> normalized.flowId,
          "source" -> normalized.source,
          "target" -> normalized.target
        )
      )
    )

    if (hasModuleMetrics(payload)) {
      analyticsEngine.ingestModuleMetric(
        ModuleMetricSample(
          id = payload.getOrElse("metric_id", s"mm-${safeId(normalized.id)}-$timestampMs"),
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
    }

    if (hasHostMetrics(payload)) {
      analyticsEngine.ingestHostMetric(
        HostMetricSample(
          id = payload.getOrElse("host_metric_id", s"hm-${safeId(normalized.id)}-$timestampMs"),
          hostId = payload.getOrElse("host_id", "kogi-host-001"),
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
    }

    val flowStatus =
      if (isError(eventType)) "degraded" else "processed"
    val flowNotes = normalized.hops.map(h => s"${h.component}:${h.stage}").mkString(" -> ")
    ledger = (ledger :+ DataFlowLedgerEntry(
      flowId = normalized.flowId,
      envelopeId = normalized.id,
      source = normalized.source,
      target = normalized.target,
      topic = normalized.topic,
      processedBy = "kogi-engine",
      status = flowStatus,
      notes = flowNotes,
      timestampMs = timestampMs
    )).takeRight(maxLedger)

    snapshot(hostId = payload.getOrElse("host_id", "kogi-host-001"))
  }

  def ingestGatewayMessage(
      topic: String,
      payload: Map[String, String],
      source: String,
      target: String,
      flowId: String,
      timestampMs: Long = System.currentTimeMillis()
  ): EngineFlowSnapshot = {
    val envelope = PlatformDataEnvelope(
      id = s"env-${safeId(flowId)}-$timestampMs",
      flowId = flowId,
      source = source,
      target = target,
      topic = topic,
      payload = payload,
      timestampMs = timestampMs
    )
    ingestEnvelope(envelope)
  }

  def snapshot(
      hostId: String = "kogi-host-001",
      windowMs: Long = 5L * 60L * 1000L
  ): EngineFlowSnapshot = {
    val componentStats = buildComponentStats(windowMs)
    val system = analyticsEngine.systemSnapshot(hostId = hostId, windowMs = windowMs)
    val recentLedger = ledger.takeRight(64).toList
    val observedTopics = envelopes.takeRight(2000).map(_.topic).distinct.sorted.toList

    EngineFlowSnapshot(
      system = system,
      components = componentStats,
      recentLedger = recentLedger,
      observedTopics = observedTopics,
      totalEnvelopes = envelopes.size,
      generatedAtMs = System.currentTimeMillis()
    )
  }

  def flowLedger(limit: Int = 100): List[DataFlowLedgerEntry] =
    ledger.takeRight(limit.max(0)).toList

  def flowEnvelopes(limit: Int = 100): List[PlatformDataEnvelope] =
    envelopes.takeRight(limit.max(0)).toList

  private def normalizeEnvelope(envelope: PlatformDataEnvelope): PlatformDataEnvelope = {
    val source = normalizeComponent(envelope.source)
    val target = normalizeComponent(envelope.target)

    val hops =
      if (envelope.hops.nonEmpty) envelope.hops
      else
        List(
          DataFlowHop(
            component = source,
            stage = "ingress",
            timestampMs = envelope.timestampMs,
            metadata = Map("topic" -> envelope.topic)
          ),
          DataFlowHop(
            component = "kogi-engine",
            stage = "process",
            timestampMs = envelope.timestampMs
          ),
          DataFlowHop(
            component = target,
            stage = "egress",
            timestampMs = envelope.timestampMs
          )
        )

    envelope.copy(
      source = source,
      target = target,
      hops = hops
    )
  }

  private def buildComponentStats(windowMs: Long): List[ComponentFlowStats] = {
    val sinceMs = System.currentTimeMillis() - windowMs
    val scoped = envelopes.filter(_.timestampMs >= sinceMs)

    val components = (
      scoped.map(_.source) ++
        scoped.map(_.target) ++
        scoped.flatMap(_.hops.map(_.component))
    ).distinct.sorted

    components.map { component =>
      val ingress = scoped.count(_.target == component)
      val egress = scoped.count(_.source == component)
      val related = scoped.filter(env =>
        env.source == component ||
          env.target == component ||
          env.hops.exists(_.component == component)
      )

      val latencies = related.flatMap(env =>
        env.payload.get("latency_ms").flatMap(parseDoubleOpt)
      )
      val avgLatencyMs =
        if (latencies.isEmpty) 0.0 else latencies.sum / latencies.size.toDouble

      val errorCount = related.count(env =>
        isError(env.payload.getOrElse("event_type", env.topic))
      )
      val lastSeenMs = related.map(_.timestampMs).sorted.lastOption.getOrElse(0L)
      val lastTopic = related.lastOption.map(_.topic).getOrElse("n/a")

      ComponentFlowStats(
        component = component,
        ingressCount = ingress,
        egressCount = egress,
        lastTopic = lastTopic,
        lastSeenMs = lastSeenMs,
        avgLatencyMs = avgLatencyMs,
        errorCount = errorCount
      )
    }.toList
  }

  private def parseInt(values: Map[String, String], key: String, default: Int): Int =
    values.get(key).flatMap(value => scala.util.Try(value.toInt).toOption).getOrElse(default)

  private def parseDouble(values: Map[String, String], key: String, default: Double): Double =
    values.get(key).flatMap(value => scala.util.Try(value.toDouble).toOption).getOrElse(default)

  private def parseDoubleOpt(value: String): Option[Double] =
    scala.util.Try(value.toDouble).toOption

  private def hasModuleMetrics(payload: Map[String, String]): Boolean =
    payload.contains("latency_ms") ||
      payload.contains("queue_depth") ||
      payload.contains("cpu_pct") ||
      payload.contains("memory_mb")

  private def hasHostMetrics(payload: Map[String, String]): Boolean =
    payload.contains("host_cpu_pct") ||
      payload.contains("host_memory_pct") ||
      payload.contains("host_disk_pct") ||
      payload.contains("host_process_count")

  private def isError(value: String): Boolean = {
    val normalized = value.toLowerCase
    normalized.contains("error") || normalized.contains("fail") || normalized.contains("incident")
  }

  private def resolveModule(source: String, topic: String): String = {
    val normalizedSource = normalizeComponent(source)
    val normalizedTopic = topic.toLowerCase

    if (normalizedSource == "kogi-kernel") "kogi-kernel"
    else if (normalizedSource == "kogi-host") "kogi-host"
    else if (normalizedSource == "kogi-server") "kogi-server"
    else if (normalizedSource.startsWith("service.") || normalizedSource == "gateway")
      normalizedSource.replace("service.", "")
    else if (normalizedTopic.startsWith("kernel")) "kogi-kernel"
    else if (normalizedTopic.startsWith("host")) "kogi-host"
    else if (normalizedTopic.startsWith("server")) "kogi-server"
    else normalizedSource
  }

  private def normalizeComponent(component: String): String = {
    val normalized = component.trim.toLowerCase
    if (normalized.isEmpty) "unknown" else normalized
  }

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")
}

final class KogiPlatformDataEngine(
    analytics: AnalyticsEngine = new AnalyticsEngine(),
    maxEnvelopes: Int = 50000,
    maxLedger: Int = 50000
) {
  private val telemetry = new TelemetryEngine(
    analyticsEngine = analytics,
    maxEnvelopes = maxEnvelopes,
    maxLedger = maxLedger
  )

  def ingestEnvelope(envelope: PlatformDataEnvelope): EngineFlowSnapshot =
    telemetry.ingestEnvelope(envelope)

  def ingestGatewayMessage(
      topic: String,
      payload: Map[String, String],
      source: String,
      target: String,
      flowId: String,
      timestampMs: Long = System.currentTimeMillis()
  ): EngineFlowSnapshot =
    telemetry.ingestGatewayMessage(topic, payload, source, target, flowId, timestampMs)

  def snapshot(
      hostId: String = "kogi-host-001",
      windowMs: Long = 5L * 60L * 1000L
  ): EngineFlowSnapshot =
    telemetry.snapshot(hostId = hostId, windowMs = windowMs)

  def flowLedger(limit: Int = 100): List[DataFlowLedgerEntry] =
    telemetry.flowLedger(limit)

  def flowEnvelopes(limit: Int = 100): List[PlatformDataEnvelope] =
    telemetry.flowEnvelopes(limit)
}
