package kogi.engine

final case class OptimizationRequest(
    workloadId: String,
    cpuPct: Double,
    memoryPct: Double,
    queueDepth: Double,
    latencyMs: Double,
    objectives: List[String] = Nil
)

final case class OptimizationRecommendation(
    id: String,
    priority: String,
    message: String
)

final case class OptimizationPlan(
    request: OptimizationRequest,
    score: Double,
    recommendations: List[OptimizationRecommendation],
    generatedAtMs: Long
)

final class OptimizationEngine {
  def optimize(request: OptimizationRequest): OptimizationPlan = {
    val buffer = scala.collection.mutable.ListBuffer.empty[OptimizationRecommendation]

    if (request.cpuPct >= 80.0) {
      buffer += OptimizationRecommendation(
        id = "opt-cpu-001",
        priority = "high",
        message = "CPU saturation detected; consider horizontal scaling or workload shedding."
      )
    }

    if (request.memoryPct >= 80.0) {
      buffer += OptimizationRecommendation(
        id = "opt-mem-001",
        priority = "high",
        message = "Memory pressure detected; tune caches and reclaim inactive workers."
      )
    }

    if (request.queueDepth >= 150.0) {
      buffer += OptimizationRecommendation(
        id = "opt-queue-001",
        priority = "medium",
        message = "Queue depth elevated; increase consumer concurrency or introduce backpressure."
      )
    }

    if (request.latencyMs >= 1200.0) {
      buffer += OptimizationRecommendation(
        id = "opt-latency-001",
        priority = "medium",
        message = "Latency p95 above threshold; identify slow handlers and optimize hot paths."
      )
    }

    val score = scorePlan(request)

    OptimizationPlan(
      request = request,
      score = score,
      recommendations = buffer.toList.sortBy(rec => priorityRank(rec.priority)),
      generatedAtMs = System.currentTimeMillis()
    )
  }

  private def scorePlan(request: OptimizationRequest): Double = {
    val cpuScore = clamp(100.0 - request.cpuPct)
    val memScore = clamp(100.0 - request.memoryPct)
    val queueScore = clamp(100.0 - (request.queueDepth / 2.0))
    val latencyScore = clamp(100.0 - (request.latencyMs / 20.0))

    (cpuScore * 0.3) + (memScore * 0.25) + (queueScore * 0.2) + (latencyScore * 0.25)
  }

  private def clamp(value: Double): Double = math.max(0.0, math.min(100.0, value))

  private def priorityRank(priority: String): Int = priority match {
    case "high" | "red"     => 0
    case "medium" | "amber" => 1
    case _                  => 2
  }
}
