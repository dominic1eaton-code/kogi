package kogi.engine

final case class EngineControlStatus(
    mode: String,
    changed: Boolean,
    timestampMs: Long
)

final class KogiEngine(
    val recommendationEngine: RecommendationEngine = new RecommendationEngine(),
    val riskEngine: RiskEngine = new RiskEngine(),
    analyticsEngineOverride: AnalyticsEngine = null,
    telemetryEngineOverride: TelemetryEngine = null,
    val optimizationEngine: OptimizationEngine = new OptimizationEngine(),
    val searchEngine: SearchEngine = new SearchEngine(),
    val queryEngine: QueryEngine = new QueryEngine(),
    val matchEngine: MatchEngine = new MatchEngine(),
    initialGraphEdges: Seq[GraphEdge] = Seq.empty,
    initialGraphNodes: Seq[GraphNode] = Seq.empty
) {
  val analyticsEngine: AnalyticsEngine =
    if (analyticsEngineOverride != null) analyticsEngineOverride
    else new AnalyticsEngine(
      riskEngine = riskEngine,
      recommendationEngine = recommendationEngine
    )

  val telemetryEngine: TelemetryEngine =
    if (telemetryEngineOverride != null) telemetryEngineOverride
    else new TelemetryEngine(analyticsEngine = analyticsEngine)

  private var controlMode: String = "stopped"

  // ------------------------------------------------------------------
  // GraphEngine – mutable; rebuilt in-place when edges/nodes are added.
  // Use graphSnapshot() to capture a point-in-time copy for diffing.
  // ------------------------------------------------------------------

  private var _graphEdges: Seq[GraphEdge] = initialGraphEdges
  private var _graphNodes: Seq[GraphNode] = initialGraphNodes
  private var _graph: GraphEngine         = new GraphEngine(_graphEdges, _graphNodes)

  def graphEngine: GraphEngine = _graph

  // ---- Mutators (return updated engine for chaining / observation) ----

  def graphAddEdge(
      from:     String,
      to:       String,
      edgeType: EdgeType,
      weight:   Double = 1.0
  ): GraphEngine = {
    _graphEdges = _graphEdges :+ GraphEdge(from, to, edgeType, weight)
    _graph      = new GraphEngine(_graphEdges, _graphNodes)
    _graph
  }

  def graphAddNode(id: String, duration: Double = 0.0): GraphEngine = {
    _graphNodes = _graphNodes :+ GraphNode(id, duration)
    _graph      = new GraphEngine(_graphEdges, _graphNodes)
    _graph
  }

  def graphRemoveEdge(from: String, to: String): GraphEngine = {
    _graphEdges = _graphEdges.filterNot(e => e.from == from && e.to == to)
    _graph      = new GraphEngine(_graphEdges, _graphNodes)
    _graph
  }

  def graphRemoveNode(id: String): GraphEngine = {
    _graphEdges = _graphEdges.filterNot(e => e.from == id || e.to == id)
    _graphNodes = _graphNodes.filterNot(_.id == id)
    _graph      = new GraphEngine(_graphEdges, _graphNodes)
    _graph
  }

  /** Replace the entire graph in one atomic call. */
  def graphLoad(edges: Seq[GraphEdge], nodes: Seq[GraphNode] = Seq.empty): GraphEngine = {
    _graphEdges = edges
    _graphNodes = nodes
    _graph      = new GraphEngine(_graphEdges, _graphNodes)
    _graph
  }

  // ---- Snapshot (immutable copy for later diffing) ----

  def graphSnapshot(): GraphEngine = new GraphEngine(_graphEdges, _graphNodes)

  // ---- Traversal queries ----

  def graphImpact(nodeId: String): ImpactReport =
    _graph.impactAnalysis(nodeId)

  def graphDependencyClosure(nodeId: String): Set[String] =
    _graph.dependencyClosure(nodeId)

  def graphReverseClosure(nodeId: String): Set[String] =
    _graph.reverseDependencyClosure(nodeId)

  def graphNeighborhood(nodeId: String): Set[String] =
    _graph.neighborhood(nodeId)

  def graphDescendants(nodeId: String): Set[String] =
    _graph.descendants(nodeId)

  def graphAncestors(nodeId: String): Set[String] =
    _graph.ancestors(nodeId)

  // ---- Analysis ----

  def graphCycles(
      edgeFilter: EdgeType => Boolean = _ => true
  ): CycleReport =
    _graph.detectCycles(edgeFilter)

  def graphTopologicalSort(
      edgeFilter: EdgeType => Boolean = _ == Dependency
  ): Either[Set[String], Seq[String]] =
    _graph.topologicalSort(edgeFilter)

  def graphCriticalPath(
      edgeFilter: EdgeType => Boolean = _ == Dependency
  ): Option[CriticalPathReport] =
    _graph.criticalPath(edgeFilter)

  /** Diff the current live graph against a previously captured snapshot. */
  def graphDiff(snapshot: GraphEngine): GraphDiff =
    snapshot.diff(_graph)

  // ---- Introspection ----

  def graphNodes: Set[String]      = _graph.allNodeIds
  def graphEdges: Seq[GraphEdge]   = _graph.edges

  // ==================================================================
  // Recommendation Engine – Personalization & Profiling API
  // ==================================================================

  /** Register or update an item in the recommendation catalogue. */
  def recUpsertItem(item: ItemProfile): Unit =
    recommendationEngine.upsertItem(item)

  def recUpsertItems(batch: Seq[ItemProfile]): Unit =
    recommendationEngine.upsertItems(batch)

  /** Register or update a user profile (demographics, preferences). */
  def recUpsertProfile(profile: UserProfile): Unit =
    recommendationEngine.upsertProfile(profile)

  // ---- Data gathering ----

  def recRecordInteraction(interaction: UserInteraction): UserProfile =
    recommendationEngine.recordInteraction(interaction)

  def recRecordRating(userId: String, itemId: String, rating: Double,
                      context: InteractionContext = InteractionContext()): UserProfile =
    recommendationEngine.recordRating(userId, itemId, rating, context)

  def recRecordReview(userId: String, itemId: String, sentiment: Double,
                      context: InteractionContext = InteractionContext()): UserProfile =
    recommendationEngine.recordReview(userId, itemId, sentiment, context)

  def recRecordDwell(userId: String, itemId: String, dwellSeconds: Double,
                     context: InteractionContext = InteractionContext()): UserProfile =
    recommendationEngine.recordDwell(userId, itemId, dwellSeconds, context)

  def recRecordSearch(userId: String, query: String,
                      context: InteractionContext = InteractionContext()): UserProfile =
    recommendationEngine.recordSearch(userId, query, context)

  // ---- Persona ----

  def recBuildPersona(userId: String): PersonaBuildResult =
    recommendationEngine.buildPersona(userId)

  def recGetProfile(userId: String): Option[UserProfile] =
    recommendationEngine.getProfile(userId)

  def recAllPersonas(): Map[String, String] =
    recommendationEngine.allPersonas()

  // ---- Recommendation methods ----

  def recHybrid(userId: String,
                context: InteractionContext = InteractionContext(),
                exclude: Set[String] = Set.empty,
                limit: Int = 8): PersonalizedRecommendationResult =
    recommendationEngine.hybridRecommendations(userId, context, exclude, limit)

  def recCollaborative(userId: String,
                       exclude: Set[String] = Set.empty,
                       limit: Int = 8): List[PersonalizedRecommendation] =
    recommendationEngine.collaborativeRecommendations(userId, exclude, limit)

  def recContentBased(userId: String,
                      exclude: Set[String] = Set.empty,
                      limit: Int = 8): List[PersonalizedRecommendation] =
    recommendationEngine.contentBasedRecommendations(userId, exclude, limit)

  def recContextual(userId: String,
                    context: InteractionContext,
                    limit: Int = 8): PersonalizedRecommendationResult =
    recommendationEngine.contextualRecommendations(userId, context, limit)

  def recColdStart(userId: String,
                   context: InteractionContext = InteractionContext(),
                   limit: Int = 8): List[PersonalizedRecommendation] =
    recommendationEngine.coldStartRecommendations(userId, context, Set.empty, limit)

  // ---- Feedback & history ----

  def recFeedbackHistory(userId: String, limit: Int = 50): Seq[FeedbackRecord] =
    recommendationEngine.feedbackHistory(userId, limit)

  def recProfileSummary(userId: String): Map[String, Any] =
    recommendationEngine.profileSummary(userId)

  // ---- Personalized search bridge ----

  def recPersonalizedSearch(userId: String, query: SearchQuery): SearchResult = {
    val profile = recommendationEngine.getProfile(userId)
    val pq = PersonalizedSearchQuery(
      query = query,
      userId = userId,
      preferenceVector     = profile.map(_.preferenceVector).getOrElse(Map.empty),
      dominantCategories   = profile.map(_.dominantCategories).getOrElse(Nil)
    )
    searchEngine.personalizedSearch(pq)
  }

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
    telemetryEngine.ingestGatewayMessage(
      topic, payload, source, target, flowId, timestampMs,
      recommendationEngine = Some(recommendationEngine)
    )

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

  def matchProfilesToResources(profiles: Seq[Profile], resources: Seq[Resource]): Map[Profile, Seq[Resource]] = {
    matchEngine.matchProfilesToResources(profiles, resources)
  }
}
