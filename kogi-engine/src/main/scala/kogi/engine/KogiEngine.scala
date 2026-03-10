package kogi.engine

final case class EngineControlStatus(
    mode: String,
    changed: Boolean,
    timestampMs: Long
)

final class KogiEngine(
    val recommendationEngine: RecommendationEngine = new RecommendationEngine(),
    val riskEngine: RiskEngine = new RiskEngine(),
    val analyticsEngine: AnalyticsEngine = new AnalyticsEngine(
      riskEngine = riskEngine,
      recommendationEngine = recommendationEngine
    ),
    val telemetryEngine: TelemetryEngine = new TelemetryEngine(analyticsEngine = analyticsEngine),
    val optimizationEngine: OptimizationEngine = new OptimizationEngine(),
    val searchEngine: SearchEngine = new SearchEngine(),
    val queryEngine: QueryEngine = new QueryEngine()
) {
  private var controlMode: String = "stopped"

  def control(action: String): EngineControlStatus = {
    val normalized = action.trim.toLowerCase
    val next =
      if (normalized == "start" || normalized == "run" || normalized == "resume") "running"
      else if (normalized == "pause") "paused"
      else if (normalized == "stop" || normalized == "shutdown") "stopped"
      else normalized

    val changed = next != controlMode
    controlMode = next
    EngineControlStatus(mode = controlMode, changed = changed, timestampMs = System.currentTimeMillis())
  }

  def status: EngineControlStatus =
    EngineControlStatus(mode = controlMode, changed = false, timestampMs = System.currentTimeMillis())

  def ingestEnvelope(envelope: PlatformDataEnvelope): EngineFlowSnapshot =
    telemetryEngine.ingestEnvelope(envelope)

  def ingestGatewayMessage(
      topic: String,
      payload: Map[String, String],
      source: String,
      target: String,
      flowId: String,
      timestampMs: Long = System.currentTimeMillis()
  ): EngineFlowSnapshot =
    telemetryEngine.ingestGatewayMessage(topic, payload, source, target, flowId, timestampMs)

  def snapshot(
      hostId: String = "kogi-host-001",
      windowMs: Long = 5L * 60L * 1000L
  ): EngineFlowSnapshot =
    telemetryEngine.snapshot(hostId = hostId, windowMs = windowMs)

  def flowLedger(limit: Int = 100): List[DataFlowLedgerEntry] =
    telemetryEngine.flowLedger(limit)

  def flowEnvelopes(limit: Int = 100): List[PlatformDataEnvelope] =
    telemetryEngine.flowEnvelopes(limit)

  def ingestEvent(event: StreamEvent): AnalyticsSnapshot =
    analyticsEngine.ingest(event)

  def ingestBusEvent(
      topic: String,
      payload: Map[String, String],
      profileId: String = "system",
      hostId: String = "kogi-host-001",
      timestampMs: Long = System.currentTimeMillis(),
      windowMs: Long = 5L * 60L * 1000L
  ): SystemRealtimeSnapshot =
    analyticsEngine.ingestBusEvent(topic, payload, profileId, hostId, timestampMs, windowMs)

  def optimize(request: OptimizationRequest): OptimizationPlan =
    optimizationEngine.optimize(request)

  def optimizeRisk(request: RiskOptimizationRequest): RiskOptimizationPlan =
    riskEngine.optimize(request)

  def search(query: SearchQuery): SearchResult =
    searchEngine.search(query)

  def index(document: SearchDocument): Unit =
    searchEngine.index(document)

  def query(sql: String): QueryPlan =
    queryEngine.optimize(sql)
}
