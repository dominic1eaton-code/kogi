package kogi.engine

final case class OptimizationRequest(
    workloadId: String,
    cpuPct: Double,
    memoryPct: Double,
    queueDepth: Double,
    latencyMs: Double,
    objectives: List[String] = Nil,
    persona: Option[PersonaLabel] = None  // drives persona-aware recommendations
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

    if (request.cpuPct >= 80.0)
      buffer += OptimizationRecommendation("opt-cpu-001", "high",
        "CPU saturation detected; consider horizontal scaling or workload shedding.")
    if (request.memoryPct >= 80.0)
      buffer += OptimizationRecommendation("opt-mem-001", "high",
        "Memory pressure detected; tune caches and reclaim inactive workers.")
    if (request.queueDepth >= 150.0)
      buffer += OptimizationRecommendation("opt-queue-001", "medium",
        "Queue depth elevated; increase consumer concurrency or introduce backpressure.")
    if (request.latencyMs >= 1200.0)
      buffer += OptimizationRecommendation("opt-latency-001", "medium",
        "Latency p95 above threshold; identify slow handlers and optimize hot paths.")

    // Persona-aware objectives
    val personaRecs = request.persona.map(personaObjectives(request, _)).getOrElse(Nil)
    buffer ++= personaRecs

    OptimizationPlan(
      request = request,
      score   = scorePlan(request),
      recommendations = buffer.toList.sortBy(r => priorityRank(r.priority)),
      generatedAtMs = System.currentTimeMillis()
    )
  }

  /**
   * Returns persona-tuned recommendations on top of standard ones.
   * PowerUsers get throughput hints; CasualBrowsers get simplification hints.
   */
  private def personaObjectives(
      request: OptimizationRequest,
      persona: PersonaLabel
  ): List[OptimizationRecommendation] = persona match {
    case PowerUser if request.latencyMs > 800.0 =>
      List(OptimizationRecommendation("opt-persona-power-latency", "medium",
        "Power user profile: prioritize p95 latency reduction over memory savings."))
    case CasualBrowser if request.cpuPct > 60.0 =>
      List(OptimizationRecommendation("opt-persona-casual-cpu", "low",
        "Casual profile: defer background jobs during low-engagement windows to reduce CPU."))
    case Explorer if request.queueDepth > 80.0 =>
      List(OptimizationRecommendation("opt-persona-explorer-queue", "low",
        "Explorer profile: widen discovery pipeline concurrency to serve broad content requests."))
    case _ => Nil
  }

  private def scorePlan(request: OptimizationRequest): Double = {
    val cpuScore     = clamp(100.0 - request.cpuPct)
    val memScore     = clamp(100.0 - request.memoryPct)
    val queueScore   = clamp(100.0 - (request.queueDepth / 2.0))
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
