package kogi.analytics

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

object PortfolioHealthPipeline {
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
}