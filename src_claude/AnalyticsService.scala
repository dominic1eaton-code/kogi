// analytics/src/main/scala/com/kogi/analytics/AnalyticsService.scala
package com.kogi.analytics

import com.kogi.analytics.models._
import java.time.Instant
import scala.util.{Try, Success, Failure}
import scala.concurrent.{Future, ExecutionContext}

/** Core analytics computation service. */
class AnalyticsService(implicit ec: ExecutionContext) {

  // ─── Portfolio Health ────────────────────────────────────────────────────────

  def computePortfolioHealth(items: List[PortfolioItem]): PortfolioHealth = {
    val portfolioId = items.headOption.map(_.portfolioId).getOrElse("unknown")
    val total = items.size
    val active = items.count(_.status == "active")
    val completed = items.count(_.status == "completed")
    val archived = items.count(_.status == "archived")

    // Health score: reward active/completed, penalize blocked/suspended
    val score = if (total == 0) 1.0
    else {
      val positiveRatio = (active.toDouble + completed.toDouble) / total.toDouble
      val completionBonus = if (total > 0) completed.toDouble / total.toDouble * 0.2 else 0.0
      math.min(1.0, positiveRatio + completionBonus)
    }

    PortfolioHealth(
      portfolioId = portfolioId,
      totalItems = total,
      activeItems = active,
      completedItems = completed,
      healthScore = BigDecimal(score).setScale(3, BigDecimal.RoundingMode.HALF_UP).toDouble,
      computedAt = Instant.now()
    )
  }

  // ─── Story Velocity ──────────────────────────────────────────────────────────

  def computeVelocity(
    stories: List[Story],
    wbsId: String,
    sprintNumber: Int,
    plannedPoints: Int,
    historicalVelocities: List[Double]
  ): VelocityMetrics = {
    val completedStories = stories.filter(_.status == "done")
    val completedPoints = completedStories.flatMap(_.points).sum
    val velocity = completedPoints.toDouble

    val avgVelocity = if (historicalVelocities.nonEmpty)
      (historicalVelocities :+ velocity).sum / (historicalVelocities.size + 1)
    else velocity

    VelocityMetrics(
      wbsId = wbsId,
      sprintNumber = sprintNumber,
      plannedPoints = plannedPoints,
      completedPoints = completedPoints,
      velocity = velocity,
      averageVelocity = BigDecimal(avgVelocity).setScale(2, BigDecimal.RoundingMode.HALF_UP).toDouble,
      period = s"Sprint $sprintNumber"
    )
  }

  // ─── Story Distribution ──────────────────────────────────────────────────────

  def computeStoryDistribution(stories: List[Story], wbsId: String): StoryDistribution = {
    val byType     = stories.groupBy(_.storyType).view.mapValues(_.size).toMap
    val byStatus   = stories.groupBy(_.status).view.mapValues(_.size).toMap
    val byPriority = stories.groupBy(_.priority).view.mapValues(_.size).toMap

    StoryDistribution(
      wbsId = wbsId,
      byType = byType,
      byStatus = byStatus,
      byPriority = byPriority,
      totalStories = stories.size
    )
  }

  // ─── Project Risk ─────────────────────────────────────────────────────────────

  def assessProjectRisk(project: Project, stories: List[Story]): ProjectRiskReport = {
    val factors = scala.collection.mutable.ListBuffer[RiskFactor]()
    var riskScore = 0.0

    // Factor 1: Blocker stories
    val blockers = stories.count(_.storyType == "blocker")
    if (blockers > 0) {
      riskScore += blockers * 15.0
      factors += RiskFactor(
        name = "Blocker Stories",
        severity = if (blockers > 3) "high" else "medium",
        description = s"$blockers unresolved blocker story(ies) detected"
      )
    }

    // Factor 2: Overdue (end date in past, status not done)
    project.endDate.foreach { end =>
      if (end.isBefore(Instant.now()) && project.status != "completed") {
        riskScore += 25.0
        factors += RiskFactor(
          name = "Schedule Overrun",
          severity = "high",
          description = "Project end date has passed and status is not completed"
        )
      }
    }

    // Factor 3: High backlog ratio
    val backlogRatio = stories.count(_.status == "backlog").toDouble / math.max(1, stories.size)
    if (backlogRatio > 0.6) {
      riskScore += 20.0
      factors += RiskFactor(
        name = "High Backlog Ratio",
        severity = "medium",
        description = f"${backlogRatio * 100}%.0f%% of stories are still in backlog"
      )
    }

    // Factor 4: No stories at all
    if (stories.isEmpty) {
      riskScore += 10.0
      factors += RiskFactor("No Stories", "low", "No stories have been created for this project")
    }

    val riskLevel = riskScore match {
      case s if s >= 50 => "critical"
      case s if s >= 30 => "high"
      case s if s >= 15 => "medium"
      case _            => "low"
    }

    val recommendations = factors.map(f => s"Address: ${f.description}").toList

    ProjectRiskReport(
      projectId = project.id,
      riskLevel = riskLevel,
      riskScore = math.min(100.0, riskScore),
      factors = factors.toList,
      recommendations = recommendations,
      generatedAt = Instant.now()
    )
  }

  // ─── Platform Metrics ─────────────────────────────────────────────────────────

  def computePlatformMetrics(
    activeUsers: Long,
    portfolios: Long,
    projects: Long,
    stories: Long,
    storiesCompletedToday: Long,
    eventsToday: Long
  ): PlatformMetrics = PlatformMetrics(
    activeUsers = activeUsers,
    totalPortfolios = portfolios,
    totalProjects = projects,
    totalStories = stories,
    storiesCompletedToday = storiesCompletedToday,
    eventsProcessedToday = eventsToday,
    timestamp = Instant.now()
  )
}
