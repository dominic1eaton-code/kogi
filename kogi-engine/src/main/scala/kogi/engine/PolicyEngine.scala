package kogi.engine

import scala.collection.mutable

final case class PolicyContext(
    componentId: String,
    eventKind: String,
    actorId: String,
    componentType: String = "component",
    properties: Map[String, String] = Map.empty,
    tags: Set[String] = Set.empty,
    timestampMs: Long = System.currentTimeMillis()
)

sealed trait PolicyDecision {
  def kind: String
  def reason: Option[String]
  def isAllow: Boolean = this == PolicyDecision.Allow
}

object PolicyDecision {
  case object Allow extends PolicyDecision {
    val kind: String = "allow"
    val reason: Option[String] = None
  }

  final case class Deny(reasonText: String) extends PolicyDecision {
    val kind: String = "deny"
    val reason: Option[String] = Some(reasonText)
  }

  final case class RequireApproval(reasonText: String) extends PolicyDecision {
    val kind: String = "require_approval"
    val reason: Option[String] = Some(reasonText)
  }
}

trait PolicyRule {
  def id: String
  def name: String
  def evaluate(ctx: PolicyContext): PolicyDecision
}

final case class SimplePolicyRule(
    id: String,
    name: String,
    decision: PolicyDecision,
    when: PolicyContext => Boolean,
    description: String = ""
) extends PolicyRule {
  override def evaluate(ctx: PolicyContext): PolicyDecision =
    if (when(ctx)) decision else PolicyDecision.Allow
}

final case class PolicyEvaluation(
    engineId: String,
    engineName: String,
    decision: PolicyDecision,
    evaluatedAtMs: Long
)

final case class PolicyEvaluationReport(
    decision: PolicyDecision,
    evaluations: Seq[PolicyEvaluation],
    evaluatedAtMs: Long
)

sealed trait ApprovalStatus { def value: String }

object ApprovalStatus {
  case object Pending extends ApprovalStatus { val value: String = "pending" }
  case object Approved extends ApprovalStatus { val value: String = "approved" }
  case object Rejected extends ApprovalStatus { val value: String = "rejected" }
  case object Expired extends ApprovalStatus { val value: String = "expired" }

  def fromString(raw: String): ApprovalStatus = raw.trim.toLowerCase match {
    case "approved" => Approved
    case "rejected" => Rejected
    case "expired"  => Expired
    case _           => Pending
  }
}

final case class ApprovalRequest(
    id: String,
    componentId: String,
    requestedBy: String,
    reason: String,
    status: ApprovalStatus,
    createdAtMs: Long,
    resolvedAtMs: Option[Long] = None,
    resolver: Option[String] = None,
    notes: Option[String] = None,
    metadata: Map[String, String] = Map.empty
)

final class PolicyEngine {
  private val rules = mutable.ArrayBuffer.empty[PolicyRule]
  private val approvals = mutable.Map.empty[String, ApprovalRequest]

  def register(rule: PolicyRule): Unit = {
    val idx = rules.indexWhere(_.id == rule.id)
    if (idx >= 0) rules.update(idx, rule) else rules += rule
  }

  def registerAll(values: Seq[PolicyRule]): Unit =
    values.foreach(register)

  def unregister(id: String): Option[PolicyRule] = {
    val idx = rules.indexWhere(_.id == id)
    if (idx >= 0) Some(rules.remove(idx)) else None
  }

  def listRules(): Seq[PolicyRule] =
    rules.toVector

  def clearRules(): Unit =
    rules.clear()

  def evaluate(ctx: PolicyContext): PolicyDecision = {
    for (rule <- rules) {
      val decision = rule.evaluate(ctx)
      if (!decision.isAllow) return decision
    }
    PolicyDecision.Allow
  }

  def evaluateReport(ctx: PolicyContext): PolicyEvaluationReport = {
    val evaluations = mutable.ArrayBuffer.empty[PolicyEvaluation]
    var finalDecision: PolicyDecision = PolicyDecision.Allow
    val evaluatedAt = System.currentTimeMillis()

    for (rule <- rules) {
      val decision = rule.evaluate(ctx)
      evaluations += PolicyEvaluation(
        engineId = rule.id,
        engineName = rule.name,
        decision = decision,
        evaluatedAtMs = evaluatedAt
      )
      if (!decision.isAllow) {
        finalDecision = decision
        return PolicyEvaluationReport(
          decision = finalDecision,
          evaluations = evaluations.toVector,
          evaluatedAtMs = evaluatedAt
        )
      }
    }

    PolicyEvaluationReport(
      decision = finalDecision,
      evaluations = evaluations.toVector,
      evaluatedAtMs = evaluatedAt
    )
  }

  def requestApproval(
      componentId: String,
      requestedBy: String,
      reason: String,
      metadata: Map[String, String] = Map.empty
  ): ApprovalRequest = {
    val now = System.currentTimeMillis()
    val id = s"apr-${safeId(componentId)}-$now"
    val request = ApprovalRequest(
      id = id,
      componentId = componentId,
      requestedBy = requestedBy,
      reason = reason,
      status = ApprovalStatus.Pending,
      createdAtMs = now,
      metadata = metadata
    )
    approvals.update(id, request)
    request
  }

  def resolveApproval(
      requestId: String,
      approved: Boolean,
      resolver: String,
      notes: Option[String] = None
  ): Option[ApprovalRequest] = {
    approvals.get(requestId).map { existing =>
      val updated = existing.copy(
        status = if (approved) ApprovalStatus.Approved else ApprovalStatus.Rejected,
        resolvedAtMs = Some(System.currentTimeMillis()),
        resolver = Some(resolver),
        notes = notes
      )
      approvals.update(requestId, updated)
      updated
    }
  }

  def approval(requestId: String): Option[ApprovalRequest] =
    approvals.get(requestId)

  def approvalsByStatus(status: Option[ApprovalStatus] = None): Seq[ApprovalRequest] = {
    val all = approvals.values.toSeq.sortBy(_.createdAtMs)
    status.map(s => all.filter(_.status == s)).getOrElse(all)
  }

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")
}
