package kogi.engine

import scala.collection.mutable

// GameEngine - mechanism orchestration for listings, bids, allocation, and incentives.
// This is a lightweight in-memory implementation intended for experimentation and tests.

sealed trait GameMechanism
case object Assignment extends GameMechanism
case object FirstPriceAuction extends GameMechanism
case object SecondPriceAuction extends GameMechanism
case object ReverseAuction extends GameMechanism

object GameMechanism {
  def fromString(raw: String): GameMechanism = raw.trim.toLowerCase match {
    case "assignment"     => Assignment
    case "first-price"    => FirstPriceAuction
    case "second-price"   => SecondPriceAuction
    case "reverse"        => ReverseAuction
    case "reverse-auction" => ReverseAuction
    case _                => Assignment
  }
}

sealed trait GameListingStatus
case object ListingOpen extends GameListingStatus
case object ListingClosed extends GameListingStatus
case object ListingFilled extends GameListingStatus
case object ListingSuspended extends GameListingStatus

sealed trait BidStatus
case object BidActive extends BidStatus
case object BidWithdrawn extends BidStatus
case object BidAccepted extends BidStatus
case object BidRejected extends BidStatus
case object BidExpired extends BidStatus

sealed trait GameAllocationStatus
case object AllocationSuccess extends GameAllocationStatus
case object AllocationNoBids extends GameAllocationStatus
case object AllocationNoEligibleBids extends GameAllocationStatus
case object AllocationClosed extends GameAllocationStatus
case object AllocationNotFound extends GameAllocationStatus
case object AllocationReserveNotMet extends GameAllocationStatus

final case class GameListing(
    id: String,
    ownerId: String,
    component: ComponentSubject,
    listingType: String = "listing",
    mechanism: GameMechanism = Assignment,
    askingPrice: Double = 0.0,
    reservePrice: Option[Double] = None,
    quantity: Int = 1,
    minQualityScore: Double = 0.0,
    status: GameListingStatus = ListingOpen,
    createdAtMs: Long = System.currentTimeMillis()
)

final case class GameBid(
    id: String,
    listingId: String,
    bidderId: String,
    amount: Double,
    qualityScore: Double = 50.0,
    bidType: String = "standard",
    submittedAtMs: Long = System.currentTimeMillis(),
    status: BidStatus = BidActive,
    metadata: Map[String, String] = Map.empty
)

final case class GameParticipantState(
    subject: UserSubject,
    reputation: Double = 50.0,
    kp: Double = 0.0,
    wins: Int = 0,
    losses: Int = 0,
    updatedAtMs: Long = System.currentTimeMillis()
)

final case class GameScoreWeights(
    matchWeight: Double = 0.55,
    priceWeight: Double = 0.25,
    qualityWeight: Double = 0.15,
    reputationWeight: Double = 0.05
) {
  def normalized: GameScoreWeights = {
    val total = matchWeight + priceWeight + qualityWeight + reputationWeight
    if (total <= 0.0) this
    else copy(
      matchWeight = matchWeight / total,
      priceWeight = priceWeight / total,
      qualityWeight = qualityWeight / total,
      reputationWeight = reputationWeight / total
    )
  }
}

final case class ScoredBid(
    bid: GameBid,
    matchScore: Double,
    priceScore: Double,
    qualityScore: Double,
    reputationScore: Double,
    compositeScore: Double,
    reasons: List[String]
)

final case class GameAllocation(
    listingId: String,
    winnerId: String,
    clearingPrice: Double,
    mechanism: GameMechanism,
    compositeScore: Double,
    matchScore: Double,
    priceScore: Double,
    qualityScore: Double,
    reputationScore: Double,
    reasons: List[String],
    createdAtMs: Long
)

final case class GameIncentive(
    participantId: String,
    kpDelta: Double,
    reputationDelta: Double,
    reason: String,
    timestampMs: Long
)

final case class GameIncentivePolicy(
    winnerBaseKp: Double = 40.0,
    ownerBaseKp: Double = 10.0,
    winnerReputationDelta: Double = 1.5,
    ownerReputationDelta: Double = 0.5,
    qualityKpScale: Double = 35.0,
    matchKpScale: Double = 45.0
)

object GameIncentivePolicy {
  val default: GameIncentivePolicy = GameIncentivePolicy()
}

final case class GameAllocationResult(
    listingId: String,
    status: GameAllocationStatus,
    allocation: Option[GameAllocation],
    scoredBids: List[ScoredBid],
    incentives: List[GameIncentive],
    generatedAtMs: Long
)

final case class GameSnapshot(
    participants: Int,
    listings: Int,
    openListings: Int,
    bids: Int,
    activeBids: Int,
    generatedAtMs: Long
)

final case class GameEvent(
    eventType: String,
    refId: String,
    data: Map[String, String],
    timestampMs: Long
)

final class GameEngine(
    matchEngine: MatchEngine = new MatchEngine(),
    defaultMatchWeights: MatchWeights = MatchWeights.default,
    allocationEngineOverride: AllocationEngine = null,
    incentiveEngineOverride: IncentiveEngine = null
) {
  val allocationEngine: AllocationEngine =
    if (allocationEngineOverride != null) allocationEngineOverride
    else new AllocationEngine(matchEngine = matchEngine, defaultMatchWeights = defaultMatchWeights)

  val incentiveEngine: IncentiveEngine =
    if (incentiveEngineOverride != null) incentiveEngineOverride
    else new IncentiveEngine()

  private val participants = mutable.Map.empty[String, GameParticipantState]
  private val listings = mutable.Map.empty[String, GameListing]
  private var bids: Vector[GameBid] = Vector.empty
  private var events: Vector[GameEvent] = Vector.empty

  // ------------------------------------------------------------------
  // Participants
  // ------------------------------------------------------------------

  def registerParticipant(
      subject: UserSubject,
      reputation: Double = 50.0,
      kp: Double = 0.0
  ): Unit = {
    val normalizedRep = clampScore(reputation)
    val state = GameParticipantState(
      subject = subject,
      reputation = normalizedRep,
      kp = kp,
      updatedAtMs = System.currentTimeMillis()
    )
    participants.update(subject.id, state)
    incentiveEngine.registerParticipant(subject.id, reputation = normalizedRep, kp = kp)
  }

  def registerParticipants(subjects: Seq[UserSubject]): Unit =
    subjects.foreach(registerParticipant(_))

  def participant(id: String): Option[GameParticipantState] =
    participants.get(id)

  def participantsSnapshot: List[GameParticipantState] =
    participants.values.toList

  // ------------------------------------------------------------------
  // Listings
  // ------------------------------------------------------------------

  def upsertListing(listing: GameListing): Unit = {
    listings.update(listing.id, listing)
    recordEvent("listing.upserted", listing.id, Map("owner_id" -> listing.ownerId))
  }

  def listing(id: String): Option[GameListing] =
    listings.get(id)

  def closeListing(id: String): Option[GameListing] =
    updateListingStatus(id, ListingClosed)

  def suspendListing(id: String): Option[GameListing] =
    updateListingStatus(id, ListingSuspended)

  def listingsByStatus(status: Option[GameListingStatus] = None): List[GameListing] = {
    val all = listings.values.toList
    status match {
      case Some(s) => all.filter(_.status == s)
      case None    => all
    }
  }

  // ------------------------------------------------------------------
  // Bids
  // ------------------------------------------------------------------

  def submitBid(bid: GameBid): GameBid = {
    val updated =
      listings.get(bid.listingId) match {
        case Some(listing) if listing.status == ListingOpen =>
          bid.copy(status = BidActive)
        case Some(_) =>
          bid.copy(status = BidRejected)
        case None =>
          bid.copy(status = BidRejected)
      }

    bids = bids.filterNot(_.id == bid.id) :+ updated

    val eventType = if (updated.status == BidActive) "bid.placed" else "bid.rejected"
    recordEvent(
      eventType,
      updated.id,
      Map("listing_id" -> updated.listingId, "bidder_id" -> updated.bidderId)
    )
    updated
  }

  def withdrawBid(id: String): Option[GameBid] = {
    val existing = bids.find(_.id == id)
    existing.map { bid =>
      val updated = bid.copy(status = BidWithdrawn)
      bids = bids.filterNot(_.id == id) :+ updated
      recordEvent("bid.withdrawn", updated.id, Map("listing_id" -> updated.listingId))
      updated
    }
  }

  def bidsForListing(listingId: String, includeInactive: Boolean = false): List[GameBid] =
    bids
      .filter(_.listingId == listingId)
      .filter(b => includeInactive || b.status == BidActive)
      .toList

  // ------------------------------------------------------------------
  // Matching and scoring
  // ------------------------------------------------------------------

  def matchListing(
      listingId: String,
      limit: Int = 10,
      matchWeights: MatchWeights = defaultMatchWeights
  ): Option[MatchResult[ComponentSubject, UserSubject]] =
    listings.get(listingId).map { listing =>
      val candidates = participants.values.map(_.subject).toSeq
      matchEngine.matchComponentToUsers(
        component = listing.component,
        role = AnyUserRole,
        candidates = candidates,
        weights = matchWeights,
        limit = limit.max(1)
      )
    }

  def scoreBids(
      listingId: String,
      scoreWeights: GameScoreWeights = GameScoreWeights(),
      matchWeights: MatchWeights = defaultMatchWeights
  ): List[ScoredBid] = {
    listings.get(listingId) match {
      case None => Nil
      case Some(listing) =>
        val request = buildAllocationRequest(
          listing = listing,
          listingBids = bidsForListing(listingId),
          scoreWeights = scoreWeights,
          matchWeights = matchWeights,
          limit = participants.size.max(1)
        )
        allocationEngine.score(request)
    }
  }

  // ------------------------------------------------------------------
  // Allocation and incentives  // ------------------------------------------------------------------
  // Allocation and incentives
  // ------------------------------------------------------------------

  def allocateListing(
      listingId: String,
      limit: Int = 1,
      scoreWeights: GameScoreWeights = GameScoreWeights(),
      matchWeights: MatchWeights = defaultMatchWeights,
      incentivePolicy: GameIncentivePolicy = GameIncentivePolicy.default,
      applyIncentives: Boolean = true
  ): GameAllocationResult = {
    val listingOpt = listings.get(listingId)
    if (listingOpt.isEmpty)
      return GameAllocationResult(listingId, AllocationNotFound, None, Nil, Nil, System.currentTimeMillis())

    val listing = listingOpt.get
    val request = buildAllocationRequest(
      listing = listing,
      listingBids = bidsForListing(listingId),
      scoreWeights = scoreWeights,
      matchWeights = matchWeights,
      limit = limit
    )
    val result = allocationEngine.allocate(request)

    result.status match {
      case AllocationSuccess =>
        updateListingStatus(listingId, ListingFilled)
        result.allocation.foreach { alloc =>
          recordEvent("allocation.created", listingId, Map("winner_id" -> alloc.winnerId))
        }
        val incentives = result.allocation match {
          case Some(allocation) if applyIncentives =>
            awardIncentives(listing, allocation, incentivePolicy)
          case _ => Nil
        }
        result.copy(incentives = incentives, generatedAtMs = System.currentTimeMillis())
      case _ => result
    }
  }

  def awardIncentives(
      listing: GameListing,
      allocation: GameAllocation,
      policy: GameIncentivePolicy = GameIncentivePolicy.default
  ): List[GameIncentive] = {
    val incentives = incentiveEngine.incentivesForAllocation(listing, allocation, policy)
    incentives.foreach { incentive =>
      val update = incentiveEngine.applyIncentive(
        incentive,
        referenceId = listing.id,
        referenceType = listing.listingType
      )
      syncParticipant(update.profile, incentive)
    }
    incentives
  }

  // ------------------------------------------------------------------
  // Observability
  // ------------------------------------------------------------------

  def snapshot(): GameSnapshot =
    GameSnapshot(
      participants = participants.size,
      listings = listings.size,
      openListings = listings.values.count(_.status == ListingOpen),
      bids = bids.size,
      activeBids = bids.count(_.status == BidActive),
      generatedAtMs = System.currentTimeMillis()
    )

  def eventLedger(limit: Int = 200): List[GameEvent] =
    events.takeRight(limit.max(0)).toList

  // ------------------------------------------------------------------
  // Internals
  // ------------------------------------------------------------------

  private def updateListingStatus(id: String, status: GameListingStatus): Option[GameListing] =
    listings.get(id).map { listing =>
      val updated = listing.copy(status = status)
      listings.update(id, updated)
      updated
    }

  private def recordEvent(eventType: String, refId: String, data: Map[String, String]): Unit = {
    events = events :+ GameEvent(
      eventType = eventType,
      refId = refId,
      data = data,
      timestampMs = System.currentTimeMillis()
    )
  }

  private def allocationParticipants: Seq[AllocationParticipant] =
    participants.values.toSeq.map(p => AllocationParticipant(p.subject, p.reputation))

  private def buildAllocationRequest(
      listing: GameListing,
      listingBids: Seq[GameBid],
      scoreWeights: GameScoreWeights,
      matchWeights: MatchWeights,
      limit: Int
  ): AllocationRequest =
    AllocationRequest(
      listing = listing,
      bids = listingBids,
      participants = allocationParticipants,
      scoreWeights = scoreWeights,
      matchWeights = matchWeights,
      limit = limit
    )

  private def syncParticipant(profile: IncentiveProfile, incentive: GameIncentive): Unit = {
    participants.get(profile.participantId).foreach { state =>
      val updated = state.copy(
        kp = profile.kpBalance,
        reputation = clampScore(profile.reputationScore),
        wins = state.wins + (if (incentive.reason.startsWith("winner")) 1 else 0),
        updatedAtMs = System.currentTimeMillis()
      )
      participants.update(state.subject.id, updated)
    }
  }

  private def clampScore(value: Double): Double =
    math.max(0.0, math.min(100.0, value))
}

