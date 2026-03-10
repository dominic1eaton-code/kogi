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
    val productivityW   = 0.30
    val cashFlowW       = 0.30
    val collaborationW  = 0.20
    val riskW           = 0.20

    val weighted =
      signal.productivity  * productivityW +
      signal.cashFlow      * cashFlowW +
      signal.collaboration * collaborationW +
      (100.0 - signal.risk) * riskW

    val status =
      if (weighted >= 80) "green"
      else if (weighted >= 60) "amber"
      else "red"

    val recommendations = List(
      if (signal.risk > 65)        Some("reduce risk exposure in exchange and campaign modules") else None,
      if (signal.cashFlow < 50)    Some("prioritize paid backlog stories and wallet recovery") else None,
      if (signal.productivity < 50) Some("rebalance sprint workload across teams") else None
    ).flatten

    PortfolioHealthScore(weighted, status, recommendations)
  }

  /**
   * Persona-aware scoring: adjust signal weights based on the user's persona.
   * e.g. a PowerUser profile gets tighter risk thresholds; a CasualBrowser
   * gets wider tolerance to avoid alert fatigue.
   */
  def scoreWithPersona(
      signal: PortfolioSignal,
      persona: PersonaLabel
  ): PortfolioHealthScore = {
    val adjusted = persona match {
      case PowerUser =>
        // Power users can tolerate higher WIP but need tighter risk guardrails
        signal.copy(risk = signal.risk * 1.10)
      case CasualBrowser =>
        // Reduce noise for low-engagement profiles
        signal.copy(risk = signal.risk * 0.85, productivity = signal.productivity * 0.90)
      case Explorer =>
        // Explorers may naturally show lower collaboration; compensate
        signal.copy(collaboration = math.min(100.0, signal.collaboration * 1.15))
      case ValueSeeker =>
        // Value seekers are cashflow-sensitive; amplify that signal
        signal.copy(cashFlow = signal.cashFlow * 0.90)
      case _ => signal
    }
    score(adjusted)
  }

  def optimize(request: RiskOptimizationRequest): RiskOptimizationPlan = {
    val health    = request.context.get("persona")
      .flatMap(p => PersonaLabel.fromString(p).map(scoreWithPersona(request.signal, _)))
      .getOrElse(score(request.signal))
    val riskScore = clamp(100.0 - request.signal.risk)
    val buffer    = scala.collection.mutable.ListBuffer.empty[RiskOptimizationRecommendation]

    if (request.signal.risk >= 70.0)
      buffer += RiskOptimizationRecommendation("risk-hot-001", "high",
        "High risk exposure detected; stabilize exchange, wallet, and compliance workflows.")
    if (request.signal.cashFlow < 50.0)
      buffer += RiskOptimizationRecommendation("risk-cashflow-001", "high",
        "Cashflow signal is weak; tighten receivables and review overdue invoices.")
    if (request.signal.productivity < 50.0)
      buffer += RiskOptimizationRecommendation("risk-productivity-001", "medium",
        "Productivity signal is low; rebalance sprint workload and reduce WIP.")
    if (request.signal.collaboration < 50.0)
      buffer += RiskOptimizationRecommendation("risk-collaboration-001", "medium",
        "Collaboration signal is soft; schedule cross-team reviews and unblock approvals.")
    if (health.status == "red")
      buffer += RiskOptimizationRecommendation("risk-health-001", "high",
        "Portfolio health is red; run stabilization playbook before new commitments.")

    RiskOptimizationPlan(
      profileId = request.profileId,
      signal = request.signal,
      health = health,
      riskScore = riskScore,
      recommendations = buffer.toList.sortBy(r => priorityRank(r.priority)),
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
  def score(signal: PortfolioSignal): PortfolioHealthScore = defaultEngine.score(signal)
}
