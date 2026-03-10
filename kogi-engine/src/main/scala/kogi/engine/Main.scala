package kogi.engine

object Main {
  def main(args: Array[String]): Unit = {
    val engine = new KogiEngine()
    val now = System.currentTimeMillis()

    val flows = Seq(
      PlatformDataEnvelope(
        id = "env-001",
        flowId = "flow-kernel-host",
        source = "kogi-kernel",
        target = "kogi-host",
        topic = "kernel.component.registered",
        payload = Map(
          "module" -> "kogi-kernel",
          "event_type" -> "component_registered",
          "latency_ms" -> "24",
          "queue_depth" -> "2",
          "cpu_pct" -> "22",
          "memory_mb" -> "188",
          "host_id" -> "kogi-host-001",
          "host_cpu_pct" -> "48",
          "host_memory_pct" -> "45",
          "host_disk_pct" -> "39",
          "host_process_count" -> "172",
          "productivity_delta" -> "1.2",
          "collaboration_delta" -> "0.8"
        ),
        timestampMs = now - 4000
      ),
      PlatformDataEnvelope(
        id = "env-002",
        flowId = "flow-host-server",
        source = "kogi-host",
        target = "kogi-server",
        topic = "host.orchestrator.dispatch",
        payload = Map(
          "module" -> "kogi-host",
          "event_type" -> "dispatch_completed",
          "latency_ms" -> "42",
          "queue_depth" -> "6",
          "cpu_pct" -> "36",
          "memory_mb" -> "264",
          "host_id" -> "kogi-host-001",
          "host_cpu_pct" -> "54",
          "host_memory_pct" -> "52",
          "host_disk_pct" -> "40",
          "host_process_count" -> "185",
          "productivity_delta" -> "1.8"
        ),
        timestampMs = now - 3000
      ),
      PlatformDataEnvelope(
        id = "env-003",
        flowId = "flow-server-gateway",
        source = "kogi-server",
        target = "gateway",
        topic = "server.gateway.route",
        payload = Map(
          "module" -> "kogi-server",
          "event_type" -> "route_ready",
          "latency_ms" -> "95",
          "queue_depth" -> "12",
          "cpu_pct" -> "44",
          "memory_mb" -> "382",
          "host_id" -> "kogi-host-001",
          "host_cpu_pct" -> "59",
          "host_memory_pct" -> "55",
          "host_disk_pct" -> "43",
          "host_process_count" -> "196",
          "collaboration_delta" -> "1.4"
        ),
        timestampMs = now - 2000
      ),
      PlatformDataEnvelope(
        id = "env-004",
        flowId = "flow-gateway-office",
        source = "gateway",
        target = "service.office",
        topic = "gateway.office.request",
        payload = Map(
          "module" -> "office",
          "event_type" -> "request_completed",
          "latency_ms" -> "176",
          "queue_depth" -> "18",
          "cpu_pct" -> "58",
          "memory_mb" -> "644",
          "host_id" -> "kogi-host-001",
          "host_cpu_pct" -> "64",
          "host_memory_pct" -> "60",
          "host_disk_pct" -> "47",
          "host_process_count" -> "214",
          "productivity_delta" -> "3.2",
          "cashflow_delta" -> "2.5",
          "collaboration_delta" -> "2.1",
          "risk_delta" -> "0.7"
        ),
        timestampMs = now - 1000
      ),
      PlatformDataEnvelope(
        id = "env-005",
        flowId = "flow-office-engine",
        source = "service.office",
        target = "kogi-engine",
        topic = "office.analytics.snapshot",
        payload = Map(
          "module" -> "office",
          "event_type" -> "snapshot_emitted",
          "latency_ms" -> "58",
          "queue_depth" -> "4",
          "cpu_pct" -> "42",
          "memory_mb" -> "520",
          "throughput_units" -> "12",
          "host_id" -> "kogi-host-001",
          "host_cpu_pct" -> "66",
          "host_memory_pct" -> "62",
          "host_disk_pct" -> "48",
          "host_process_count" -> "223",
          "productivity_delta" -> "2.4",
          "cashflow_delta" -> "1.1",
          "collaboration_delta" -> "1.5"
        ),
        timestampMs = now
      )
    )

    flows.foreach(engine.ingestEnvelope)
    val snapshot = engine.snapshot(hostId = "kogi-host-001")

    println(s"kogi-engine flow envelopes=${snapshot.totalEnvelopes} observed_topics=${snapshot.observedTopics.size}")
    println(
      f"host=${snapshot.system.host.hostId} status=${snapshot.system.host.status} saturation=${snapshot.system.host.saturationScore}%.2f cpu=${snapshot.system.host.cpuAvgPct}%.2f mem=${snapshot.system.host.memoryAvgPct}%.2f"
    )

    println("module-realtime:")
    snapshot.system.modules.foreach { module =>
      println(
        f"- ${module.module}: throughput=${module.throughputPerMin}%.2f/min error=${module.errorRate * 100.0}%.2f%% p95=${module.p95LatencyMs}%.2fms health=${module.health.status}"
      )
    }

    println("component-flow-stats:")
    snapshot.components.foreach { component =>
      println(
        f"- ${component.component}: in=${component.ingressCount} out=${component.egressCount} avg_latency=${component.avgLatencyMs}%.2fms errors=${component.errorCount}"
      )
    }

    println("system-recommendations:")
    snapshot.system.recommendations.foreach(card => println(s"- [${card.priority}] ${card.message}"))
  }
}
