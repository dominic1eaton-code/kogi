package kogi.engine

object EngineActionDispatcher {
  import EngineRequestReader._
  import EngineModelBuilders._

  def handle(engine: KogiEngine, action: String, request: Map[String, Any]): Map[String, Any] = {
    val act = action.toLowerCase
    act match {
      // ----------------------------------------------------------------
      // Core engine control
      // ----------------------------------------------------------------
      case "control" =>
        val mode = readString(request, Seq("mode", "value", "action"), "start")
        controlResponse(engine.control(mode))

      case "status" =>
        controlResponse(engine.status)

      // ----------------------------------------------------------------
      // Telemetry + flow ingest
      // ----------------------------------------------------------------
      case "ingest" =>
        val topic = readString(request, Seq("topic"), "engine.ingest")
        val source = readString(request, Seq("source"), "kogi.network.engine")
        val target = readString(request, Seq("target"), "kogi.engine")
        val flowId = readString(request, Seq("flow_id", "flow-id", "flowId"), s"flow-${safeId(topic)}")
        val timestampMs = readLong(request, Seq("timestamp_ms", "timestamp-ms", "timestampMs"), System.currentTimeMillis())
        val payload = readStringMap(request, Seq("payload"))
        snapshotResponse(engine.ingestGatewayMessage(topic, payload, source, target, flowId, timestampMs))

      case "ingest-envelope" =>
        val topic = readString(request, Seq("topic"), "engine.ingest")
        val source = readString(request, Seq("source"), "kogi.network.engine")
        val target = readString(request, Seq("target"), "kogi.engine")
        val flowId = readString(request, Seq("flow_id", "flow-id", "flowId"), s"flow-${safeId(topic)}")
        val timestampMs = readLong(request, Seq("timestamp_ms", "timestamp-ms", "timestampMs"), System.currentTimeMillis())
        val id = readString(request, Seq("id"), s"env-${safeId(flowId)}-$timestampMs")
        val payload = readStringMap(request, Seq("payload"))
        val envelope = PlatformDataEnvelope(
          id = id,
          flowId = flowId,
          source = source,
          target = target,
          topic = topic,
          payload = payload,
          timestampMs = timestampMs
        )
        snapshotResponse(engine.ingestEnvelope(envelope))

      case "snapshot" =>
        val hostId = readString(request, Seq("host_id", "host-id", "hostId"), "kogi-host-001")
        val windowMs = readLong(request, Seq("window_ms", "window-ms", "windowMs"), 5L * 60L * 1000L)
        snapshotResponse(engine.snapshot(hostId, windowMs))

      case "flow-ledger" =>
        val limit = readInt(request, Seq("limit"), 100)
        ok(act, "limit" -> limit, "entries" -> engine.flowLedger(limit))

      case "flow-envelopes" =>
        val limit = readInt(request, Seq("limit"), 100)
        ok(act, "limit" -> limit, "envelopes" -> engine.flowEnvelopes(limit))

      // ----------------------------------------------------------------
      // Analytics engine
      // ----------------------------------------------------------------
      case "analytics-module-catalog" =>
        ok(act, "modules" -> engine.analyticsEngine.moduleCatalog())

      case "analytics-ingest-event" =>
        val event = buildStreamEvent(request)
        ok(act, "snapshot" -> engine.analyticsEngine.ingest(event))

      case "analytics-ingest-batch" =>
        val events = readMapList(request, Seq("events", "event")).map(buildStreamEvent)
        if (events.isEmpty) err(act, "missing events")
        else ok(act, "count" -> events.size, "snapshots" -> engine.analyticsEngine.ingestBatch(events))

      case "analytics-ingest-module-metric" =>
        val sample = buildModuleMetricSample(request)
        ok(act, "snapshot" -> engine.analyticsEngine.ingestModuleMetric(sample))

      case "analytics-ingest-host-metric" =>
        val sample = buildHostMetricSample(request)
        ok(act, "snapshot" -> engine.analyticsEngine.ingestHostMetric(sample))

      case "analytics-ingest-realtime" =>
        val eventOpt = readMapList(request, Seq("event")).headOption.map(buildStreamEvent)
        val moduleOpt = readMapList(request, Seq("module_metric", "module-metric")).headOption.map(buildModuleMetricSample)
        val hostOpt = readMapList(request, Seq("host_metric", "host-metric")).headOption.map(buildHostMetricSample)
        val hostId = readString(request, Seq("host_id", "hostId"), "kogi-host-001")
        val windowMs = readLong(request, Seq("window_ms", "windowMs"), 5L * 60L * 1000L)
        ok(act, "snapshot" -> engine.analyticsEngine.ingestRealtime(eventOpt, moduleOpt, hostOpt, hostId, windowMs))

      case "analytics-ingest-bus" =>
        val topic = readString(request, Seq("topic"), "analytics.bus")
        val profileId = readString(request, Seq("profile_id", "profileId"), "system")
        val hostId = readString(request, Seq("host_id", "hostId"), "kogi-host-001")
        val timestampMs = readLong(request, Seq("timestamp_ms", "timestampMs"), System.currentTimeMillis())
        val windowMs = readLong(request, Seq("window_ms", "windowMs"), 5L * 60L * 1000L)
        val payload = readStringMap(request, Seq("payload"))
        ok(act, "snapshot" -> engine.ingestBusEvent(topic, payload, profileId, hostId, timestampMs, windowMs))

      case "analytics-snapshot-profile" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "system")
        val window = readInt(request, Seq("window"), 250)
        ok(act, "snapshot" -> engine.analyticsEngine.snapshotForProfile(profileId, window))

      case "analytics-module-activity" =>
        val window = readInt(request, Seq("window"), 250)
        ok(act, "modules" -> engine.analyticsEngine.globalModuleActivity(window), "window" -> window)

      case "analytics-module-snapshot" =>
        val module = readString(request, Seq("module"), "")
        if (module.isEmpty) err(act, "missing module")
        else {
          val windowMs = readLong(request, Seq("window_ms", "windowMs"), 5L * 60L * 1000L)
          ok(act, "snapshot" -> engine.analyticsEngine.moduleSnapshot(module, windowMs))
        }

      case "analytics-host-snapshot" =>
        val hostId = readString(request, Seq("host_id", "hostId"), "kogi-host-001")
        val windowMs = readLong(request, Seq("window_ms", "windowMs"), 5L * 60L * 1000L)
        ok(act, "snapshot" -> engine.analyticsEngine.hostSnapshot(hostId, windowMs))

      case "analytics-system-snapshot" =>
        val hostId = readString(request, Seq("host_id", "hostId"), "kogi-host-001")
        val windowMs = readLong(request, Seq("window_ms", "windowMs"), 5L * 60L * 1000L)
        val modules = readStringList(request, Seq("modules"))
        ok(act, "snapshot" -> engine.analyticsEngine.systemSnapshot(hostId, modules, windowMs))

      // ----------------------------------------------------------------
      // Stream engine
      // ----------------------------------------------------------------
      case "stream-ingest" =>
        val event = buildStreamEvent(request)
        engine.dataStreamingEngine.ingest(event)
        ok(act, "size" -> engine.dataStreamingEngine.size)

      case "stream-ingest-batch" =>
        val events = readMapList(request, Seq("events", "event")).map(buildStreamEvent)
        engine.dataStreamingEngine.ingestBatch(events)
        ok(act, "count" -> events.size, "size" -> engine.dataStreamingEngine.size)

      case "stream-recent" =>
        val limit = readInt(request, Seq("limit"), 100)
        ok(act, "events" -> engine.dataStreamingEngine.recent(limit))

      case "stream-all" =>
        ok(act, "events" -> engine.dataStreamingEngine.all)

      case "stream-by-profile" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 100)
        ok(act, "events" -> engine.dataStreamingEngine.eventsByProfile(profileId, limit))

      case "stream-by-module" =>
        val module = readString(request, Seq("module"), "")
        val limit = readInt(request, Seq("limit"), 100)
        ok(act, "events" -> engine.dataStreamingEngine.eventsByModule(module, limit))

      case "stream-since" =>
        val sinceMs = readLong(request, Seq("since_ms", "sinceMs"), System.currentTimeMillis())
        ok(act, "events" -> engine.dataStreamingEngine.eventsSince(sinceMs))

      case "stream-by-module-since" =>
        val module = readString(request, Seq("module"), "")
        val sinceMs = readLong(request, Seq("since_ms", "sinceMs"), System.currentTimeMillis())
        ok(act, "events" -> engine.dataStreamingEngine.eventsByModuleSince(module, sinceMs))

      case "stream-size" =>
        ok(act, "size" -> engine.dataStreamingEngine.size)

      // ----------------------------------------------------------------
      // Recommendation engine
      // ----------------------------------------------------------------
      case "rec-upsert-item" =>
        val item = buildItemProfile(request)
        engine.recUpsertItem(item)
        ok(act, "item" -> item)

      case "rec-upsert-items" =>
        val items = readMapList(request, Seq("items", "item")).map(buildItemProfile)
        engine.recUpsertItems(items)
        ok(act, "count" -> items.size)

      case "rec-upsert-profile" =>
        val profile = buildUserProfile(request)
        engine.recUpsertProfile(profile)
        ok(act, "profile" -> profile)

      case "rec-record-interaction" =>
        val interaction = buildUserInteraction(request)
        ok(act, "profile" -> engine.recRecordInteraction(interaction))

      case "rec-record-interactions" =>
        val interactions = readMapList(request, Seq("interactions", "interaction")).map(buildUserInteraction)
        engine.recommendationEngine.recordInteractions(interactions)
        ok(act, "count" -> interactions.size)

      case "rec-record-rating" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val itemId = readString(request, Seq("item_id", "itemId"), "")
        val rating = readDouble(request, Seq("rating"), 0.0)
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "profile" -> engine.recRecordRating(userId, itemId, rating, ctx))

      case "rec-record-review" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val itemId = readString(request, Seq("item_id", "itemId"), "")
        val sentiment = readDouble(request, Seq("sentiment"), 0.0)
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "profile" -> engine.recRecordReview(userId, itemId, sentiment, ctx))

      case "rec-record-dwell" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val itemId = readString(request, Seq("item_id", "itemId"), "")
        val dwell = readDouble(request, Seq("dwell_seconds", "dwell"), 0.0)
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "profile" -> engine.recRecordDwell(userId, itemId, dwell, ctx))

      case "rec-record-search" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val queryText = readString(request, Seq("query"), "")
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "profile" -> engine.recRecordSearch(userId, queryText, ctx))

      case "rec-build-persona" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        ok(act, "persona" -> engine.recBuildPersona(userId))

      case "rec-profile" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        ok(act, "profile" -> engine.recGetProfile(userId))

      case "rec-personas" =>
        ok(act, "personas" -> engine.recAllPersonas())

      case "rec-hybrid" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 8)
        val exclude = readStringSet(request, Seq("exclude"))
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "result" -> engine.recHybrid(userId, ctx, exclude, limit))

      case "rec-collab" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 8)
        val exclude = readStringSet(request, Seq("exclude"))
        ok(act, "recommendations" -> engine.recCollaborative(userId, exclude, limit))

      case "rec-content" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 8)
        val exclude = readStringSet(request, Seq("exclude"))
        ok(act, "recommendations" -> engine.recContentBased(userId, exclude, limit))

      case "rec-contextual" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 8)
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "result" -> engine.recContextual(userId, ctx, limit))

      case "rec-cold-start" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 8)
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "recommendations" -> engine.recColdStart(userId, ctx, limit))

      case "rec-feedback" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 50)
        ok(act, "history" -> engine.recFeedbackHistory(userId, limit))

      case "rec-profile-summary" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        ok(act, "summary" -> engine.recProfileSummary(userId))

      case "rec-personalized-search" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val query = buildSearchQuery(request)
        ok(act, "result" -> engine.recPersonalizedSearch(userId, query))

      case "rec-stats" =>
        ok(act,
          "item_count" -> engine.recommendationEngine.itemCatalogSize,
          "profile_count" -> engine.recommendationEngine.profileCount,
          "interaction_count" -> engine.recommendationEngine.interactionCount
        )

      // ----------------------------------------------------------------
      // Personalization engine
      // ----------------------------------------------------------------
      case "personalize" | "personalization" =>
        val req = buildPersonalizationRequest(request)
        ok(act, "result" -> engine.personalizationEngine.personalize(req))

      case "personalization-set-preference" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val key = readString(request, Seq("key"), "")
        val value = readString(request, Seq("value"), "")
        engine.personalizationEngine.setPreference(profileId, key, value)
        ok(act, "profile_id" -> profileId, "key" -> key, "value" -> value)

      case "personalization-set-preferences" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val prefs = readStringMap(request, Seq("preferences", "prefs"))
        engine.personalizationEngine.setPreferences(profileId, prefs)
        ok(act, "profile_id" -> profileId, "preferences" -> prefs)

      case "personalization-clear-preference" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val key = readString(request, Seq("key"), "")
        engine.personalizationEngine.clearPreference(profileId, key)
        ok(act, "profile_id" -> profileId, "key" -> key)

      case "personalization-preferences" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        ok(act, "preferences" -> engine.personalizationEngine.getExplicitPreferences(profileId))

      case "personalization-resolve-preferences" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val ctx = readStringMap(request, Seq("context"))
        val persona = personaFor(engine, profileId, request)
        ok(act, "resolved" -> engine.personalizationEngine.resolvePreferences(profileId, persona, ctx))

      case "personalization-delivery-config" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val ctx = readStringMap(request, Seq("context"))
        val persona = personaFor(engine, profileId, request)
        val resolved = engine.personalizationEngine.resolvePreferences(profileId, persona, ctx)
        ok(act, "config" -> engine.personalizationEngine.buildDeliveryConfig(profileId, persona, resolved))

      case "personalization-rank-content" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val items = readStringList(request, Seq("items", "item_ids"))
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "ranked" -> engine.personalizationEngine.rankContent(profileId, items, ctx))

      case "personalization-register-segment" =>
        val segment = buildSegment(request)
        engine.personalizationEngine.registerSegment(segment)
        ok(act, "segment" -> segment)

      case "personalization-register-segments" =>
        val segments = readMapList(request, Seq("segments", "segment")).map(buildSegment)
        engine.personalizationEngine.registerSegments(segments)
        ok(act, "count" -> segments.size)

      case "personalization-assign-segments" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val persona = personaFor(engine, profileId, request)
        val attrs = readStringMap(request, Seq("attributes", "attrs"))
        ok(act, "assignment" -> engine.personalizationEngine.assignSegments(profileId, persona, attrs))

      case "personalization-segment-assignment" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        ok(act, "assignment" -> engine.personalizationEngine.getSegmentAssignment(profileId))

      case "personalization-profiles-in-segment" =>
        val segmentId = readString(request, Seq("segment_id", "segmentId"), "")
        ok(act, "profiles" -> engine.personalizationEngine.profilesInSegment(segmentId))

      case "personalization-build-persona" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        ok(act, "persona" -> engine.personalizationEngine.buildPersona(profileId))

      case "personalization-detect-drift" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        ok(act, "drift" -> engine.personalizationEngine.detectPersonaDrift(profileId))

      case "personalization-apply-drift" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        ok(act, "persona" -> engine.personalizationEngine.applyPersonaDriftIfNeeded(profileId))

      case "personalization-register-experiment" =>
        val exp = buildExperiment(request)
        engine.personalizationEngine.registerExperiment(exp)
        ok(act, "experiment" -> exp)

      case "personalization-assign-experiment" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val exp = buildExperiment(request)
        ok(act, "assignment" -> engine.personalizationEngine.assignExperiment(profileId, exp))

      case "personalization-record-exposure" =>
        val experimentId = readString(request, Seq("experiment_id", "experimentId"), "")
        val variantId = readString(request, Seq("variant_id", "variantId"), "")
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val converted = readBoolean(request, Seq("converted"), false)
        engine.personalizationEngine.recordExposure(experimentId, variantId, profileId, converted)
        ok(act, "experiment_id" -> experimentId, "variant_id" -> variantId, "profile_id" -> profileId, "converted" -> converted)

      case "personalization-experiment-stats" =>
        val experimentId = readString(request, Seq("experiment_id", "experimentId"), "")
        ok(act, "stats" -> engine.personalizationEngine.experimentStats(experimentId))

      case "personalization-experiment-assignments" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        ok(act, "assignments" -> engine.personalizationEngine.experimentAssignmentsFor(profileId))

      case "personalization-register-bandit" =>
        val slotId = readString(request, Seq("slot_id", "slotId"), "")
        val arms = readStringList(request, Seq("arms", "arm_ids"))
        engine.personalizationEngine.registerBanditSlot(slotId, arms)
        ok(act, "slot_id" -> slotId, "arms" -> arms)

      case "personalization-select-bandit" =>
        val slotId = readString(request, Seq("slot_id", "slotId"), "")
        ok(act, "selection" -> engine.personalizationEngine.selectBanditArm(slotId))

      case "personalization-record-bandit" =>
        val slotId = readString(request, Seq("slot_id", "slotId"), "")
        val armId = readString(request, Seq("arm_id", "armId"), "")
        val reward = readDouble(request, Seq("reward"), 0.0)
        engine.personalizationEngine.recordBanditReward(slotId, armId, reward)
        ok(act, "slot_id" -> slotId, "arm_id" -> armId, "reward" -> reward)

      case "personalization-bandit-stats" =>
        val slotId = readString(request, Seq("slot_id", "slotId"), "")
        val total = readInt(request, Seq("total_pulls"), -1)
        ok(act, "stats" -> engine.personalizationEngine.banditStats(slotId, total))

      case "personalization-adapt-content" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val items = readStringList(request, Seq("items", "item_ids"))
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        val itemMetaRaw = readString(request, Seq("item_meta", "item-meta"), "")
        val itemMeta = parseItemMeta(itemMetaRaw)
        ok(act, "items" -> engine.personalizationEngine.adaptContent(profileId, items, ctx, itemMeta))

      case "personalization-record-interaction" =>
        val interaction = buildUserInteraction(request)
        ok(act, "profile" -> engine.personalizationEngine.recordInteraction(interaction))

      case "personalization-record-interactions" =>
        val interactions = readMapList(request, Seq("interactions", "interaction")).map(buildUserInteraction)
        engine.personalizationEngine.recordInteractions(interactions)
        ok(act, "count" -> interactions.size)

      case "personalization-register-item" =>
        val item = buildItemProfile(request)
        engine.personalizationEngine.registerItem(item)
        ok(act, "item" -> item)

      case "personalization-recommendations" =>
        val profileId = readString(request, Seq("profile_id", "profileId", "user_id", "userId"), "")
        val limit = readInt(request, Seq("limit"), 8)
        val ctx = buildInteractionContext(readMap(request, Seq("context")) ++ request)
        ok(act, "result" -> engine.personalizationEngine.getRecommendations(profileId, ctx, limit))

      // ----------------------------------------------------------------
      // Search + query
      // ----------------------------------------------------------------
      case "search" =>
        searchResponse(engine.search(buildSearchQuery(request)))

      case "index" =>
        val doc = buildSearchDocument(request)
        engine.index(doc)
        ok(act, "id" -> doc.id)

      case "search-index-batch" =>
        val docs = readMapList(request, Seq("documents", "docs", "items")).map(buildSearchDocument)
        engine.searchEngine.indexBatch(docs)
        ok(act, "count" -> docs.size)

      case "search-remove" =>
        val id = readString(request, Seq("id"), "")
        engine.searchEngine.remove(id)
        ok(act, "id" -> id)

      case "search-filter" =>
        ok(act, "documents" -> engine.searchEngine.filter(buildSearchQuery(request)))

      case "search-personalized" =>
        val userId = readString(request, Seq("user_id", "userId"), "")
        val query = buildSearchQuery(request)
        val prefMap = readStringMap(request, Seq("preferences", "preference_vector"))
        val dominant = readStringList(request, Seq("dominant_categories", "dominant"))
        if (prefMap.nonEmpty || dominant.nonEmpty) {
          val prefVec = prefMap.map { case (k, v) => k -> v.toDoubleOption.getOrElse(0.0) }
          val pq = PersonalizedSearchQuery(query, userId, prefVec, dominant)
          ok(act, "result" -> engine.searchEngine.personalizedSearch(pq))
        } else {
          ok(act, "result" -> engine.recPersonalizedSearch(userId, query))
        }

      case "query" =>
        val sql = readString(request, Seq("sql"), "")
        planResponse(engine.query(sql))

      case "query-analyze" =>
        val sql = readString(request, Seq("sql"), "")
        ok(act, "analysis" -> engine.queryEngine.analyze(sql))

      case "query-optimize" =>
        val sql = readString(request, Seq("sql"), "")
        ok(act, "plan" -> engine.queryEngine.optimize(sql))

      case "query-optimize-profile" =>
        val sql = readString(request, Seq("sql"), "")
        val profile = buildQueryProfile(readMap(request, Seq("profile")) ++ request)
        ok(act, "plan" -> engine.queryEngine.optimizeWithProfile(sql, profile))

      // ----------------------------------------------------------------
      // Risk + optimization
      // ----------------------------------------------------------------
      case "risk-score" =>
        val signal = buildPortfolioSignal(request)
        ok(act, "health" -> engine.riskEngine.score(signal))

      case "risk-score-persona" =>
        val signal = buildPortfolioSignal(request)
        val persona = personaFor(engine, readString(request, Seq("profile_id", "profileId"), ""), request)
        ok(act, "health" -> engine.riskEngine.scoreWithPersona(signal, persona))

      case "risk-optimize" =>
        val req = buildRiskOptimizationRequest(request)
        ok(act, "plan" -> engine.riskEngine.optimize(req))

      case "risk-manage" =>
        val req = buildRiskOptimizationRequest(request)
        ok(act, "plan" -> engine.riskEngine.manage(req.profileId, req.signal))

      case "optimize" =>
        val req = buildOptimizationRequest(request)
        ok(act, "plan" -> engine.optimize(req))

      // ----------------------------------------------------------------
      // Policy engine
      // ----------------------------------------------------------------
      case "policy-register-simple" =>
        val id = readString(request, Seq("id"), "")
        val name = readString(request, Seq("name"), "")
        val decisionKind = readString(request, Seq("decision"), "allow")
        val reason = readString(request, Seq("reason"), "")
        val componentId = readOptString(request, Seq("component_id", "componentId"))
        val eventKind = readOptString(request, Seq("event_kind", "eventKind"))
        val actorId = readOptString(request, Seq("actor_id", "actorId"))
        val componentType = readOptString(request, Seq("component_type", "componentType"))
        val tags = readStringList(request, Seq("tags"))
        val props = readStringMap(request, Seq("properties", "props"))
        val criteriaParts = Seq(
          componentId.map(v => s"component_id=$v"),
          eventKind.map(v => s"event_kind=$v"),
          actorId.map(v => s"actor_id=$v"),
          componentType.map(v => s"component_type=$v"),
          Option.when(tags.nonEmpty)(s"tags=${tags.mkString(",")}"),
          Option.when(props.nonEmpty)(s"props=${props.map { case (k, v) => s\"$k=$v\" }.mkString(",")}")
        ).flatten
        val description = readString(request, Seq("description"), criteriaParts.mkString("; "))
        val decision = decisionKind.trim.toLowerCase match {
          case "deny" => PolicyDecision.Deny(if (reason.nonEmpty) reason else "denied")
          case "require_approval" | "require-approval" | "approval" =>
            PolicyDecision.RequireApproval(if (reason.nonEmpty) reason else "requires approval")
          case _ => PolicyDecision.Allow
        }
        val rule = SimplePolicyRule(
          id = if (id.nonEmpty) id else s"policy-${System.currentTimeMillis()}",
          name = if (name.nonEmpty) name else "policy-rule",
          decision = decision,
          when = ctx =>
            componentId.forall(_ == ctx.componentId) &&
              eventKind.forall(_ == ctx.eventKind) &&
              actorId.forall(_ == ctx.actorId) &&
              componentType.forall(_ == ctx.componentType) &&
              (tags.isEmpty || tags.exists(ctx.tags.contains)) &&
              props.forall { case (k, v) => ctx.properties.get(k).contains(v) },
          description = description
        )
        engine.policyEngine.register(rule)
        ok(act, "rule" -> policyRuleSummary(rule))

      case "policy-unregister" =>
        val id = readString(request, Seq("id"), "")
        ok(act, "rule" -> engine.policyEngine.unregister(id))

      case "policy-list" =>
        ok(act, "rules" -> engine.policyEngine.listRules().map(policyRuleSummary))

      case "policy-clear" =>
        engine.policyEngine.clearRules()
        ok(act)

      case "policy-evaluate" =>
        val ctx = buildPolicyContext(request)
        ok(act, "decision" -> policyDecisionMap(engine.policyEngine.evaluate(ctx)))

      case "policy-evaluate-report" =>
        val ctx = buildPolicyContext(request)
        ok(act, "report" -> engine.policyEngine.evaluateReport(ctx))

      case "policy-request-approval" =>
        val componentId = readString(request, Seq("component_id", "componentId"), "")
        val requestedBy = readString(request, Seq("requested_by", "requestedBy"), "")
        val reason = readString(request, Seq("reason"), "")
        val metadata = readStringMap(request, Seq("metadata", "meta"))
        ok(act, "request" -> engine.policyEngine.requestApproval(componentId, requestedBy, reason, metadata))

      case "policy-resolve-approval" =>
        val requestId = readString(request, Seq("request_id", "requestId"), "")
        val approved = readBoolean(request, Seq("approved"), false)
        val resolver = readString(request, Seq("resolver"), "")
        val notes = readOptString(request, Seq("notes"))
        ok(act, "request" -> engine.policyEngine.resolveApproval(requestId, approved, resolver, notes))

      case "policy-approval" =>
        val requestId = readString(request, Seq("request_id", "requestId"), "")
        ok(act, "request" -> engine.policyEngine.approval(requestId))

      case "policy-approvals" =>
        val statusRaw = readOptString(request, Seq("status"))
        val status = statusRaw.map(ApprovalStatus.fromString)
        ok(act, "requests" -> engine.policyEngine.approvalsByStatus(status))

      // ----------------------------------------------------------------
      // Match engine
      // ----------------------------------------------------------------
      case "match-register-user" =>
        val user = buildUserSubject(request)
        engine.matchEngine.registerUser(user)
        ok(act, "user" -> user)

      case "match-register-users" =>
        val users = readMapList(request, Seq("users", "user")).map(buildUserSubject)
        engine.matchEngine.registerUsers(users)
        ok(act, "count" -> users.size)

      case "match-register-component" =>
        val component = buildComponentSubject(request)
        engine.matchEngine.registerComponent(component)
        ok(act, "component" -> component)

      case "match-register-components" =>
        val components = readMapList(request, Seq("components", "component")).map(buildComponentSubject)
        engine.matchEngine.registerComponents(components)
        ok(act, "count" -> components.size)

      case "match-register-resource" =>
        val resource = buildResourceSubject(request)
        engine.matchEngine.registerResource(resource)
        ok(act, "resource" -> resource)

      case "match-register-resources" =>
        val resources = readMapList(request, Seq("resources", "resource")).map(buildResourceSubject)
        engine.matchEngine.registerResources(resources)
        ok(act, "count" -> resources.size)

      case "match-register-asset" =>
        val asset = buildAssetSubject(request)
        engine.matchEngine.registerAsset(asset)
        ok(act, "asset" -> asset)

      case "match-register-assets" =>
        val assets = readMapList(request, Seq("assets", "asset")).map(buildAssetSubject)
        engine.matchEngine.registerAssets(assets)
        ok(act, "count" -> assets.size)

      case "match-counts" =>
        ok(act,
          "users" -> engine.matchEngine.userCount,
          "components" -> engine.matchEngine.componentCount,
          "resources" -> engine.matchEngine.resourceCount,
          "assets" -> engine.matchEngine.assetCount
        )

      case "match-user-to-components" =>
        val user = resolveUser(engine, request)
        val candidates = readMapList(request, Seq("components", "component")).map(buildComponentSubject) match {
          case Nil => engine.matchEngine.components
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchUserToComponents(user, candidates, weights, limit))

      case "match-component-to-users" =>
        val component = resolveComponent(engine, request)
        val role = UserRole.fromString(readString(request, Seq("role"), "any"))
        val candidates = readMapList(request, Seq("users", "user")).map(buildUserSubject) match {
          case Nil => engine.matchEngine.users
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchComponentToUsers(component, role, candidates, weights, limit))

      case "match-assets-to-component" =>
        val component = resolveComponent(engine, request)
        val candidates = readMapList(request, Seq("assets", "asset")).map(buildAssetSubject) match {
          case Nil => engine.matchEngine.assets
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchAssetsToComponent(component, candidates, weights, limit))

      case "match-components-to-asset" =>
        val asset = resolveAsset(engine, request)
        val candidates = readMapList(request, Seq("components", "component")).map(buildComponentSubject) match {
          case Nil => engine.matchEngine.components
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchComponentsToAsset(asset, candidates, weights, limit))

      case "match-resources-to-component" =>
        val component = resolveComponent(engine, request)
        val candidates = readMapList(request, Seq("resources", "resource")).map(buildResourceSubject) match {
          case Nil => engine.matchEngine.resources
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchResourcesToComponent(component, candidates, weights, limit))

      case "match-components-to-resource" =>
        val resource = resolveResource(engine, request)
        val candidates = readMapList(request, Seq("components", "component")).map(buildComponentSubject) match {
          case Nil => engine.matchEngine.components
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchComponentsToResource(resource, candidates, weights, limit))

      case "match-profiles-resources" =>
        val profiles = readMapList(request, Seq("profiles", "profile")).map(buildProfile)
        val resources = readMapList(request, Seq("resources", "resource")).map(buildResource)
        if (profiles.isEmpty || resources.isEmpty) err(act, "profiles and resources are required")
        else {
          val matches = engine.matchEngine.matchProfilesToResources(profiles, resources)
          val summary = matches.map { case (p, rs) => p.id -> rs.map(_.id) }
          ok(act, "matches" -> summary)
        }

      case "match-talent" =>
        val requester = resolveUser(engine, request)
        val candidates = readMapList(request, Seq("users", "user")).map(buildUserSubject) match {
          case Nil => engine.matchEngine.users
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.talent
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchTalent(requester, candidates, weights, limit))

      case "match-by-persona" =>
        val persona = PersonaLabel.fromString(readString(request, Seq("persona"), "newcomer")).getOrElse(Newcomer)
        val candidates = readMapList(request, Seq("users", "user")).map(buildUserSubject) match {
          case Nil => engine.matchEngine.users
          case xs  => xs
        }
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "users" -> engine.matchEngine.matchByPersona(persona, candidates, limit))

      case "match-similar-users" =>
        val user = resolveUser(engine, request)
        val candidates = readMapList(request, Seq("users", "user")).map(buildUserSubject) match {
          case Nil => engine.matchEngine.users
          case xs  => xs
        }
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchSimilarUsers(user, candidates, weights, limit))

      case "match-workloads-to-plans" =>
        val requests = readMapList(request, Seq("requests", "workloads")).map(buildOptimizationRequest)
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "results" -> engine.matchEngine.matchWorkloadsToPlans(requests, limit))

      case "match-artifacts-to-user" =>
        val user = resolveUser(engine, request)
        val artifacts = readMapList(request, Seq("artifacts", "artifact")).map(buildAnalyticsArtifact)
        val weightsMap = readMap(request, Seq("weights", "match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.analytics
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "result" -> engine.matchEngine.matchArtifactsToUser(user, artifacts, weights, limit))

      case "match-component-bundle" =>
        val component = resolveComponent(engine, request)
        val role = UserRole.fromString(readString(request, Seq("role"), "any"))
        val userCandidates = readMapList(request, Seq("users", "user")).map(buildUserSubject) match {
          case Nil => engine.matchEngine.users
          case xs  => xs
        }
        val resourceCandidates = readMapList(request, Seq("resources", "resource")).map(buildResourceSubject) match {
          case Nil => engine.matchEngine.resources
          case xs  => xs
        }
        val assetCandidates = readMapList(request, Seq("assets", "asset")).map(buildAssetSubject) match {
          case Nil => engine.matchEngine.assets
          case xs  => xs
        }
        val limit = readInt(request, Seq("limit"), 5)
        ok(act, "bundle" -> engine.matchEngine.componentBundle(component, role, userCandidates, resourceCandidates, assetCandidates, limit))

      case "match-exchange-listings" =>
        val listings = readMapList(request, Seq("listings", "listing")).map(buildComponentSubject) match {
          case Nil => engine.matchEngine.components
          case xs  => xs
        }
        val buyers = readMapList(request, Seq("buyers", "users", "user")).map(buildUserSubject) match {
          case Nil => engine.matchEngine.users
          case xs  => xs
        }
        val limit = readInt(request, Seq("limit"), 10)
        ok(act, "matches" -> engine.matchEngine.matchExchangeListings(listings, buyers, limit))

      // ----------------------------------------------------------------
      // Allocation engine
      // ----------------------------------------------------------------
      case "allocation-score" =>
        val hasListing = readMap(request, Seq("listing")).nonEmpty || request.keys.exists(_.startsWith("listing_"))
        if (!hasListing) err(act, "listing required")
        else ok(act, "scored" -> engine.allocationEngine.score(buildAllocationRequest(request)))

      case "allocation-allocate" =>
        val hasListing = readMap(request, Seq("listing")).nonEmpty || request.keys.exists(_.startsWith("listing_"))
        if (!hasListing) err(act, "listing required")
        else ok(act, "result" -> engine.allocationEngine.allocate(buildAllocationRequest(request)))

      // ----------------------------------------------------------------
      // Incentive engine
      // ----------------------------------------------------------------
      case "incentive-register-participant" =>
        val participantId = readString(request, Seq("participant_id", "participantId"), "")
        val reputation = readDouble(request, Seq("reputation"), 50.0)
        val kp = readDouble(request, Seq("kp"), 0.0)
        ok(act, "profile" -> engine.incentiveEngine.registerParticipant(participantId, reputation, kp))

      case "incentive-profile" =>
        val participantId = readString(request, Seq("participant_id", "participantId"), "")
        ok(act, "profile" -> engine.incentiveEngine.profile(participantId))

      case "incentive-balance" =>
        val participantId = readString(request, Seq("participant_id", "participantId"), "")
        ok(act, "balance" -> engine.incentiveEngine.balance(participantId))

      case "incentive-apply-event" =>
        val event = buildIncentiveEvent(request)
        val applyStreak = readBoolean(request, Seq("apply_streak", "apply_streak_multiplier"), false)
        ok(act, "update" -> engine.incentiveEngine.applyEvent(event, applyStreak))

      case "incentive-apply-incentive" =>
        val incentive = buildGameIncentive(request)
        val referenceId = readString(request, Seq("reference_id"), "")
        val referenceType = readString(request, Seq("reference_type"), "")
        val applyStreak = readBoolean(request, Seq("apply_streak", "apply_streak_multiplier"), false)
        ok(act, "update" -> engine.incentiveEngine.applyIncentive(incentive, referenceId, referenceType, applyStreak))

      case "incentive-apply-incentives" =>
        val incentives = readMapList(request, Seq("incentives", "incentive")).map(buildGameIncentive)
        val referenceId = readString(request, Seq("reference_id"), "")
        val referenceType = readString(request, Seq("reference_type"), "")
        val applyStreak = readBoolean(request, Seq("apply_streak", "apply_streak_multiplier"), false)
        ok(act, "updates" -> engine.incentiveEngine.applyIncentives(incentives, referenceId, referenceType, applyStreak))

      case "incentive-earn" =>
        val participantId = readString(request, Seq("participant_id", "participantId"), "")
        val kp = readDouble(request, Seq("kp"), 0.0)
        val reason = readString(request, Seq("reason"), "")
        val referenceId = readString(request, Seq("reference_id"), "")
        val referenceType = readString(request, Seq("reference_type"), "")
        ok(act, "update" -> engine.incentiveEngine.earn(participantId, kp, reason, referenceId, referenceType))

      case "incentive-penalize" =>
        val participantId = readString(request, Seq("participant_id", "participantId"), "")
        val kpPenalty = readDouble(request, Seq("kp"), 0.0)
        val repPenalty = readDouble(request, Seq("reputation"), 0.0)
        val reason = readString(request, Seq("reason"), "")
        val referenceId = readString(request, Seq("reference_id"), "")
        val referenceType = readString(request, Seq("reference_type"), "")
        ok(act, "update" -> engine.incentiveEngine.penalize(participantId, kpPenalty, repPenalty, reason, referenceId, referenceType))

      case "incentive-redeem" =>
        val participantId = readString(request, Seq("participant_id", "participantId"), "")
        val cost = readDouble(request, Seq("cost"), 0.0)
        val reason = readString(request, Seq("reason"), "")
        val referenceId = readString(request, Seq("reference_id"), "")
        val referenceType = readString(request, Seq("reference_type"), "")
        engine.incentiveEngine.redeem(participantId, cost, reason, referenceId, referenceType) match {
          case Left(errMsg) => err(act, errMsg)
          case Right(update) => ok(act, "update" -> update)
        }

      case "incentive-incentives-for-allocation" =>
        val listingMap = readMap(request, Seq("listing"))
        val allocationMap = readMap(request, Seq("allocation"))
        val policyMap = readMap(request, Seq("policy"))
        if (listingMap.isEmpty || allocationMap.isEmpty) err(act, "listing and allocation required")
        else {
          val listing = buildGameListing(listingMap)
          val allocation = buildGameAllocation(allocationMap)
          val policy = if (policyMap.nonEmpty) buildGameIncentivePolicy(policyMap) else GameIncentivePolicy.default
          ok(act, "incentives" -> engine.incentiveEngine.incentivesForAllocation(listing, allocation, policy))
        }

      case "incentive-ledger" =>
        val limit = readInt(request, Seq("limit"), 200)
        ok(act, "entries" -> engine.incentiveEngine.ledgerEntries(limit))

      // ----------------------------------------------------------------
      // Game engine
      // ----------------------------------------------------------------
      case "game-register-participant" =>
        val user = buildUserSubject(request)
        val reputation = readDouble(request, Seq("reputation"), 50.0)
        val kp = readDouble(request, Seq("kp"), 0.0)
        engine.gameEngine.registerParticipant(user, reputation, kp)
        ok(act, "user" -> user)

      case "game-register-participants" =>
        val users = readMapList(request, Seq("users", "user")).map(buildUserSubject)
        engine.gameEngine.registerParticipants(users)
        ok(act, "count" -> users.size)

      case "game-participant" =>
        val id = readString(request, Seq("id", "participant_id", "participantId"), "")
        ok(act, "participant" -> engine.gameEngine.participant(id))

      case "game-participants" =>
        ok(act, "participants" -> engine.gameEngine.participantsSnapshot)

      case "game-upsert-listing" =>
        val listing = buildGameListing(request)
        engine.gameEngine.upsertListing(listing)
        ok(act, "listing" -> listing)

      case "game-listing" =>
        val id = readString(request, Seq("id", "listing_id", "listingId"), "")
        ok(act, "listing" -> engine.gameEngine.listing(id))

      case "game-close-listing" =>
        val id = readString(request, Seq("id", "listing_id", "listingId"), "")
        ok(act, "listing" -> engine.gameEngine.closeListing(id))

      case "game-suspend-listing" =>
        val id = readString(request, Seq("id", "listing_id", "listingId"), "")
        ok(act, "listing" -> engine.gameEngine.suspendListing(id))

      case "game-listings" =>
        val statusRaw = readOptString(request, Seq("status"))
        val status = statusRaw.map(parseListingStatus)
        ok(act, "listings" -> engine.gameEngine.listingsByStatus(status))

      case "game-submit-bid" =>
        val bid = buildGameBid(request)
        ok(act, "bid" -> engine.gameEngine.submitBid(bid))

      case "game-withdraw-bid" =>
        val id = readString(request, Seq("id", "bid_id", "bidId"), "")
        ok(act, "bid" -> engine.gameEngine.withdrawBid(id))

      case "game-bids" =>
        val listingId = readString(request, Seq("listing_id", "listingId"), "")
        val includeInactive = readBoolean(request, Seq("include_inactive"), false)
        ok(act, "bids" -> engine.gameEngine.bidsForListing(listingId, includeInactive))

      case "game-match-listing" =>
        val listingId = readString(request, Seq("listing_id", "listingId"), "")
        val limit = readInt(request, Seq("limit"), 10)
        val weightsMap = readMap(request, Seq("match_weights", "match-weights"))
        val weights = if (weightsMap.nonEmpty) buildMatchWeights(weightsMap) else MatchWeights.default
        ok(act, "result" -> engine.gameEngine.matchListing(listingId, limit, weights))

      case "game-score-bids" =>
        val listingId = readString(request, Seq("listing_id", "listingId"), "")
        val scoreWeightsMap = readMap(request, Seq("score_weights", "score-weights"))
        val matchWeightsMap = readMap(request, Seq("match_weights", "match-weights"))
        val scoreWeights = if (scoreWeightsMap.nonEmpty) buildGameScoreWeights(scoreWeightsMap) else GameScoreWeights()
        val matchWeights = if (matchWeightsMap.nonEmpty) buildMatchWeights(matchWeightsMap) else MatchWeights.default
        ok(act, "scored" -> engine.gameEngine.scoreBids(listingId, scoreWeights, matchWeights))

      case "game-allocate-listing" =>
        val listingId = readString(request, Seq("listing_id", "listingId"), "")
        val limit = readInt(request, Seq("limit"), 1)
        val scoreWeightsMap = readMap(request, Seq("score_weights", "score-weights"))
        val matchWeightsMap = readMap(request, Seq("match_weights", "match-weights"))
        val policyMap = readMap(request, Seq("policy"))
        val applyIncentives = readBoolean(request, Seq("apply_incentives"), true)
        val scoreWeights = if (scoreWeightsMap.nonEmpty) buildGameScoreWeights(scoreWeightsMap) else GameScoreWeights()
        val matchWeights = if (matchWeightsMap.nonEmpty) buildMatchWeights(matchWeightsMap) else MatchWeights.default
        val policy = if (policyMap.nonEmpty) buildGameIncentivePolicy(policyMap) else GameIncentivePolicy.default
        ok(act, "result" -> engine.gameEngine.allocateListing(listingId, limit, scoreWeights, matchWeights, policy, applyIncentives))

      case "game-award-incentives" =>
        val listingMap = readMap(request, Seq("listing"))
        val allocationMap = readMap(request, Seq("allocation"))
        val policyMap = readMap(request, Seq("policy"))
        if (listingMap.isEmpty || allocationMap.isEmpty) err(act, "listing and allocation required")
        else {
          val listing = buildGameListing(listingMap)
          val allocation = buildGameAllocation(allocationMap)
          val policy = if (policyMap.nonEmpty) buildGameIncentivePolicy(policyMap) else GameIncentivePolicy.default
          ok(act, "incentives" -> engine.gameEngine.awardIncentives(listing, allocation, policy))
        }

      case "game-snapshot" =>
        ok(act, "snapshot" -> engine.gameEngine.snapshot())

      case "game-events" =>
        val limit = readInt(request, Seq("limit"), 200)
        ok(act, "events" -> engine.gameEngine.eventLedger(limit))

      // ----------------------------------------------------------------
      // Graph engine
      // ----------------------------------------------------------------
      case "graph-add-edge" =>
        val from = readString(request, Seq("from"), "")
        val to = readString(request, Seq("to"), "")
        if (from.isEmpty || to.isEmpty) err(act, "missing from/to")
        else {
          val edgeType = parseEdgeType(readString(request, Seq("edge_type", "edge-type"), "dependency"))
          val weight = readDouble(request, Seq("weight"), 1.0)
          engine.graphAddEdge(from, to, edgeType, weight)
          ok(act, "from" -> from, "to" -> to, "edge_type" -> edgeType.toString,
            "weight" -> weight, "total_edges" -> engine.graphEdges.size)
        }

      case "graph-add-node" =>
        val id = readString(request, Seq("id"), "")
        if (id.isEmpty) err(act, "missing id")
        else {
          val duration = readDouble(request, Seq("duration"), 0.0)
          engine.graphAddNode(id, duration)
          ok(act, "id" -> id, "duration" -> duration, "total_nodes" -> engine.graphNodes.size)
        }

      case "graph-remove-edge" =>
        val from = readString(request, Seq("from"), "")
        val to = readString(request, Seq("to"), "")
        if (from.isEmpty || to.isEmpty) err(act, "missing from/to")
        else {
          engine.graphRemoveEdge(from, to)
          ok(act, "from" -> from, "to" -> to, "total_edges" -> engine.graphEdges.size)
        }

      case "graph-remove-node" =>
        val id = readString(request, Seq("id"), "")
        if (id.isEmpty) err(act, "missing id")
        else {
          engine.graphRemoveNode(id)
          ok(act, "id" -> id, "total_nodes" -> engine.graphNodes.size)
        }

      case "graph-load" =>
        val edges = readMapList(request, Seq("edges", "edge")).map(buildGraphEdge)
        val nodes = readMapList(request, Seq("nodes", "node")).map(buildGraphNode)
        engine.graphLoad(edges, nodes)
        ok(act, "edges" -> edges.size, "nodes" -> nodes.size)

      case "graph-traverse" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node")
        else {
          val edgeTypeRaw = readString(request, Seq("edge_type", "edge-type"), "any")
          val filter: EdgeType => Boolean = edgeTypeRaw.toLowerCase match {
            case "any" => _ => true
            case _ => e => e == parseEdgeType(edgeTypeRaw)
          }
          val result = engine.graphEngine.traverseFrom(nodeId, filter)
          ok(act, "node" -> nodeId, "traverse" -> result.toSeq.sorted)
        }

      case "graph-rtraverse" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node")
        else {
          val edgeTypeRaw = readString(request, Seq("edge_type", "edge-type"), "any")
          val filter: EdgeType => Boolean = edgeTypeRaw.toLowerCase match {
            case "any" => _ => true
            case _ => e => e == parseEdgeType(edgeTypeRaw)
          }
          val result = engine.graphEngine.reverseTraverseFrom(nodeId, filter)
          ok(act, "node" -> nodeId, "traverse" -> result.toSeq.sorted)
        }

      case "graph-impact" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node") else graphImpactResponse(engine.graphImpact(nodeId))

      case "graph-closure" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node")
        else ok(act, "node" -> nodeId, "closure" -> engine.graphDependencyClosure(nodeId).toSeq.sorted)

      case "graph-rdeps" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node")
        else ok(act, "node" -> nodeId, "reverse_closure" -> engine.graphReverseClosure(nodeId).toSeq.sorted)

      case "graph-neighbors" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node")
        else ok(act, "node" -> nodeId, "neighbors" -> engine.graphNeighborhood(nodeId).toSeq.sorted)

      case "graph-ancestors" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node")
        else ok(act, "node" -> nodeId, "ancestors" -> engine.graphAncestors(nodeId).toSeq.sorted)

      case "graph-descendants" =>
        val nodeId = readString(request, Seq("node"), "")
        if (nodeId.isEmpty) err(act, "missing node")
        else ok(act, "node" -> nodeId, "descendants" -> engine.graphDescendants(nodeId).toSeq.sorted)

      case "graph-cycles" =>
        graphCyclesResponse(engine.graphCycles())

      case "graph-topo" =>
        engine.graphTopologicalSort() match {
          case Right(order) => ok(act, "has_cycle" -> false, "order" -> order)
          case Left(cycleNodes) => err(act, "topological sort requires a DAG", "has_cycle" -> true, "cycle_nodes" -> cycleNodes.toSeq.sorted)
        }

      case "graph-critical" =>
        engine.graphCriticalPath() match {
          case Some(report) => graphCriticalResponse(report)
          case None => err(act, "critical path is undefined: cycle detected in graph")
        }

      case "graph-diff" =>
        val beforeEdges = readMapList(request, Seq("before_edges", "beforeEdges")).map(buildGraphEdge)
        val tupleEdges = request.collect { case (k, v) if k.startsWith("before-edge-") =>
          parseEdgeTuple(asString(v))
        }.flatten
        val allBefore = if (beforeEdges.nonEmpty) beforeEdges else tupleEdges.toSeq
        val beforeEngine = new GraphEngine(allBefore)
        graphDiffResponse(beforeEngine.diff(engine.graphEngine))

      case "graph-nodes" =>
        ok(act, "count" -> engine.graphNodes.size, "nodes" -> engine.graphNodes.toSeq.sorted)

      case "graph-edges" =>
        ok(act, "count" -> engine.graphEdges.size,
          "edges" -> engine.graphEdges.map(e => Map("from" -> e.from, "to" -> e.to,
            "edge_type" -> e.edgeType.toString, "weight" -> e.weight)))

      case _ =>
        err(act, s"unknown action: $act")
    }
  }

  // ----------------------------------------------------------------
  // Helpers
  // ----------------------------------------------------------------
  private def ok(action: String, fields: (String, Any)*): Map[String, Any] =
    Map("status" -> "ok", "action" -> action) ++
      fields.map { case (k, v) => k -> EngineValueCodec.normalize(v) }.toMap

  private def err(action: String, message: String, fields: (String, Any)*): Map[String, Any] =
    Map("status" -> "error", "action" -> action, "error" -> message) ++
      fields.map { case (k, v) => k -> EngineValueCodec.normalize(v) }.toMap

  private def personaFor(engine: KogiEngine, profileId: String, request: Map[String, Any]): PersonaLabel =
    readOptString(request, Seq("persona"))
      .map(PersonaLabel.fromString)
      .flatten
      .orElse(engine.recommendationEngine.getProfile(profileId).map(_.persona))
      .getOrElse(Newcomer)

  private def resolveUser(engine: KogiEngine, request: Map[String, Any]): UserSubject = {
    val id = readString(request, Seq("user_id", "userId", "id"), "")
    val useRegistry = readBoolean(request, Seq("use_registry", "registry"), true)
    if (useRegistry && id.nonEmpty) engine.matchEngine.user(id).getOrElse(buildUserSubject(request))
    else buildUserSubject(request)
  }

  private def resolveComponent(engine: KogiEngine, request: Map[String, Any]): ComponentSubject = {
    val id = readString(request, Seq("component_id", "componentId", "id"), "")
    val useRegistry = readBoolean(request, Seq("use_registry", "registry"), true)
    if (useRegistry && id.nonEmpty) engine.matchEngine.component(id).getOrElse(buildComponentSubject(request))
    else buildComponentSubject(request)
  }

  private def resolveResource(engine: KogiEngine, request: Map[String, Any]): ResourceSubject = {
    val id = readString(request, Seq("resource_id", "resourceId", "id"), "")
    val useRegistry = readBoolean(request, Seq("use_registry", "registry"), true)
    if (useRegistry && id.nonEmpty) engine.matchEngine.resource(id).getOrElse(buildResourceSubject(request))
    else buildResourceSubject(request)
  }

  private def resolveAsset(engine: KogiEngine, request: Map[String, Any]): AssetSubject = {
    val id = readString(request, Seq("asset_id", "assetId", "id"), "")
    val useRegistry = readBoolean(request, Seq("use_registry", "registry"), true)
    if (useRegistry && id.nonEmpty) engine.matchEngine.asset(id).getOrElse(buildAssetSubject(request))
    else buildAssetSubject(request)
  }

  private def policyDecisionMap(decision: PolicyDecision): Map[String, Any] =
    Map("kind" -> decision.kind, "reason" -> decision.reason.orNull)

  private def policyRuleSummary(rule: PolicyRule): Map[String, Any] = rule match {
    case s: SimplePolicyRule =>
      Map("id" -> s.id, "name" -> s.name, "description" -> s.description,
        "decision" -> policyDecisionMap(s.decision))
    case other =>
      Map("id" -> other.id, "name" -> other.name)
  }

  private def parseEdgeType(raw: String): EdgeType = raw.trim.toLowerCase match {
    case "hierarchy"    => Hierarchy
    case "relationship" => Relationship
    case _              => Dependency
  }

  private def parseListingStatus(raw: String): GameListingStatus = raw.trim.toLowerCase match {
    case "closed"    => ListingClosed
    case "filled"    => ListingFilled
    case "suspended" => ListingSuspended
    case _            => ListingOpen
  }

  private def parseItemMeta(raw: String): Map[String, Map[String, String]] = {
    if (raw == null || raw.trim.isEmpty) return Map.empty
    raw
      .split("\\|")
      .toList
      .map(_.trim)
      .filter(_.nonEmpty)
      .flatMap { entry =>
        val parts = entry.split(":", 2)
        if (parts.length != 2) None
        else {
          val id = parts(0).trim
          val meta = parseMapString(parts(1).replace(';', ','))
          if (id.isEmpty) None else Some(id -> meta)
        }
      }
      .toMap
  }

  private def parseEdgeTuple(value: String): Option[GraphEdge] = {
    val parts = value.split(":")
    if (parts.length >= 2) {
      val edgeType = if (parts.length >= 3) parseEdgeType(parts(2)) else Dependency
      val weight   = if (parts.length >= 4) parts(3).toDoubleOption.getOrElse(1.0) else 1.0
      Some(GraphEdge(parts(0), parts(1), edgeType, weight))
    } else None
  }

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")

  // ----------------------------------------------------------------
  // Response builders (kept consistent with existing CLI output)
  // ----------------------------------------------------------------
  private def controlResponse(status: EngineControlStatus): Map[String, Any] =
    Map(
      "engine"       -> "kogi-engine",
      "status"       -> "ok",
      "mode"         -> status.mode,
      "changed"      -> status.changed,
      "timestamp_ms" -> status.timestampMs
    )

  private def snapshotResponse(snapshot: EngineFlowSnapshot): Map[String, Any] =
    Map(
      "engine"           -> "kogi-engine",
      "status"           -> "ok",
      "total_envelopes"  -> snapshot.totalEnvelopes,
      "observed_topics"  -> snapshot.observedTopics,
      "generated_at_ms"  -> snapshot.generatedAtMs,
      "host" -> Map(
        "host_id"        -> snapshot.system.host.hostId,
        "status"         -> snapshot.system.host.status,
        "saturation"     -> snapshot.system.host.saturationScore,
        "cpu_avg_pct"    -> snapshot.system.host.cpuAvgPct,
        "memory_avg_pct" -> snapshot.system.host.memoryAvgPct
      ),
      "recommendations"  -> snapshot.system.recommendations.map(_.message)
    )

  private def planResponse(plan: QueryPlan): Map[String, Any] =
    Map(
      "engine" -> "kogi-engine",
      "status" -> "ok",
      "query" -> Map(
        "statement"       -> plan.analysis.statementType,
        "normalized_sql"  -> plan.analysis.normalizedSql,
        "tables"          -> plan.analysis.tables,
        "warnings"        -> plan.analysis.warnings,
        "estimated_cost"  -> plan.analysis.estimatedCost,
        "optimized_sql"   -> plan.optimizedSql,
        "hints"           -> plan.hints
      ),
      "generated_at_ms" -> plan.generatedAtMs
    )

  private def searchResponse(result: SearchResult): Map[String, Any] =
    Map(
      "engine"  -> "kogi-engine",
      "status"  -> "ok",
      "query"   -> result.query.text,
      "total"   -> result.total,
      "matches" -> result.matches.map { m =>
        Map(
          "id"         -> m.document.id,
          "title"      -> m.document.title,
          "score"      -> m.score,
          "highlights" -> m.highlights
        )
      },
      "generated_at_ms" -> result.generatedAtMs
    )

  private def graphImpactResponse(r: ImpactReport): Map[String, Any] =
    Map(
      "status"     -> "ok",
      "action"     -> "graph-impact",
      "root"       -> r.root,
      "affected"   -> r.affected.toSeq.sorted,
      "dependents" -> r.dependents.toSeq.sorted
    )

  private def graphCyclesResponse(r: CycleReport): Map[String, Any] =
    Map(
      "status"      -> "ok",
      "action"      -> "graph-cycles",
      "has_cycles"  -> r.hasCycles,
      "cycle_count" -> r.cycles.size,
      "cycles"      -> r.cycles.map(_.mkString(" -> "))
    )

  private def graphCriticalResponse(r: CriticalPathReport): Map[String, Any] =
    Map(
      "status"     -> "ok",
      "action"     -> "graph-critical",
      "path"       -> r.path,
      "total_cost" -> r.totalCost,
      "node_slack" -> r.nodeSlack.map { case (k, v) => k -> v }
    )

  private def graphDiffResponse(d: GraphDiff): Map[String, Any] =
    Map(
      "status"        -> "ok",
      "action"        -> "graph-diff",
      "added_nodes"   -> d.addedNodes.toSeq.sorted,
      "removed_nodes" -> d.removedNodes.toSeq.sorted,
      "added_edges"   -> d.addedEdges.map(e => s"${e.from}->${e.to}").toSeq.sorted,
      "removed_edges" -> d.removedEdges.map(e => s"${e.from}->${e.to}").toSeq.sorted,
      "changed_edges" -> d.changedEdges.map { case (o, n) =>
        Map("from" -> o.from, "to" -> o.to,
          "old_type" -> o.edgeType.toString, "new_type" -> n.edgeType.toString,
          "old_weight" -> o.weight, "new_weight" -> n.weight)
      }.toSeq
    )
}
