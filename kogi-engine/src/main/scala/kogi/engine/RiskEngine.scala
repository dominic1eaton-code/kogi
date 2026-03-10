package kogi.engine

final case class PortfolioSignal(
    productivity: Double,
    cashFlow: Double,
    collaboration: Double,
    risk: Double
)

final case class PortfolioHealthScore(
    overall: Double,
    status: String,
    recommendations: List[String]
)

final case class RiskOptimizationRequest(
    profileId: String,
    signal: PortfolioSignal,
    context: Map[String, String] = Map.empty
)

final case class RiskOptimizationRecommendation(
    id: String,
    priority: String,
    message: String
)

final case class RiskOptimizationPlan(
    profileId: String,
    signal: PortfolioSignal,
    health: PortfolioHealthScore,
    riskScore: Double,
    recommendations: List[RiskOptimizationRecommendation],
    generatedAtMs: Long
)

final class RiskEngine {
  def score(signal: PortfolioSignal): PortfolioHealthScore = {
    val productivityW = 0.30
    val cashFlowW = 0.30
    val collaborationW = 0.20
    val riskW = 0.20

    val weighted =
      signal.productivity * productivityW +
        signal.cashFlow * cashFlowW +
        signal.collaboration * collaborationW +
        (100.0 - signal.risk) * riskW

    val status =
      if (weighted >= 80) "green"
      else if (weighted >= 60) "amber"
      else "red"

    val recommendations = List(
      if (signal.risk > 65) Some("reduce risk exposure in exchange and campaign modules") else None,
      if (signal.cashFlow < 50) Some("prioritize paid backlog stories and wallet recovery") else None,
      if (signal.productivity < 50) Some("rebalance sprint workload across teams") else None
    ).flatten

    PortfolioHealthScore(weighted, status, recommendations)
  }

  def optimize(request: RiskOptimizationRequest): RiskOptimizationPlan = {
    val health = score(request.signal)
    val riskScore = clamp(100.0 - request.signal.risk)
    val buffer = scala.collection.mutable.ListBuffer.empty[RiskOptimizationRecommendation]

    if (request.signal.risk >= 70.0) {
      buffer += RiskOptimizationRecommendation(
        id = "risk-hot-001",
        priority = "high",
        message = "High risk exposure detected; stabilize exchange, wallet, and compliance workflows."
      )
    }

    if (request.signal.cashFlow < 50.0) {
      buffer += RiskOptimizationRecommendation(
        id = "risk-cashflow-001",
        priority = "high",
        message = "Cashflow signal is weak; tighten receivables and review overdue invoices."
      )
    }

    if (request.signal.productivity < 50.0) {
      buffer += RiskOptimizationRecommendation(
        id = "risk-productivity-001",
        priority = "medium",
        message = "Productivity signal is low; rebalance sprint workload and reduce WIP."
      )
    }

    if (request.signal.collaboration < 50.0) {
      buffer += RiskOptimizationRecommendation(
        id = "risk-collaboration-001",
        priority = "medium",
        message = "Collaboration signal is soft; schedule cross-team reviews and unblock approvals."
      )
    }

    if (health.status == "red") {
      buffer += RiskOptimizationRecommendation(
        id = "risk-health-001",
        priority = "high",
        message = "Portfolio health is red; run stabilization playbook before new commitments."
      )
    }

    val recs = buffer.toList.sortBy(rec => priorityRank(rec.priority))

    RiskOptimizationPlan(
      profileId = request.profileId,
      signal = request.signal,
      health = health,
      riskScore = riskScore,
      recommendations = recs,
      generatedAtMs = System.currentTimeMillis()
    )
  }

  def manage(profileId: String, signal: PortfolioSignal): RiskOptimizationPlan =
    optimize(RiskOptimizationRequest(profileId = profileId, signal = signal))

  private def clamp(value: Double): Double = math.max(0.0, math.min(100.0, value))

  private def priorityRank(priority: String): Int = priority match {
    case "high" | "red"     => 0
    case "medium" | "amber" => 1
    case _                  => 2
  }
}

object RiskEngine {
  private val defaultEngine = new RiskEngine()

  def score(signal: PortfolioSignal): PortfolioHealthScore =
    defaultEngine.score(signal)
}
