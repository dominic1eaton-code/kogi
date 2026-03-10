package kogi.engine

final class RecommendationEngine(
    recommendationLimit: Int = 8,
    discoverLimit: Int = 8,
    exploreLimit: Int = 8
) {
  def analyticsRecommendations(
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
      .take(recommendationLimit)
  }

  def analyticsDiscover(
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

    cards.toList.take(discoverLimit)
  }

  def analyticsExplore(
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

    cards.toList.take(exploreLimit)
  }

  def moduleRecommendations(
      module: String,
      signal: PortfolioSignal,
      health: PortfolioHealthScore,
      events: Seq[StreamEvent],
      moduleActivity: List[ModuleActivity],
      errorRate: Double,
      p95LatencyMs: Double,
      avgQueueDepth: Double,
      avgCpuPct: Double
  ): List[RecommendationCard] = {
    val recommendations =
      analyticsRecommendations(signal, health, events, moduleActivity) ++
        moduleOperationalRecommendations(module, errorRate, p95LatencyMs, avgQueueDepth, avgCpuPct)

    recommendations
      .groupBy(_.message)
      .values
      .map(_.head)
      .toList
      .sortBy(card => priorityRank(card.priority))
      .take(recommendationLimit)
  }

  def moduleDiscover(
      module: String,
      metrics: Seq[ModuleMetricSample],
      events: Seq[StreamEvent],
      moduleActivity: List[ModuleActivity]
  ): List[DiscoverCard] = {
    (analyticsDiscover(events, moduleActivity) ++ moduleDiscoverInternal(module, metrics))
      .take(discoverLimit)
  }

  def moduleExplore(
      module: String,
      errorRate: Double,
      blockedRate: Double,
      events: Seq[StreamEvent],
      moduleActivity: List[ModuleActivity],
      recommendations: List[RecommendationCard]
  ): List[ExploreCard] = {
    (analyticsExplore(events, moduleActivity, recommendations) ++ moduleExploreInternal(module, errorRate, blockedRate))
      .take(exploreLimit)
  }

  def systemRecommendations(
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
      .take(recommendationLimit)
  }

  def systemDiscover(
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

    cards.toList.take(discoverLimit)
  }

  def systemExplore(
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

    cards.toList.take(exploreLimit)
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

  private def moduleDiscoverInternal(
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

  private def moduleExploreInternal(
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

  private def containsAny(value: String, parts: String*): Boolean = {
    val normalized = value.toLowerCase
    parts.exists(normalized.contains)
  }

  private def priorityRank(priority: String): Int = priority match {
    case "high" | "red"     => 0
    case "medium" | "amber" => 1
    case _                  => 2
  }

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")

  private def average(values: Seq[Double]): Double =
    if (values.isEmpty) 0.0 else values.sum / values.size.toDouble
}
