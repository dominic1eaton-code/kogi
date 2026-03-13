package kogi.engine

import scala.collection.mutable

final case class IncentiveProfile(
    participantId: String,
    kpBalance: Double,
    reputationScore: Double,
    tier: String,
    streakWeeks: Int,
    streakMultiplier: Double,
    lastActivityMs: Long,
    updatedAtMs: Long
)

final case class IncentiveEvent(
    participantId: String,
    eventType: String,
    kpDelta: Double,
    reputationDelta: Double = 0.0,
    referenceId: String = "",
    referenceType: String = "",
    timestampMs: Long = System.currentTimeMillis()
)

final case class KpLedgerEntry(
    id: String,
    participantId: String,
    eventType: String,
    kpDelta: Double,
    reputationDelta: Double,
    balanceAfter: Double,
    reputationAfter: Double,
    streakWeeks: Int,
    streakMultiplier: Double,
    referenceId: String,
    referenceType: String,
    timestampMs: Long
)

final case class IncentiveUpdate(
    profile: IncentiveProfile,
    entry: KpLedgerEntry
)

final class IncentiveEngine {
  private val profiles = mutable.Map.empty[String, IncentiveProfile]
  private var ledger: Vector[KpLedgerEntry] = Vector.empty

  private val weekMs: Long = 7L * 24L * 60L * 60L * 1000L

  def registerParticipant(
      participantId: String,
      reputation: Double = 50.0,
      kp: Double = 0.0
  ): IncentiveProfile = {
    profiles.getOrElseUpdate(
      participantId,
      IncentiveProfile(
        participantId = participantId,
        kpBalance = math.max(0.0, kp),
        reputationScore = clampScore(reputation),
        tier = tierFor(clampScore(reputation)),
        streakWeeks = 0,
        streakMultiplier = 1.0,
        lastActivityMs = 0L,
        updatedAtMs = System.currentTimeMillis()
      )
    )
  }

  def profile(participantId: String): Option[IncentiveProfile] =
    profiles.get(participantId)

  def balance(participantId: String): Double =
    profiles.get(participantId).map(_.kpBalance).getOrElse(0.0)

  def applyEvent(event: IncentiveEvent, applyStreakMultiplier: Boolean = false): IncentiveUpdate = {
    val existing = profiles.getOrElse(event.participantId, registerParticipant(event.participantId))

    val (streakWeeks, lastActivity) = updateStreak(existing, event)
    val streakMultiplier = streakMultiplierFor(streakWeeks)

    val kpDelta =
      if (applyStreakMultiplier && event.kpDelta > 0.0) event.kpDelta * streakMultiplier
      else event.kpDelta

    val newBalance = math.max(0.0, existing.kpBalance + kpDelta)
    val newReputation = clampScore(existing.reputationScore + event.reputationDelta)

    val updatedProfile = IncentiveProfile(
      participantId = existing.participantId,
      kpBalance = newBalance,
      reputationScore = newReputation,
      tier = tierFor(newReputation),
      streakWeeks = streakWeeks,
      streakMultiplier = streakMultiplier,
      lastActivityMs = lastActivity,
      updatedAtMs = event.timestampMs
    )

    val entry = KpLedgerEntry(
      id = s"kp-${event.participantId}-${event.timestampMs}",
      participantId = event.participantId,
      eventType = event.eventType,
      kpDelta = kpDelta,
      reputationDelta = event.reputationDelta,
      balanceAfter = newBalance,
      reputationAfter = newReputation,
      streakWeeks = streakWeeks,
      streakMultiplier = streakMultiplier,
      referenceId = event.referenceId,
      referenceType = event.referenceType,
      timestampMs = event.timestampMs
    )

    profiles.update(event.participantId, updatedProfile)
    ledger = ledger :+ entry

    IncentiveUpdate(updatedProfile, entry)
  }

  def applyIncentive(
      incentive: GameIncentive,
      referenceId: String = "",
      referenceType: String = "",
      applyStreakMultiplier: Boolean = false
  ): IncentiveUpdate = {
    applyEvent(
      IncentiveEvent(
        participantId = incentive.participantId,
        eventType = s"incentive:${incentive.reason}",
        kpDelta = incentive.kpDelta,
        reputationDelta = incentive.reputationDelta,
        referenceId = referenceId,
        referenceType = referenceType,
        timestampMs = incentive.timestampMs
      ),
      applyStreakMultiplier = applyStreakMultiplier
    )
  }

  def applyIncentives(
      incentives: Seq[GameIncentive],
      referenceId: String = "",
      referenceType: String = "",
      applyStreakMultiplier: Boolean = false
  ): List[IncentiveUpdate] =
    incentives.toList.map(i => applyIncentive(i, referenceId, referenceType, applyStreakMultiplier))

  def earn(
      participantId: String,
      kp: Double,
      reason: String,
      referenceId: String = "",
      referenceType: String = ""
  ): IncentiveUpdate =
    applyEvent(
      IncentiveEvent(
        participantId = participantId,
        eventType = s"earn:${reason}",
        kpDelta = kp,
        referenceId = referenceId,
        referenceType = referenceType
      )
    )

  def penalize(
      participantId: String,
      kpPenalty: Double,
      reputationPenalty: Double,
      reason: String,
      referenceId: String = "",
      referenceType: String = ""
  ): IncentiveUpdate =
    applyEvent(
      IncentiveEvent(
        participantId = participantId,
        eventType = s"penalty:${reason}",
        kpDelta = -math.abs(kpPenalty),
        reputationDelta = -math.abs(reputationPenalty),
        referenceId = referenceId,
        referenceType = referenceType
      )
    )

  def redeem(
      participantId: String,
      cost: Double,
      reason: String,
      referenceId: String = "",
      referenceType: String = ""
  ): Either[String, IncentiveUpdate] = {
    val current = profiles.getOrElse(participantId, registerParticipant(participantId))
    if (current.kpBalance < cost) Left("insufficient_kp")
    else {
      val update = applyEvent(
        IncentiveEvent(
          participantId = participantId,
          eventType = s"redeem:${reason}",
          kpDelta = -math.abs(cost),
          referenceId = referenceId,
          referenceType = referenceType
        )
      )
      Right(update)
    }
  }

  def incentivesForAllocation(
      listing: GameListing,
      allocation: GameAllocation,
      policy: GameIncentivePolicy = GameIncentivePolicy.default
  ): List[GameIncentive] = {
    val winnerKp =
      policy.winnerBaseKp +
      allocation.matchScore * policy.matchKpScale +
      allocation.qualityScore * policy.qualityKpScale

    val ownerKp =
      policy.ownerBaseKp +
      allocation.matchScore * (policy.matchKpScale * 0.25)

    val now = System.currentTimeMillis()

    List(
      GameIncentive(
        participantId = allocation.winnerId,
        kpDelta = winnerKp,
        reputationDelta = policy.winnerReputationDelta,
        reason = s"winner:${listing.listingType}",
        timestampMs = now
      ),
      GameIncentive(
        participantId = listing.ownerId,
        kpDelta = ownerKp,
        reputationDelta = policy.ownerReputationDelta,
        reason = s"owner:${listing.listingType}",
        timestampMs = now
      )
    )
  }

  def ledgerEntries(limit: Int = 200): List[KpLedgerEntry] =
    ledger.takeRight(limit.max(0)).toList

  private def updateStreak(profile: IncentiveProfile, event: IncentiveEvent): (Int, Long) = {
    val qualifies = event.kpDelta > 0.0 && !event.eventType.startsWith("redeem")
    if (!qualifies) (profile.streakWeeks, profile.lastActivityMs)
    else if (profile.lastActivityMs == 0L) (1, event.timestampMs)
    else {
      val diff = event.timestampMs - profile.lastActivityMs
      if (diff >= weekMs * 2) (1, event.timestampMs)
      else if (diff >= weekMs) (profile.streakWeeks + 1, event.timestampMs)
      else (profile.streakWeeks, profile.lastActivityMs)
    }
  }

  private def streakMultiplierFor(weeks: Int): Double = {
    val multiplier = 1.0 + math.max(0, weeks - 1) * 0.05
    math.min(2.0, multiplier)
  }

  private def tierFor(score: Double): String =
    if (score >= 90.0) "elite"
    else if (score >= 75.0) "trusted"
    else if (score >= 55.0) "established"
    else if (score >= 35.0) "developing"
    else "new"

  private def clampScore(value: Double): Double =
    math.max(0.0, math.min(100.0, value))
}
