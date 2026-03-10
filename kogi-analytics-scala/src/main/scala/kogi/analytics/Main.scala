package kogi.analytics

object Main {
  def main(args: Array[String]): Unit = {
    val engine = new AnalyticsRecommendationDiscoverExploreEngine()

    val now = System.currentTimeMillis()
    val stream = Seq(
      StreamEvent(
        id = "evt-001",
        profileId = "profile-work-001",
        module = "office",
        eventType = "task_completed",
        timestampMs = now - 5000,
        productivityDelta = 4.0,
        collaborationDelta = 1.1
      ),
      StreamEvent(
        id = "evt-002",
        profileId = "profile-work-001",
        module = "office",
        eventType = "story_blocked",
        timestampMs = now - 4000,
        productivityDelta = -1.2,
        riskDelta = 1.8
      ),
      StreamEvent(
        id = "evt-003",
        profileId = "profile-work-001",
        module = "exchange",
        eventType = "deal_won",
        timestampMs = now - 3000,
        cashFlowDelta = 6.4,
        metadata = Map("provider" -> "coinbase")
      ),
      StreamEvent(
        id = "evt-004",
        profileId = "profile-work-001",
        module = "community",
        eventType = "message_sent",
        timestampMs = now - 2000,
        collaborationDelta = 2.2,
        metadata = Map("provider" -> "slack")
      ),
      StreamEvent(
        id = "evt-005",
        profileId = "profile-work-001",
        module = "studio",
        eventType = "prototype_created",
        timestampMs = now - 1000,
        productivityDelta = 1.8,
        riskDelta = 0.4
      )
    )

    val snapshots = engine.ingestBatch(stream)
    val latest = snapshots.last

    val moduleMetrics = Seq(
      ModuleMetricSample(
        id = "mm-001",
        module = "office",
        timestampMs = now - 3000,
        latencyMs = 210.0,
        queueDepth = 24,
        cpuPct = 54.0,
        memoryMb = 612.0,
        metadata = Map("route" -> "/api/v1/office/dashboard")
      ),
      ModuleMetricSample(
        id = "mm-002",
        module = "exchange",
        timestampMs = now - 2000,
        latencyMs = 480.0,
        queueDepth = 68,
        errorCount = 1,
        successCount = 6,
        cpuPct = 72.0,
        memoryMb = 712.0,
        metadata = Map("route" -> "/api/v1/exchange/deals")
      ),
      ModuleMetricSample(
        id = "mm-003",
        module = "kogi-host",
        timestampMs = now - 1000,
        latencyMs = 150.0,
        queueDepth = 11,
        cpuPct = 44.0,
        memoryMb = 498.0,
        metadata = Map("route" -> "host.scheduler.tick")
      )
    )

    moduleMetrics.foreach(engine.ingestModuleMetric)

    val hostMetrics = Seq(
      HostMetricSample(
        id = "hm-001",
        hostId = "kogi-host-001",
        timestampMs = now - 3000,
        cpuPct = 61.0,
        memoryPct = 57.0,
        diskPct = 49.0,
        processCount = 212,
        schedulerLoad = 0.53,
        networkInKbps = 580.0,
        networkOutKbps = 620.0
      ),
      HostMetricSample(
        id = "hm-002",
        hostId = "kogi-host-001",
        timestampMs = now - 1000,
        cpuPct = 67.0,
        memoryPct = 63.0,
        diskPct = 51.0,
        processCount = 225,
        schedulerLoad = 0.59,
        networkInKbps = 640.0,
        networkOutKbps = 710.0
      )
    )

    hostMetrics.foreach(engine.ingestHostMetric)
    val fromBus = engine.ingestBusEvent(
      topic = "gateway.office.request",
      payload = Map(
        "event_type" -> "request_completed",
        "module" -> "office",
        "latency_ms" -> "190",
        "queue_depth" -> "18",
        "cpu_pct" -> "52",
        "memory_mb" -> "640",
        "host_cpu_pct" -> "65",
        "host_memory_pct" -> "62",
        "host_disk_pct" -> "52",
        "host_process_count" -> "231",
        "host_scheduler_load" -> "0.61"
      ),
      hostId = "kogi-host-001",
      timestampMs = now
    )
    val system = engine.systemSnapshot(hostId = "kogi-host-001")

    println(f"kogi-analytics status=${latest.health.status} overall=${latest.health.overall}%.2f")
    println(
      f"signal productivity=${latest.signal.productivity}%.2f cashFlow=${latest.signal.cashFlow}%.2f collaboration=${latest.signal.collaboration}%.2f risk=${latest.signal.risk}%.2f"
    )

    println("recommendations:")
    latest.recommendations.foreach(card => println(s"- [${card.priority}] ${card.message}"))

    println("discover:")
    latest.discover.foreach(card => println(s"- ${card.title}: ${card.evidence}"))

    println("explore:")
    latest.explore.foreach(card => println(s"- ${card.title}: ${card.nextAction}"))

    println("module-activity:")
    latest.modules.foreach(module => println(f"- ${module.module}: events=${module.events} impact=${module.netImpact}%.2f"))

    println("system-host:")
    println(
      f"- host=${system.host.hostId} status=${system.host.status} saturation=${system.host.saturationScore}%.2f cpu=${system.host.cpuAvgPct}%.2f mem=${system.host.memoryAvgPct}%.2f"
    )

    println("module-realtime:")
    system.modules.foreach { module =>
      println(
        f"- ${module.module}: events=${module.eventCount} throughput=${module.throughputPerMin}%.2f/min error=${module.errorRate * 100.0}%.2f%% p95=${module.p95LatencyMs}%.2fms health=${module.health.status}"
      )
    }

    println("module-catalog:")
    println(s"- ${engine.moduleCatalog().mkString(", ")}")

    println("bus-ingest-snapshot:")
    println(
      f"- host=${fromBus.host.hostId} status=${fromBus.host.status} modules=${fromBus.modules.size} recommendations=${fromBus.recommendations.size}"
    )
  }
}
