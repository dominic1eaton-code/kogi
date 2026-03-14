package kogi.engine

object EngineModelBuilders {
  import EngineRequestReader._

  private def personaFrom(raw: String): PersonaLabel =
    PersonaLabel.fromString(raw).getOrElse(UnknownPersona)

  private def userRoleFrom(raw: String): UserRole = UserRole.fromString(raw)
  private def componentKindFrom(raw: String): ComponentKind = ComponentKind.fromString(raw)

  private def edgeTypeFrom(raw: String): EdgeType = raw.trim.toLowerCase match {
    case "hierarchy" => Hierarchy
    case "relationship" => Relationship
    case _ => Dependency
  }

  private def listingStatusFrom(raw: String): GameListingStatus = raw.trim.toLowerCase match {
    case "closed" => ListingClosed
    case "filled" => ListingFilled
    case "suspended" => ListingSuspended
    case _ => ListingOpen
  }

  private def bidStatusFrom(raw: String): BidStatus = raw.trim.toLowerCase match {
    case "withdrawn" => BidWithdrawn
    case "accepted" => BidAccepted
    case "rejected" => BidRejected
    case "expired" => BidExpired
    case _ => BidActive
  }

  private def signalFrom(map: Map[String, Any]): PortfolioSignal =
    PortfolioSignal(
      productivity = readDouble(map, Seq("productivity", "productivity_delta"), 0.0),
      cashFlow = readDouble(map, Seq("cash_flow", "cashflow", "cashflow_delta"), 0.0),
      collaboration = readDouble(map, Seq("collaboration", "collaboration_delta"), 0.0),
      risk = readDouble(map, Seq("risk", "risk_delta"), 0.0)
    )
  def buildPortfolioSignal(map: Map[String, Any]): PortfolioSignal =
    signalFrom(map)



  private def doubleMap(map: Map[String, Any]): Map[String, Double] =
    map.map { case (k, v) => k -> asDoubleOpt(v).getOrElse(0.0) }

  private def intMap(map: Map[String, Any]): Map[String, Int] =
    map.map { case (k, v) => k -> asIntOpt(v).getOrElse(0) }

  private def seqMap(map: Map[String, Any]): Map[String, Seq[String]] =
    map.map { case (k, v) => k -> asStringList(v) }

  def buildInteractionContext(map: Map[String, Any]): InteractionContext =
    InteractionContext(
      device = readString(map, Seq("device"), "unknown"),
      location = readString(map, Seq("location", "region"), "unknown"),
      timeOfDay = readString(map, Seq("time_of_day", "timeOfDay"), "unknown"),
      referrer = readString(map, Seq("referrer"), ""),
      metadata = readStringMap(map, Seq("metadata", "meta"))
    )

  def buildUserInteraction(map: Map[String, Any]): UserInteraction = {
    val ctxMap = readMap(map, Seq("context"))
    val ctx = if (ctxMap.nonEmpty) buildInteractionContext(ctxMap) else buildInteractionContext(map)
    UserInteraction(
      id = readString(map, Seq("id"), s"ui-${System.currentTimeMillis()}"),
      userId = readString(map, Seq("user_id", "userId"), ""),
      itemId = readString(map, Seq("item_id", "itemId"), ""),
      interactionType = InteractionType.fromString(readString(map, Seq("interaction_type", "type", "interaction"), "view")),
      value = readDouble(map, Seq("value"), 1.0),
      sessionId = readString(map, Seq("session_id", "sessionId"), ""),
      timestampMs = readLong(map, Seq("timestamp_ms", "timestampMs"), System.currentTimeMillis()),
      context = ctx
    )
  }

  def buildItemProfile(map: Map[String, Any]): ItemProfile =
    ItemProfile(
      id = readString(map, Seq("id"), ""),
      title = readString(map, Seq("title"), ""),
      category = readString(map, Seq("category"), "general"),
      tags = readStringList(map, Seq("tags", "tag")),
      attributes = readStringMap(map, Seq("attributes", "attrs")),
      popularityScore = readDouble(map, Seq("popularity_score", "popularity"), 0.0),
      createdMs = readLong(map, Seq("created_ms", "createdAtMs"), System.currentTimeMillis())
    )

  def buildUserProfile(map: Map[String, Any]): UserProfile = {
    val prefVec = doubleMap(readMap(map, Seq("preference_vector", "preferences")))
    val breakdown = intMap(readMap(map, Seq("interaction_breakdown")))
    UserProfile(
      id = readString(map, Seq("id", "user_id", "userId"), ""),
      demographics = readStringMap(map, Seq("demographics")),
      preferenceVector = prefVec,
      persona = personaFrom(readString(map, Seq("persona"), "newcomer")),
      personaTraits = readStringList(map, Seq("persona_traits", "traits")),
      personaConfidence = readDouble(map, Seq("persona_confidence"), 0.0),
      totalInteractions = readInt(map, Seq("total_interactions"), 0),
      interactionBreakdown = breakdown,
      dominantCategories = readStringList(map, Seq("dominant_categories")),
      preferredDevices = readStringList(map, Seq("preferred_devices")),
      preferredTimeOfDay = readStringList(map, Seq("preferred_time_of_day", "preferred_time")),
      createdMs = readLong(map, Seq("created_ms"), System.currentTimeMillis()),
      lastSeenMs = readLong(map, Seq("last_seen_ms"), System.currentTimeMillis())
    )
  }

  def buildSearchQuery(map: Map[String, Any]): SearchQuery =
    SearchQuery(
      text = readString(map, Seq("text", "query"), ""),
      tags = readStringList(map, Seq("tags")),
      metadata = readStringMap(map, Seq("metadata", "meta")),
      limit = readInt(map, Seq("limit"), 10)
    )

  def buildSearchDocument(map: Map[String, Any]): SearchDocument =
    SearchDocument(
      id = readString(map, Seq("id"), s"doc-${System.currentTimeMillis()}"),
      title = readString(map, Seq("title"), ""),
      body = readString(map, Seq("body", "content"), ""),
      tags = readStringList(map, Seq("tags")),
      metadata = readStringMap(map, Seq("metadata", "meta"))
    )

  def buildStreamEvent(map: Map[String, Any]): StreamEvent =
    StreamEvent(
      id = readString(map, Seq("id"), s"evt-${System.currentTimeMillis()}"),
      profileId = readString(map, Seq("profile_id", "profileId", "user_id", "userId"), "system"),
      module = readString(map, Seq("module"), "system"),
      eventType = readString(map, Seq("event_type", "eventType"), "event"),
      timestampMs = readLong(map, Seq("timestamp_ms", "timestampMs"), System.currentTimeMillis()),
      productivityDelta = readDouble(map, Seq("productivity_delta"), 0.0),
      cashFlowDelta = readDouble(map, Seq("cashflow_delta", "cash_flow_delta"), 0.0),
      collaborationDelta = readDouble(map, Seq("collaboration_delta"), 0.0),
      riskDelta = readDouble(map, Seq("risk_delta"), 0.0),
      metadata = readStringMap(map, Seq("metadata", "meta"))
    )

  def buildModuleMetricSample(map: Map[String, Any]): ModuleMetricSample =
    ModuleMetricSample(
      id = readString(map, Seq("id"), s"mm-${System.currentTimeMillis()}"),
      module = readString(map, Seq("module"), "system"),
      timestampMs = readLong(map, Seq("timestamp_ms", "timestampMs"), System.currentTimeMillis()),
      latencyMs = readDouble(map, Seq("latency_ms", "latency"), 0.0),
      queueDepth = readInt(map, Seq("queue_depth"), 0),
      errorCount = readInt(map, Seq("error_count"), 0),
      successCount = readInt(map, Seq("success_count"), 1),
      throughputUnits = readDouble(map, Seq("throughput_units"), 1.0),
      cpuPct = readDouble(map, Seq("cpu_pct"), 0.0),
      memoryMb = readDouble(map, Seq("memory_mb"), 0.0),
      metadata = readStringMap(map, Seq("metadata", "meta"))
    )

  def buildHostMetricSample(map: Map[String, Any]): HostMetricSample =
    HostMetricSample(
      id = readString(map, Seq("id"), s"hm-${System.currentTimeMillis()}"),
      hostId = readString(map, Seq("host_id", "hostId"), "kogi-host-001"),
      timestampMs = readLong(map, Seq("timestamp_ms", "timestampMs"), System.currentTimeMillis()),
      cpuPct = readDouble(map, Seq("cpu_pct"), 0.0),
      memoryPct = readDouble(map, Seq("memory_pct"), 0.0),
      diskPct = readDouble(map, Seq("disk_pct"), 0.0),
      processCount = readInt(map, Seq("process_count"), 0),
      schedulerLoad = readDouble(map, Seq("scheduler_load"), 0.0),
      networkInKbps = readDouble(map, Seq("network_in_kbps"), 0.0),
      networkOutKbps = readDouble(map, Seq("network_out_kbps"), 0.0),
      metadata = readStringMap(map, Seq("metadata", "meta"))
    )

  def buildOptimizationRequest(map: Map[String, Any]): OptimizationRequest =
    OptimizationRequest(
      workloadId = readString(map, Seq("workload_id", "workloadId", "id"), "workload"),
      cpuPct = readDouble(map, Seq("cpu_pct"), 0.0),
      memoryPct = readDouble(map, Seq("memory_pct"), 0.0),
      queueDepth = readDouble(map, Seq("queue_depth"), 0.0),
      latencyMs = readDouble(map, Seq("latency_ms"), 0.0),
      objectives = readStringList(map, Seq("objectives")),
      persona = readOptString(map, Seq("persona")).map(personaFrom)
    )

  def buildRiskOptimizationRequest(map: Map[String, Any]): RiskOptimizationRequest = {
    val signal =
      if (map.contains("signal")) signalFrom(readMap(map, Seq("signal")))
      else signalFrom(map)
    RiskOptimizationRequest(
      profileId = readString(map, Seq("profile_id", "profileId", "user_id", "userId"), "system"),
      signal = signal,
      context = readStringMap(map, Seq("context"))
    )
  }

  def buildQueryProfile(map: Map[String, Any]): QueryProfile =
    QueryProfile(
      userId = readString(map, Seq("user_id", "userId", "id"), "system"),
      persona = personaFrom(readString(map, Seq("persona"), "newcomer")),
      preferredLimit = readInt(map, Seq("preferred_limit", "limit"), 1000),
      allowAnalyticHints = readBoolean(map, Seq("allow_analytic_hints", "analytic_hints"), false)
    )

  def buildPersonalizationRequest(map: Map[String, Any]): PersonalizationRequest =
    PersonalizationRequest(
      profileId = readString(map, Seq("profile_id", "profileId", "user_id", "userId"), ""),
      context = readStringMap(map, Seq("context")),
      preferences = readStringMap(map, Seq("preferences", "prefs")),
      persona = readOptString(map, Seq("persona")).map(personaFrom)
    )

  def buildSegment(map: Map[String, Any]): Segment =
    Segment(
      id = readString(map, Seq("id"), ""),
      name = readString(map, Seq("name"), ""),
      rules = seqMap(readMap(map, Seq("rules"))),
      description = readString(map, Seq("description"), "")
    )

  def buildExperimentVariant(map: Map[String, Any]): ExperimentVariant =
    ExperimentVariant(
      id = readString(map, Seq("id"), ""),
      name = readString(map, Seq("name"), ""),
      weight = readDouble(map, Seq("weight"), 1.0)
    )

  def buildExperiment(map: Map[String, Any]): Experiment = {
    val variants = readMapList(map, Seq("variants")).map(buildExperimentVariant)
    Experiment(
      id = readString(map, Seq("id"), ""),
      name = readString(map, Seq("name"), ""),
      variants = variants,
      targetSegments = readStringList(map, Seq("target_segments", "segments")),
      active = readBoolean(map, Seq("active"), true)
    )
  }

  def buildUserSubject(map: Map[String, Any]): UserSubject =
    UserSubject(
      id = readString(map, Seq("id", "user_id", "userId"), ""),
      role = userRoleFrom(readString(map, Seq("role"), "any")),
      persona = personaFrom(readString(map, Seq("persona"), "newcomer")),
      skills = readStringList(map, Seq("skills")),
      interests = readStringList(map, Seq("interests")),
      budget = readOptString(map, Seq("budget")).flatMap(asDoubleOpt),
      location = readString(map, Seq("location"), "unknown"),
      tags = readStringList(map, Seq("tags")),
      attributes = readStringMap(map, Seq("attributes", "attrs"))
    )

  def buildComponentSubject(map: Map[String, Any]): ComponentSubject =
    ComponentSubject(
      id = readString(map, Seq("id", "component_id", "componentId"), ""),
      kind = componentKindFrom(readString(map, Seq("kind"), "any")),
      category = readString(map, Seq("category"), "general"),
      status = readString(map, Seq("status"), "active"),
      ownerIds = readStringList(map, Seq("owner_ids", "owners")),
      valueScore = readDouble(map, Seq("value_score"), 50.0),
      riskScore = readDouble(map, Seq("risk_score"), 50.0),
      tags = readStringList(map, Seq("tags")),
      attributes = readStringMap(map, Seq("attributes", "attrs"))
    )

  def buildResourceSubject(map: Map[String, Any]): ResourceSubject =
    ResourceSubject(
      id = readString(map, Seq("id", "resource_id", "resourceId"), ""),
      resourceType = readString(map, Seq("resource_type", "type"), "general"),
      capacity = readDouble(map, Seq("capacity"), 0.0),
      unit = readString(map, Seq("unit"), "units"),
      category = readString(map, Seq("category"), "general"),
      tags = readStringList(map, Seq("tags")),
      attributes = readStringMap(map, Seq("attributes", "attrs"))
    )

  def buildAssetSubject(map: Map[String, Any]): AssetSubject =
    AssetSubject(
      id = readString(map, Seq("id", "asset_id", "assetId"), ""),
      assetClass = readString(map, Seq("asset_class", "class"), "general"),
      marketValue = readDouble(map, Seq("market_value", "value"), 0.0),
      liquidity = readDouble(map, Seq("liquidity"), 50.0),
      category = readString(map, Seq("category"), "general"),
      tags = readStringList(map, Seq("tags")),
      attributes = readStringMap(map, Seq("attributes", "attrs"))
    )

  def buildProfile(map: Map[String, Any]): Profile =
    Profile(
      id = readString(map, Seq("id"), ""),
      tags = readStringList(map, Seq("tags")),
      attributes = readStringMap(map, Seq("attributes", "attrs"))
    )

  def buildResource(map: Map[String, Any]): Resource =
    Resource(
      id = readString(map, Seq("id"), ""),
      tags = readStringList(map, Seq("tags")),
      attributes = readStringMap(map, Seq("attributes", "attrs"))
    )

  def buildAnalyticsArtifact(map: Map[String, Any]): AnalyticsArtifact =
    AnalyticsArtifact(
      id = readString(map, Seq("id"), ""),
      artifactType = readString(map, Seq("artifact_type", "type"), "artifact"),
      score = readDouble(map, Seq("score"), 0.0),
      sourceModule = readString(map, Seq("source_module", "source"), "system"),
      tags = readStringList(map, Seq("tags")),
      attributes = readStringMap(map, Seq("attributes", "attrs"))
    )

  def buildMatchWeights(map: Map[String, Any]): MatchWeights =
    MatchWeights(
      tagOverlap = readDouble(map, Seq("tag_overlap"), MatchWeights.default.tagOverlap),
      attributeOverlap = readDouble(map, Seq("attribute_overlap"), MatchWeights.default.attributeOverlap),
      categoryMatch = readDouble(map, Seq("category_match"), MatchWeights.default.categoryMatch),
      personaAlignment = readDouble(map, Seq("persona_alignment"), MatchWeights.default.personaAlignment),
      signalStrength = readDouble(map, Seq("signal_strength"), MatchWeights.default.signalStrength),
      contextBoost = readDouble(map, Seq("context_boost"), MatchWeights.default.contextBoost)
    )

  def buildGameScoreWeights(map: Map[String, Any]): GameScoreWeights =
    GameScoreWeights(
      matchWeight = readDouble(map, Seq("match_weight"), 0.55),
      priceWeight = readDouble(map, Seq("price_weight"), 0.25),
      qualityWeight = readDouble(map, Seq("quality_weight"), 0.15),
      reputationWeight = readDouble(map, Seq("reputation_weight"), 0.05)
    )

  def buildGameListing(map: Map[String, Any]): GameListing = {
    val componentMap = readMap(map, Seq("component"))
    val component =
      if (componentMap.nonEmpty) buildComponentSubject(componentMap)
      else buildComponentSubject(map.view.filterKeys(_.startsWith("component_")).map { case (k, v) => k.stripPrefix("component_") -> v }.toMap)

    val reserveOpt = readOptString(map, Seq("reserve_price", "reserve")).flatMap(asDoubleOpt)

    GameListing(
      id = readString(map, Seq("id", "listing_id"), ""),
      ownerId = readString(map, Seq("owner_id", "ownerId"), ""),
      component = component,
      listingType = readString(map, Seq("listing_type", "type"), "listing"),
      mechanism = GameMechanism.fromString(readString(map, Seq("mechanism"), "assignment")),
      askingPrice = readDouble(map, Seq("asking_price", "price"), 0.0),
      reservePrice = reserveOpt,
      quantity = readInt(map, Seq("quantity"), 1),
      minQualityScore = readDouble(map, Seq("min_quality_score"), 0.0),
      status = listingStatusFrom(readString(map, Seq("status"), "open")),
      createdAtMs = readLong(map, Seq("created_at_ms", "createdAtMs"), System.currentTimeMillis())
    )
  }

  def buildGameAllocation(map: Map[String, Any]): GameAllocation =
    GameAllocation(
      listingId = readString(map, Seq("listing_id", "listingId"), ""),
      winnerId = readString(map, Seq("winner_id", "winnerId"), ""),
      clearingPrice = readDouble(map, Seq("clearing_price"), 0.0),
      mechanism = GameMechanism.fromString(readString(map, Seq("mechanism"), "assignment")),
      compositeScore = readDouble(map, Seq("composite_score"), 0.0),
      matchScore = readDouble(map, Seq("match_score"), 0.0),
      priceScore = readDouble(map, Seq("price_score"), 0.0),
      qualityScore = readDouble(map, Seq("quality_score"), 0.0),
      reputationScore = readDouble(map, Seq("reputation_score"), 0.0),
      reasons = readStringList(map, Seq("reasons")),
      createdAtMs = readLong(map, Seq("created_at_ms", "createdAtMs"), System.currentTimeMillis())
    )

  def buildGameIncentivePolicy(map: Map[String, Any]): GameIncentivePolicy =
    GameIncentivePolicy(
      winnerBaseKp = readDouble(map, Seq("winner_base_kp"), 40.0),
      ownerBaseKp = readDouble(map, Seq("owner_base_kp"), 10.0),
      winnerReputationDelta = readDouble(map, Seq("winner_reputation_delta"), 1.5),
      ownerReputationDelta = readDouble(map, Seq("owner_reputation_delta"), 0.5),
      qualityKpScale = readDouble(map, Seq("quality_kp_scale"), 35.0),
      matchKpScale = readDouble(map, Seq("match_kp_scale"), 45.0)
    )

  def buildGameBid(map: Map[String, Any]): GameBid =
    GameBid(
      id = readString(map, Seq("id", "bid_id"), ""),
      listingId = readString(map, Seq("listing_id", "listingId"), ""),
      bidderId = readString(map, Seq("bidder_id", "bidderId"), ""),
      amount = readDouble(map, Seq("amount"), 0.0),
      qualityScore = readDouble(map, Seq("quality_score"), 50.0),
      bidType = readString(map, Seq("bid_type", "type"), "standard"),
      submittedAtMs = readLong(map, Seq("submitted_at_ms", "submittedAtMs"), System.currentTimeMillis()),
      status = bidStatusFrom(readString(map, Seq("status"), "active")),
      metadata = readStringMap(map, Seq("metadata", "meta"))
    )

  def buildAllocationParticipant(map: Map[String, Any]): AllocationParticipant = {
    val subjectMap = readMap(map, Seq("subject", "user"))
    val subject = if (subjectMap.nonEmpty) buildUserSubject(subjectMap) else buildUserSubject(map)
    AllocationParticipant(
      subject = subject,
      reputation = readDouble(map, Seq("reputation"), 50.0)
    )
  }

  def buildAllocationRequest(map: Map[String, Any]): AllocationRequest = {
    val listing = buildGameListing(readMap(map, Seq("listing")) ++ map.filterKeys(_.startsWith("listing_")).map { case (k, v) => k.stripPrefix("listing_") -> v })
    val bids = readMapList(map, Seq("bids")).map(buildGameBid)
    val participants = readMapList(map, Seq("participants", "participant"))
      .map(buildAllocationParticipant)
    val scoreWeights = buildGameScoreWeights(readMap(map, Seq("score_weights", "weights")))
    val matchWeights = buildMatchWeights(readMap(map, Seq("match_weights")))
    val limit = readInt(map, Seq("limit"), 1)
    AllocationRequest(
      listing = listing,
      bids = bids,
      participants = participants,
      scoreWeights = scoreWeights,
      matchWeights = matchWeights,
      limit = limit
    )
  }

  def buildIncentiveEvent(map: Map[String, Any]): IncentiveEvent =
    IncentiveEvent(
      participantId = readString(map, Seq("participant_id", "participantId"), ""),
      eventType = readString(map, Seq("event_type", "eventType"), "event"),
      kpDelta = readDouble(map, Seq("kp_delta"), 0.0),
      reputationDelta = readDouble(map, Seq("reputation_delta"), 0.0),
      referenceId = readString(map, Seq("reference_id"), ""),
      referenceType = readString(map, Seq("reference_type"), ""),
      timestampMs = readLong(map, Seq("timestamp_ms"), System.currentTimeMillis())
    )

  def buildGameIncentive(map: Map[String, Any]): GameIncentive =
    GameIncentive(
      participantId = readString(map, Seq("participant_id", "participantId"), ""),
      kpDelta = readDouble(map, Seq("kp_delta"), 0.0),
      reputationDelta = readDouble(map, Seq("reputation_delta"), 0.0),
      reason = readString(map, Seq("reason"), ""),
      timestampMs = readLong(map, Seq("timestamp_ms"), System.currentTimeMillis())
    )

  def buildPolicyContext(map: Map[String, Any]): PolicyContext =
    PolicyContext(
      componentId = readString(map, Seq("component_id", "componentId"), ""),
      eventKind = readString(map, Seq("event_kind", "eventKind"), ""),
      actorId = readString(map, Seq("actor_id", "actorId"), ""),
      componentType = readString(map, Seq("component_type"), "component"),
      properties = readStringMap(map, Seq("properties", "props")),
      tags = readStringSet(map, Seq("tags")),
      timestampMs = readLong(map, Seq("timestamp_ms"), System.currentTimeMillis())
    )

  def buildGraphEdge(map: Map[String, Any]): GraphEdge =
    GraphEdge(
      from = readString(map, Seq("from"), ""),
      to = readString(map, Seq("to"), ""),
      edgeType = edgeTypeFrom(readString(map, Seq("edge_type", "edgeType"), "dependency")),
      weight = readDouble(map, Seq("weight"), 1.0)
    )

  def buildGraphNode(map: Map[String, Any]): GraphNode =
    GraphNode(
      id = readString(map, Seq("id"), ""),
      duration = readDouble(map, Seq("duration"), 0.0)
    )
}
