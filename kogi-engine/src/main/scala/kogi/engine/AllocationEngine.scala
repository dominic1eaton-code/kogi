package kogi.engine

import scala.collection.mutable

final case class AllocationParticipant(
    subject: UserSubject,
    reputation: Double = 50.0
)

final case class AllocationRequest(
    listing: GameListing,
    bids: Seq[GameBid],
    participants: Seq[AllocationParticipant],
    scoreWeights: GameScoreWeights = GameScoreWeights(),
    matchWeights: MatchWeights = MatchWeights.default,
    limit: Int = 1
)

final class AllocationEngine(
    matchEngine: MatchEngine = new MatchEngine(),
    defaultMatchWeights: MatchWeights = MatchWeights.default
) {

  def score(request: AllocationRequest): List[ScoredBid] = {
    val listing = request.listing
    val activeBids = request.bids.filter(_.status == BidActive)
    if (activeBids.isEmpty) return Nil

    val candidates = request.participants.map(_.subject)
    val weights =
      if (request.matchWeights == MatchWeights.default) defaultMatchWeights
      else request.matchWeights
    val matchResult = matchEngine.matchComponentToUsers(
      component = listing.component,
      role = AnyUserRole,
      candidates = candidates,
      weights = weights,
      limit = candidates.size.max(1)
    )

    val matchScores = matchResult.candidates.map(c => c.subject.id -> c.score).toMap
    val matchReasons = matchResult.candidates.map(c => c.subject.id -> c.reasons).toMap
    val reputationMap = request.participants.map(p => p.subject.id -> p.reputation).toMap

    val baseline = baselinePrice(listing, activeBids)
    val w = request.scoreWeights.normalized

    activeBids.flatMap { bid =>
      val reputation = reputationMap.getOrElse(bid.bidderId, 50.0)
      val repScore = clamp01(reputation / 100.0)
      val matchScore = matchScores.getOrElse(bid.bidderId, 0.0)
      val priceScore = priceScoreFor(bid.amount, baseline, listing.mechanism)
      val qualityScore = clamp01(bid.qualityScore / 100.0)

      if (bid.qualityScore < listing.minQualityScore) None
      else {
        val composite =
          (matchScore * w.matchWeight) +
          (priceScore * w.priceWeight) +
          (qualityScore * w.qualityWeight) +
          (repScore * w.reputationWeight)

        val reasons = mutable.ListBuffer[String]()
        matchReasons.get(bid.bidderId).foreach(reasons ++= _)
        reasons += f"price score ${(priceScore * 100.0).toInt}%%"
        reasons += f"quality score ${(qualityScore * 100.0).toInt}%%"
        reasons += f"reputation ${(repScore * 100.0).toInt}%%"

        Some(ScoredBid(
          bid = bid,
          matchScore = matchScore,
          priceScore = priceScore,
          qualityScore = qualityScore,
          reputationScore = repScore,
          compositeScore = clamp01(composite),
          reasons = reasons.toList
        ))
      }
    }.sortBy(sb => -sb.compositeScore).toList
  }

  def allocate(request: AllocationRequest): GameAllocationResult = {
    val listing = request.listing
    if (listing.status != ListingOpen)
      return GameAllocationResult(listing.id, AllocationClosed, None, Nil, Nil, System.currentTimeMillis())

    val activeBids = request.bids.filter(_.status == BidActive)
    if (activeBids.isEmpty)
      return GameAllocationResult(listing.id, AllocationNoBids, None, Nil, Nil, System.currentTimeMillis())

    val scored = score(request.copy(bids = activeBids))
    if (scored.isEmpty)
      return GameAllocationResult(listing.id, AllocationNoEligibleBids, None, Nil, Nil, System.currentTimeMillis())

    val winner = scored.head
    val second = scored.drop(1).headOption

    if (!reserveSatisfied(listing, winner.bid.amount))
      return GameAllocationResult(listing.id, AllocationReserveNotMet, None, scored, Nil, System.currentTimeMillis())

    val clearingPrice = listing.mechanism match {
      case SecondPriceAuction =>
        second.map(_.bid.amount).getOrElse(winner.bid.amount)
      case Assignment =>
        if (listing.askingPrice > 0.0) listing.askingPrice else winner.bid.amount
      case ReverseAuction =>
        winner.bid.amount
      case _ =>
        winner.bid.amount
    }

    val allocation = GameAllocation(
      listingId = listing.id,
      winnerId = winner.bid.bidderId,
      clearingPrice = clearingPrice,
      mechanism = listing.mechanism,
      compositeScore = winner.compositeScore,
      matchScore = winner.matchScore,
      priceScore = winner.priceScore,
      qualityScore = winner.qualityScore,
      reputationScore = winner.reputationScore,
      reasons = winner.reasons,
      createdAtMs = System.currentTimeMillis()
    )

    GameAllocationResult(
      listingId = listing.id,
      status = AllocationSuccess,
      allocation = Some(allocation),
      scoredBids = scored.take(request.limit.max(1)),
      incentives = Nil,
      generatedAtMs = System.currentTimeMillis()
    )
  }

  private def baselinePrice(listing: GameListing, activeBids: Seq[GameBid]): Double = {
    if (listing.askingPrice > 0.0) listing.askingPrice
    else {
      val sorted = activeBids.map(_.amount).sorted
      if (sorted.isEmpty) 1.0
      else if (sorted.size % 2 == 1) sorted(sorted.size / 2)
      else {
        val mid = sorted.size / 2
        (sorted(mid - 1) + sorted(mid)) / 2.0
      }
    }
  }

  private def priceScoreFor(amount: Double, baseline: Double, mechanism: GameMechanism): Double = {
    if (baseline <= 0.0) return 0.0
    mechanism match {
      case ReverseAuction =>
        clamp01(baseline / amount)
      case _ =>
        clamp01(amount / baseline)
    }
  }

  private def reserveSatisfied(listing: GameListing, amount: Double): Boolean =
    listing.reservePrice match {
      case None => true
      case Some(reserve) =>
        listing.mechanism match {
          case ReverseAuction => amount <= reserve
          case _             => amount >= reserve
        }
    }

  private def clamp01(value: Double): Double =
    math.max(0.0, math.min(1.0, value))
}


