package kogi.engine

import scala.collection.mutable

// =============================================================
//  RecommendationEngine – Extended
//
//  Features:
//    1. User Profiling & Persona Building
//    2. Collaborative Filtering  (user–user similarity)
//    3. Content-Based Filtering  (item–item similarity)
//    4. Hybrid Models            (weighted blend)
//    5. Real-Time & Contextual   (location / time / device)
//    6. Cold-Start Handling      (new users and new items)
//    7. Feedback Loop            (implicit + explicit signals)
//    8. Implicit & Explicit Data Gathering
// =============================================================


// -------------------------------------------------------------
// Data Gathering – Interaction Events
// -------------------------------------------------------------

sealed trait InteractionType
case object Click    extends InteractionType  // implicit
case object View     extends InteractionType  // implicit
case object Dwell    extends InteractionType  // implicit – time-on-item
case object Search   extends InteractionType  // implicit
case object Rate     extends InteractionType  // explicit (0.0–5.0)
case object Review   extends InteractionType  // explicit
case object Purchase extends InteractionType  // explicit
case object Bookmark extends InteractionType  // explicit
case object Share    extends InteractionType  // explicit
case object Ignore   extends InteractionType  // implicit negative
case object Dismiss  extends InteractionType  // explicit negative

object InteractionType {
  def weight(t: InteractionType): Double = t match {
    case Purchase => 5.0
    case Rate     => 4.0
    case Review   => 4.0
    case Bookmark => 3.0
    case Share    => 3.0
    case Click    => 2.0
    case View     => 1.5
    case Dwell    => 1.0
    case Search   => 1.0
    case Ignore   => -1.0
    case Dismiss  => -2.0
  }

  def fromString(s: String): InteractionType = s.trim.toLowerCase match {
    case "click"    => Click
    case "view"     => View
    case "dwell"    => Dwell
    case "search"   => Search
    case "rate"     => Rate
    case "review"   => Review
    case "purchase" => Purchase
    case "bookmark" => Bookmark
    case "share"    => Share
    case "ignore"   => Ignore
    case "dismiss"  => Dismiss
    case _          => View
  }
}

final case class UserInteraction(
    id: String,
    userId: String,
    itemId: String,
    interactionType: InteractionType,
    value: Double = 1.0,
    sessionId: String = "",
    timestampMs: Long,
    context: InteractionContext = InteractionContext()
)

final case class InteractionContext(
    device: String = "unknown",
    location: String = "unknown",
    timeOfDay: String = "unknown",
    referrer: String = "",
    metadata: Map[String, String] = Map.empty
)

object InteractionContext {
  def fromMs(timestampMs: Long): InteractionContext = {
    val cal = java.util.Calendar.getInstance()
    cal.setTimeInMillis(timestampMs)
    val hour = cal.get(java.util.Calendar.HOUR_OF_DAY)
    val timeOfDay =
      if (hour >= 5  && hour < 12) "morning"
      else if (hour >= 12 && hour < 17) "afternoon"
      else if (hour >= 17 && hour < 21) "evening"
      else "night"
    InteractionContext(timeOfDay = timeOfDay)
  }
}


// -------------------------------------------------------------
// Item Profile – Content-Based Filtering substrate
// -------------------------------------------------------------

final case class ItemProfile(
    id: String,
    title: String,
    category: String,
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty,
    popularityScore: Double = 0.0,
    createdMs: Long = System.currentTimeMillis()
) {
  def featureVector: Map[String, Double] =
    tags.map(_ -> 1.0).toMap ++
    attributes.keys.map(_ -> 1.0).toMap +
    ("category:" + category -> 2.0)
}


// -------------------------------------------------------------
// User Profiling & Persona
// -------------------------------------------------------------

sealed trait PersonaLabel
case object Explorer      extends PersonaLabel
case object Specialist    extends PersonaLabel
case object CasualBrowser extends PersonaLabel
case object PowerUser     extends PersonaLabel
case object EarlyAdopter  extends PersonaLabel
case object ValueSeeker   extends PersonaLabel
case object Collaborator  extends PersonaLabel
case object Newcomer      extends PersonaLabel
case object UnknownPersona extends PersonaLabel

object PersonaLabel {
  def label(p: PersonaLabel): String = p match {
    case Explorer       => "explorer"
    case Specialist     => "specialist"
    case CasualBrowser  => "casual-browser"
    case PowerUser      => "power-user"
    case EarlyAdopter   => "early-adopter"
    case ValueSeeker    => "value-seeker"
    case Collaborator   => "collaborator"
    case Newcomer       => "newcomer"
    case UnknownPersona => "unknown"
  }
}

final case class PersonaBuildResult(
    userId: String,
    persona: PersonaLabel,
    traits: List[String],
    confidence: Double,
    dominantCategories: List[String],
    updatedMs: Long
)

final case class UserProfile(
    id: String,
    demographics: Map[String, String] = Map.empty,
    preferenceVector: Map[String, Double] = Map.empty,
    persona: PersonaLabel = Newcomer,
    personaTraits: List[String] = Nil,
    personaConfidence: Double = 0.0,
    totalInteractions: Int = 0,
    interactionBreakdown: Map[String, Int] = Map.empty,
    dominantCategories: List[String] = Nil,
    preferredDevices: List[String] = Nil,
    preferredTimeOfDay: List[String] = Nil,
    createdMs: Long = System.currentTimeMillis(),
    lastSeenMs: Long = System.currentTimeMillis()
)


// -------------------------------------------------------------
// Recommendation Results
// -------------------------------------------------------------

sealed trait FilteringMethod
case object CollaborativeFiltering extends FilteringMethod
case object ContentBasedFiltering  extends FilteringMethod
case object HybridFiltering        extends FilteringMethod
case object ColdStartFallback      extends FilteringMethod
case object ContextualBoost        extends FilteringMethod

final case class PersonalizedRecommendation(
    itemId: String,
    score: Double,
    method: FilteringMethod,
    reason: String,
    contextBoosted: Boolean = false
)

final case class PersonalizedRecommendationResult(
    userId: String,
    recommendations: List[PersonalizedRecommendation],
    persona: PersonaLabel,
    method: FilteringMethod,
    coldStart: Boolean,
    generatedAtMs: Long
)

final case class FeedbackRecord(
    userId: String,
    itemId: String,
    interactionType: InteractionType,
    scoreDelta: Double,
    timestampMs: Long
)


// =============================================================
//  RecommendationEngine
// =============================================================

final class RecommendationEngine(
    recommendationLimit: Int = 8,
    discoverLimit: Int = 8,
    exploreLimit: Int = 8,
    collaborativeWeight: Double = 0.55,
    contentWeight: Double = 0.45,
    coldStartMinInteractions: Int = 5,
    maxInteractions: Int = 200000,
    maxFeedback: Int = 100000,
    profileDecayFactor: Double = 0.05
) {

  private val profiles     = mutable.Map[String, UserProfile]()
  private val items        = mutable.Map[String, ItemProfile]()
  private var interactions = Vector[UserInteraction]()
  private var feedbackLog  = Vector[FeedbackRecord]()
  private var userSimDirty = true
  private var userSimCache = Map[(String, String), Double]()


  // ==================================================================
  // 1. Data Gathering
  // ==================================================================

  def recordInteraction(interaction: UserInteraction): UserProfile = {
    interactions = (interactions :+ interaction).takeRight(maxInteractions)
    userSimDirty = true

    val baseItem = items.getOrElse(interaction.itemId,
      ItemProfile(id = interaction.itemId, title = interaction.itemId, category = "unknown"))
    val w = InteractionType.weight(interaction.interactionType)
    items(interaction.itemId) = baseItem.copy(
      popularityScore = math.max(0.0, baseItem.popularityScore + w * 0.1)
    )

    applyFeedback(interaction)
  }

  def recordInteractions(batch: Seq[UserInteraction]): Unit = batch.foreach(recordInteraction)

  def recordRating(userId: String, itemId: String, rating: Double,
                   context: InteractionContext = InteractionContext()): UserProfile =
    recordInteraction(UserInteraction(
      id = s"rate-$userId-$itemId-${System.currentTimeMillis()}",
      userId = userId, itemId = itemId, interactionType = Rate,
      value = rating / 5.0, timestampMs = System.currentTimeMillis(), context = context
    ))

  def recordReview(userId: String, itemId: String, sentiment: Double,
                   context: InteractionContext = InteractionContext()): UserProfile =
    recordInteraction(UserInteraction(
      id = s"review-$userId-$itemId-${System.currentTimeMillis()}",
      userId = userId, itemId = itemId, interactionType = Review,
      value = sentiment, timestampMs = System.currentTimeMillis(), context = context
    ))

  def recordDwell(userId: String, itemId: String, dwellSeconds: Double,
                  context: InteractionContext = InteractionContext()): UserProfile =
    recordInteraction(UserInteraction(
      id = s"dwell-$userId-$itemId-${System.currentTimeMillis()}",
      userId = userId, itemId = itemId, interactionType = Dwell,
      value = math.min(dwellSeconds / 60.0, 1.0),
      timestampMs = System.currentTimeMillis(), context = context
    ))

  def recordSearch(userId: String, query: String,
                   context: InteractionContext = InteractionContext()): UserProfile =
    recordInteraction(UserInteraction(
      id = s"search-$userId-${System.currentTimeMillis()}",
      userId = userId, itemId = s"query:$query", interactionType = Search,
      timestampMs = System.currentTimeMillis(), context = context
    ))


  // ==================================================================
  // 2. Feedback Loop & Continuous Learning
  // ==================================================================

  private def applyFeedback(interaction: UserInteraction): UserProfile = {
    val userId  = interaction.userId
    val weight  = InteractionType.weight(interaction.interactionType) * interaction.value
    val item    = items.getOrElse(interaction.itemId,
      ItemProfile(id = interaction.itemId, title = interaction.itemId, category = "unknown"))
    val profile = profiles.getOrElse(userId, UserProfile(
      id = userId, createdMs = System.currentTimeMillis()
    ))

    val updatedPrefs = item.featureVector.foldLeft(profile.preferenceVector) { case (prefs, (feature, _)) =>
      val current = prefs.getOrElse(feature, 0.0)
      val updated = (1.0 - profileDecayFactor) * current + profileDecayFactor * weight
      prefs + (feature -> clamp(updated, -5.0, 5.0))
    }

    val typeName  = interaction.interactionType.toString
    val breakdown = profile.interactionBreakdown +
      (typeName -> (profile.interactionBreakdown.getOrElse(typeName, 0) + 1))

    val deviceList =
      if (interaction.context.device != "unknown" &&
          !profile.preferredDevices.contains(interaction.context.device))
        (profile.preferredDevices :+ interaction.context.device).takeRight(3)
      else profile.preferredDevices

    val timeList =
      if (interaction.context.timeOfDay != "unknown" &&
          !profile.preferredTimeOfDay.contains(interaction.context.timeOfDay))
        (profile.preferredTimeOfDay :+ interaction.context.timeOfDay).takeRight(4)
      else profile.preferredTimeOfDay

    val newProfile = profile.copy(
      preferenceVector     = updatedPrefs,
      totalInteractions    = profile.totalInteractions + 1,
      interactionBreakdown = breakdown,
      preferredDevices     = deviceList,
      preferredTimeOfDay   = timeList,
      lastSeenMs           = interaction.timestampMs
    )

    feedbackLog = (feedbackLog :+ FeedbackRecord(
      userId = userId, itemId = interaction.itemId,
      interactionType = interaction.interactionType,
      scoreDelta = weight, timestampMs = interaction.timestampMs
    )).takeRight(maxFeedback)

    val updatedWithPersona =
      if (newProfile.totalInteractions % 10 == 0) {
        val persona = buildPersona(userId, Some(newProfile))
        newProfile.copy(
          persona = persona.persona,
          personaTraits = persona.traits,
          personaConfidence = persona.confidence,
          dominantCategories = persona.dominantCategories
        )
      } else newProfile

    profiles(userId) = updatedWithPersona
    updatedWithPersona
  }


  // ==================================================================
  // 3. Persona Building
  // ==================================================================

  def buildPersona(userId: String,
                   baseProfile: Option[UserProfile] = None): PersonaBuildResult = {
    val profile = baseProfile.orElse(profiles.get(userId)).getOrElse(
      return PersonaBuildResult(userId, Newcomer, List("no interactions recorded"),
        0.0, Nil, System.currentTimeMillis())
    )

    val userInteractions = interactions.filter(_.userId == userId)
    val total = userInteractions.size

    if (total < coldStartMinInteractions) {
      return PersonaBuildResult(userId, Newcomer,
        List("insufficient interactions for persona detection"),
        confidence = total.toDouble / coldStartMinInteractions.toDouble,
        dominantCategories = Nil,
        updatedMs = System.currentTimeMillis()
      )
    }

    val byType        = userInteractions.groupBy(_.interactionType)
    val purchaseCount = byType.getOrElse(Purchase, Vector.empty).size
    val shareCount    = byType.getOrElse(Share, Vector.empty).size
    val reviewCount   = byType.getOrElse(Review, Vector.empty).size
    val clickCount    = byType.getOrElse(Click, Vector.empty).size
    val ignoreCount   = byType.getOrElse(Ignore, Vector.empty).size +
                        byType.getOrElse(Dismiss, Vector.empty).size

    val uniqueCategories = userInteractions
      .flatMap(i => items.get(i.itemId).map(_.category)).distinct.size
    val engagementRate   = ratio(clickCount + purchaseCount + reviewCount, total)
    val negativeRate     = ratio(ignoreCount, total)

    val weekMs       = 7L * 24L * 60L * 60L * 1000L
    val recentCount  = userInteractions.count(_.timestampMs >= System.currentTimeMillis() - weekMs)
    val frequencyScore = math.min(recentCount.toDouble / 20.0, 1.0)

    val traits = mutable.ListBuffer[String]()
    var persona: PersonaLabel = UnknownPersona
    var confidence = 0.5

    if (total < coldStartMinInteractions * 2) {
      persona = Newcomer; confidence = 0.60
      traits += "new user – limited history"
    } else if (uniqueCategories >= 5 && engagementRate > 0.4) {
      persona = Explorer; confidence = 0.75
      traits += s"explores $uniqueCategories+ categories"
      traits += "broad interaction diversity"
    } else if (uniqueCategories <= 2 && total >= 20) {
      persona = Specialist; confidence = 0.80
      traits += s"deep focus in $uniqueCategories area(s)"
    } else if (frequencyScore > 0.7 && engagementRate > 0.5) {
      persona = PowerUser; confidence = 0.85
      traits += "high frequency"
      traits += "strong engagement"
    } else if (shareCount + reviewCount > total * 0.2) {
      persona = Collaborator; confidence = 0.78
      traits += "frequent sharer and reviewer"
    } else if (frequencyScore < 0.2 && engagementRate < 0.25) {
      persona = CasualBrowser; confidence = 0.70
      traits += "low frequency"
      traits += "passive browsing pattern"
    } else if (negativeRate > 0.35) {
      persona = ValueSeeker; confidence = 0.65
      traits += "selective – high dismiss/ignore rate"
    } else {
      persona = UnknownPersona; confidence = 0.40
    }

    val catScores = userInteractions
      .flatMap(i => items.get(i.itemId).map(it =>
        it.category -> InteractionType.weight(i.interactionType)))
      .groupBy(_._1)
      .view.mapValues(_.map(_._2).sum).toMap
    val dominantCats = catScores.toSeq.sortBy(-_._2).take(3).map(_._1).toList

    PersonaBuildResult(userId, persona, traits.toList, confidence, dominantCats,
      System.currentTimeMillis())
  }

  def getProfile(userId: String): Option[UserProfile] = profiles.get(userId)
  def upsertProfile(profile: UserProfile): Unit = { profiles(profile.id) = profile; userSimDirty = true }
  def upsertItem(item: ItemProfile): Unit    = items(item.id) = item
  def upsertItems(batch: Seq[ItemProfile]): Unit = batch.foreach(upsertItem)


  // ==================================================================
  // 4. Collaborative Filtering
  // ==================================================================

  private def cosineSimilarity(a: Map[String, Double], b: Map[String, Double]): Double = {
    val shared = a.keySet intersect b.keySet
    if (shared.isEmpty) return 0.0
    val dot   = shared.map(k => a(k) * b(k)).sum
    val normA = math.sqrt(a.values.map(v => v * v).sum)
    val normB = math.sqrt(b.values.map(v => v * v).sum)
    if (normA == 0.0 || normB == 0.0) 0.0 else dot / (normA * normB)
  }

  private def rebuildUserSimCache(): Unit = {
    val allUsers = profiles.values.toSeq
    val cache    = mutable.Map[(String, String), Double]()
    for (i <- allUsers.indices; j <- i + 1 until allUsers.size) {
      val sim = cosineSimilarity(allUsers(i).preferenceVector, allUsers(j).preferenceVector)
      cache((allUsers(i).id, allUsers(j).id)) = sim
      cache((allUsers(j).id, allUsers(i).id)) = sim
    }
    userSimCache = cache.toMap
    userSimDirty = false
  }

  private def similarUsers(userId: String, topN: Int = 10): Seq[(String, Double)] = {
    if (userSimDirty) rebuildUserSimCache()
    userSimCache
      .collect { case ((uid, other), sim) if uid == userId && sim > 0.0 => other -> sim }
      .toSeq.sortBy(-_._2).take(topN)
  }

  def collaborativeRecommendations(
      userId: String,
      exclude: Set[String] = Set.empty,
      limit: Int = recommendationLimit
  ): List[PersonalizedRecommendation] = {
    val neighbours = similarUsers(userId)
    if (neighbours.isEmpty) return Nil

    val itemScores = mutable.Map[String, Double]()
    neighbours.foreach { case (neighbourId, sim) =>
      interactions
        .filter(i => i.userId == neighbourId && !exclude.contains(i.itemId))
        .foreach { i =>
          val w = InteractionType.weight(i.interactionType) * sim * i.value
          itemScores(i.itemId) = itemScores.getOrElse(i.itemId, 0.0) + w
        }
    }

    val userItems = interactions.filter(_.userId == userId).map(_.itemId).toSet
    itemScores
      .filter { case (id, _) => !userItems.contains(id) && !exclude.contains(id) }
      .toSeq.sortBy(-_._2).take(limit)
      .map { case (itemId, score) =>
        PersonalizedRecommendation(itemId, score, CollaborativeFiltering,
          "users similar to you engaged with this item")
      }.toList
  }


  // ==================================================================
  // 5. Content-Based Filtering
  // ==================================================================

  def contentBasedRecommendations(
      userId: String,
      exclude: Set[String] = Set.empty,
      limit: Int = recommendationLimit
  ): List[PersonalizedRecommendation] = {
    val profile = profiles.getOrElse(userId, return Nil)
    if (profile.preferenceVector.isEmpty) return Nil

    val userVec   = profile.preferenceVector
    val userItems = interactions.filter(_.userId == userId).map(_.itemId).toSet

    items.values
      .filterNot(item => userItems.contains(item.id) || exclude.contains(item.id))
      .map { item =>
        val sim = cosineSimilarity(userVec, item.featureVector)
        PersonalizedRecommendation(item.id, sim, ContentBasedFiltering,
          s"matches your preferences in ${item.category}")
      }
      .toSeq.sortBy(-_.score).take(limit).toList
  }


  // ==================================================================
  // 6. Hybrid Model
  // ==================================================================

  def hybridRecommendations(
      userId: String,
      context: InteractionContext = InteractionContext(),
      exclude: Set[String] = Set.empty,
      limit: Int = recommendationLimit
  ): PersonalizedRecommendationResult = {
    val profile    = profiles.getOrElse(userId, UserProfile(id = userId))
    val isColdStart = profile.totalInteractions < coldStartMinInteractions

    if (isColdStart) {
      val recs = coldStartRecommendations(userId, context, exclude, limit)
      return PersonalizedRecommendationResult(userId, recs, profile.persona,
        ColdStartFallback, coldStart = true, generatedAtMs = System.currentTimeMillis())
    }

    val collab  = collaborativeRecommendations(userId, exclude, limit * 2)
    val content = contentBasedRecommendations(userId, exclude, limit * 2)

    val merged = mutable.Map[String, Double]()
    collab.foreach  { r => merged(r.itemId) = merged.getOrElse(r.itemId, 0.0) + r.score * collaborativeWeight }
    content.foreach { r => merged(r.itemId) = merged.getOrElse(r.itemId, 0.0) + r.score * contentWeight }

    val boosted = applyContextualBoost(merged.toMap, context, profile)

    val recommendations = boosted.toSeq.sortBy(-_._2).take(limit).map { case (itemId, score) =>
      val wasContextBoosted = score > merged.getOrElse(itemId, 0.0)
      PersonalizedRecommendation(itemId, score, HybridFiltering,
        buildReason(itemId, profile), wasContextBoosted)
    }.toList

    PersonalizedRecommendationResult(userId, recommendations, profile.persona,
      HybridFiltering, coldStart = false, generatedAtMs = System.currentTimeMillis())
  }


  // ==================================================================
  // 7. Contextual Recommendations (real-time)
  // ==================================================================

  private def applyContextualBoost(
      scores: Map[String, Double],
      context: InteractionContext,
      profile: UserProfile
  ): Map[String, Double] =
    scores.map { case (itemId, score) =>
      val item  = items.getOrElse(itemId, ItemProfile(id = itemId, title = itemId, category = "unknown"))
      var boost = 1.0
      if (item.tags.contains(context.timeOfDay))                                 boost += 0.20
      if (context.device != "unknown" &&
          item.attributes.get("device").contains(context.device))                boost += 0.15
      if (context.location != "unknown" &&
          item.attributes.get("region").contains(context.location))              boost += 0.25
      if (profile.preferredTimeOfDay.contains(context.timeOfDay))                boost += 0.10
      itemId -> (score * boost)
    }

  def contextualRecommendations(
      userId: String,
      context: InteractionContext,
      limit: Int = recommendationLimit
  ): PersonalizedRecommendationResult =
    hybridRecommendations(userId, context, Set.empty, limit)


  // ==================================================================
  // 8. Cold-Start Handling
  // ==================================================================

  def coldStartRecommendations(
      userId: String,
      context: InteractionContext = InteractionContext(),
      exclude: Set[String] = Set.empty,
      limit: Int = recommendationLimit
  ): List[PersonalizedRecommendation] = {
    val profile = profiles.getOrElse(userId, UserProfile(id = userId))

    val demographicSimilar: Seq[PersonalizedRecommendation] =
      if (profile.demographics.nonEmpty) {
        profiles.values
          .filter(p => p.id != userId && sharedDemographics(p.demographics, profile.demographics) >= 2)
          .flatMap(p => interactions.filter(_.userId == p.id))
          .groupBy(_.itemId)
          .map { case (id, is) => id -> is.map(i => InteractionType.weight(i.interactionType) * i.value).sum }
          .toSeq.sortBy(-_._2).take(limit)
          .map { case (id, score) =>
            PersonalizedRecommendation(id, score, ColdStartFallback,
              "popular with users like you (demographics)")
          }
      } else Nil

    val popular = items.values
      .filterNot(i => exclude.contains(i.id))
      .toSeq.sortBy(-_.popularityScore).take(limit)
      .map(item => PersonalizedRecommendation(item.id, item.popularityScore,
        ColdStartFallback, s"trending in ${item.category}"))

    val contextPrimed = items.values
      .filter(item => item.tags.contains(context.timeOfDay) || item.tags.contains(context.device))
      .toSeq.sortBy(-_.popularityScore).take(4)
      .map(item => PersonalizedRecommendation(item.id, item.popularityScore * 1.1,
        ColdStartFallback, s"contextually relevant for ${context.timeOfDay}"))

    (demographicSimilar ++ contextPrimed ++ popular)
      .filterNot(r => exclude.contains(r.itemId))
      .groupBy(_.itemId).values.map(_.maxBy(_.score)).toList
      .sortBy(-_.score).take(limit)
  }

  private def sharedDemographics(a: Map[String, String], b: Map[String, String]): Int =
    a.count { case (k, v) => b.get(k).contains(v) }


  // ==================================================================
  // Analytics-facing methods (unchanged surface, enhanced internals)
  // ==================================================================

  def analyticsRecommendations(
      signal: PortfolioSignal,
      health: PortfolioHealthScore,
      events: Seq[StreamEvent],
      modules: List[ModuleActivity],
      userId: Option[String] = None,
      context: InteractionContext = InteractionContext()
  ): List[RecommendationCard] = {
    val buffer = mutable.ListBuffer.empty[RecommendationCard]

    if (signal.risk > 65.0)
      buffer += RecommendationCard("rec-risk-001", "high",
        "Reduce risk exposure by triaging blocked/failed events in office and exchange modules.")
    if (signal.cashFlow < 50.0)
      buffer += RecommendationCard("rec-cashflow-001", "high",
        "Prioritize paid backlog work and recovery actions for wallet/campaign pipelines.")
    if (signal.productivity < 50.0)
      buffer += RecommendationCard("rec-productivity-001", "medium",
        "Rebalance sprint workload and reduce WIP across active workspace lanes.")
    if (signal.collaboration < 50.0)
      buffer += RecommendationCard("rec-collab-001", "medium",
        "Increase team sync cadence and direct-message response coverage.")

    modules.filter(_.events < 2).map(_.module).pipe { low =>
      if (low.nonEmpty)
        buffer += RecommendationCard("rec-coverage-001", "low",
          s"Low activity detected in modules: ${low.mkString(", ")}; verify sync/integration coverage.")
    }

    health.recommendations.zipWithIndex.foreach { case (msg, idx) =>
      buffer += RecommendationCard(f"rec-health-${idx + 1}%03d", "medium", msg)
    }

    val stale = events.count(e => containsAny(e.eventType, "stale", "timeout"))
    if (stale > 0)
      buffer += RecommendationCard("rec-stream-001", "medium",
        s"Detected $stale stale/timeout events; inspect streaming ingest latency.")

    // Persona-aware overlay
    userId.flatMap(profiles.get).foreach { profile =>
      profile.persona match {
        case PowerUser =>
          buffer += RecommendationCard("rec-persona-power", "low",
            "Power user detected – enable advanced analytics dashboards and batch export features.")
        case Explorer =>
          buffer += RecommendationCard("rec-persona-explorer", "low",
            "Exploratory usage pattern – surface cross-module insight cards and discovery prompts.")
        case CasualBrowser =>
          buffer += RecommendationCard("rec-persona-casual", "low",
            "Low engagement pattern – simplify the primary action path and reduce information density.")
        case Collaborator =>
          buffer += RecommendationCard("rec-persona-collab", "low",
            "High collaboration signal – promote team sharing workflows and review pipelines.")
        case _ =>
      }
    }

    buffer.toList.groupBy(_.message).values.map(_.head).toList
      .sortBy(c => priorityRank(c.priority)).take(recommendationLimit)
  }

  // Backward-compatible overload
  def analyticsRecommendations(
      signal: PortfolioSignal,
      health: PortfolioHealthScore,
      events: Seq[StreamEvent],
      modules: List[ModuleActivity]
  ): List[RecommendationCard] =
    analyticsRecommendations(signal, health, events, modules, None, InteractionContext())

  def analyticsDiscover(
      events: Seq[StreamEvent],
      modules: List[ModuleActivity]
  ): List[DiscoverCard] = {
    if (events.isEmpty) {
      return List(DiscoverCard("discover-empty-001", "No Stream Activity",
        "No events in window. Start ingesting module telemetry."))
    }
    val providers = events.flatMap(_.metadata.get("provider")).distinct.sorted
    val dominant  = modules.headOption
    val topTypes  = events.groupBy(_.eventType).toList.sortBy(-_._2.size).take(3)
    val cards     = mutable.ListBuffer.empty[DiscoverCard]

    dominant.foreach { m =>
      cards += DiscoverCard("discover-module-001",
        s"${m.module} is the dominant activity module",
        f"${m.events} events with net impact ${m.netImpact}%.2f")
    }
    if (providers.nonEmpty)
      cards += DiscoverCard("discover-provider-001", "Connected providers discovered", providers.mkString(", "))
    topTypes.foreach { case (et, g) =>
      cards += DiscoverCard(s"discover-event-${safeId(et)}", s"Trending event: $et",
        s"${g.size} events in current window")
    }
    cards.toList.take(discoverLimit)
  }

  def analyticsExplore(
      events: Seq[StreamEvent],
      modules: List[ModuleActivity],
      recommendations: List[RecommendationCard]
  ): List[ExploreCard] = {
    val cards         = mutable.ListBuffer.empty[ExploreCard]
    val activeModules = modules.map(_.module).toSet
    cards += ExploreCard("explore-portfolio-001", "Explore portfolio item concentration",
      "Compare top module activity against portfolio risk and capital allocation.")
    if (activeModules.contains("office") && !activeModules.contains("exchange"))
      cards += ExploreCard("explore-exchange-001", "Office-heavy, exchange-light activity",
        "Explore exchange integrations for capital and deal flow automation.")
    if (activeModules.contains("community") && !activeModules.contains("marketplace"))
      cards += ExploreCard("explore-marketplace-001", "Community activity can convert to marketplace",
        "Explore publishing community demand signals as marketplace listings.")
    if (recommendations.exists(_.priority == "high"))
      cards += ExploreCard("explore-stabilize-001", "High-priority risk or cashflow actions detected",
        "Run stabilization playbook before expanding experimentation lanes.")
    val innovationSignals = events.count(e => containsAny(e.eventType, "prototype", "experiment", "idea"))
    if (innovationSignals > 0)
      cards += ExploreCard("explore-innovation-001", "Innovation stream detected",
        s"Promote top $innovationSignals prototype/idea events into studio-to-office execution pipeline.")
    cards.toList.take(exploreLimit)
  }

  def moduleRecommendations(
      module: String, signal: PortfolioSignal, health: PortfolioHealthScore,
      events: Seq[StreamEvent], moduleActivity: List[ModuleActivity],
      errorRate: Double, p95LatencyMs: Double, avgQueueDepth: Double, avgCpuPct: Double
  ): List[RecommendationCard] = {
    val recs =
      analyticsRecommendations(signal, health, events, moduleActivity) ++
        moduleOperationalRecommendations(module, errorRate, p95LatencyMs, avgQueueDepth, avgCpuPct)
    recs.groupBy(_.message).values.map(_.head).toList
      .sortBy(c => priorityRank(c.priority)).take(recommendationLimit)
  }

  def moduleDiscover(module: String, metrics: Seq[ModuleMetricSample],
                     events: Seq[StreamEvent], moduleActivity: List[ModuleActivity]): List[DiscoverCard] =
    (analyticsDiscover(events, moduleActivity) ++ moduleDiscoverInternal(module, metrics)).take(discoverLimit)

  def moduleExplore(module: String, errorRate: Double, blockedRate: Double,
                    events: Seq[StreamEvent], moduleActivity: List[ModuleActivity],
                    recommendations: List[RecommendationCard]): List[ExploreCard] =
    (analyticsExplore(events, moduleActivity, recommendations) ++
      moduleExploreInternal(module, errorRate, blockedRate)).take(exploreLimit)

  def systemRecommendations(host: HostRealtimeSnapshot,
                             modules: Seq[ModuleRealtimeSnapshot]): List[RecommendationCard] = {
    val cards = mutable.ListBuffer.empty[RecommendationCard]
    if (host.status == "red")
      cards += RecommendationCard("sys-rec-host-red", "high",
        "Host saturation is critical; scale host resources and rebalance module workloads.")
    else if (host.status == "amber")
      cards += RecommendationCard("sys-rec-host-amber", "medium",
        "Host is trending hot; tune scheduler/process limits and watch memory pressure.")
    val redMods = modules.filter(_.health.status == "red").map(_.module)
    if (redMods.nonEmpty)
      cards += RecommendationCard("sys-rec-module-red", "high",
        s"Critical module health detected in: ${redMods.mkString(", ")}.")
    val latMods = modules.filter(_.p95LatencyMs >= 1200.0).map(_.module)
    if (latMods.nonEmpty)
      cards += RecommendationCard("sys-rec-latency", "high",
        s"High p95 latency in modules: ${latMods.mkString(", ")}.")
    val qMods = modules.filter(_.avgQueueDepth >= 150.0).map(_.module)
    if (qMods.nonEmpty)
      cards += RecommendationCard("sys-rec-queue", "medium",
        s"Backpressure risk in modules: ${qMods.mkString(", ")}.")
    cards.toList.groupBy(_.message).values.map(_.head).toList
      .sortBy(c => priorityRank(c.priority)).take(recommendationLimit)
  }

  def systemDiscover(host: HostRealtimeSnapshot, modules: Seq[ModuleRealtimeSnapshot]): List[DiscoverCard] = {
    val cards = mutable.ListBuffer.empty[DiscoverCard]
    cards += DiscoverCard("sys-discover-host", s"Host ${host.hostId} status ${host.status}",
      f"cpu=${host.cpuAvgPct}%.1f%% mem=${host.memoryAvgPct}%.1f%% disk=${host.diskAvgPct}%.1f%% saturation=${host.saturationScore}%.1f")
    modules.sortBy(-_.throughputPerMin).headOption.foreach { top =>
      cards += DiscoverCard("sys-discover-throughput", s"${top.module} is the throughput leader",
        f"${top.throughputPerMin}%.2f units/min in current window")
    }
    modules.sortBy(-_.errorRate).headOption.filter(_.errorRate > 0).foreach { top =>
      cards += DiscoverCard("sys-discover-error", s"${top.module} has highest error rate",
        f"${top.errorRate * 100.0}%.2f%% error rate")
    }
    cards.toList.take(discoverLimit)
  }

  def systemExplore(host: HostRealtimeSnapshot, modules: Seq[ModuleRealtimeSnapshot]): List[ExploreCard] = {
    val cards = mutable.ListBuffer.empty[ExploreCard]
    val moduleSet = modules.map(_.module).toSet
    cards += ExploreCard("sys-explore-capacity", "Explore capacity planning model",
      "Use module throughput and host saturation to simulate scale-up/scale-out plans.")
    if (moduleSet.contains("kogi-kernel") || moduleSet.contains("kernel"))
      cards += ExploreCard("sys-explore-kernel", "Explore kernel scheduler instrumentation",
        "Correlate kernel queue depth and dispatch latency with module p95 latency.")
    if (moduleSet.contains("kogi-host") || moduleSet.contains("host"))
      cards += ExploreCard("sys-explore-host", "Explore host orchestration optimization",
        "Tune module isolation quotas based on observed cpu/memory contention.")
    if (host.status != "green")
      cards += ExploreCard("sys-explore-resilience", "Explore resilience hardening",
        "Enable failover routing and workload shedding policies for peak periods.")
    cards.toList.take(exploreLimit)
  }


  // ==================================================================
  // Introspection / admin helpers
  // ==================================================================

  def profileSummary(userId: String): Map[String, Any] = {
    val p = profiles.getOrElse(userId, UserProfile(id = userId))
    Map(
      "id"                  -> p.id,
      "persona"             -> PersonaLabel.label(p.persona),
      "persona_confidence"  -> p.personaConfidence,
      "traits"              -> p.personaTraits,
      "total_interactions"  -> p.totalInteractions,
      "dominant_categories" -> p.dominantCategories,
      "preferred_time"      -> p.preferredTimeOfDay,
      "preferred_devices"   -> p.preferredDevices,
      "last_seen_ms"        -> p.lastSeenMs,
      "cold_start"          -> (p.totalInteractions < coldStartMinInteractions)
    )
  }

  def allPersonas(): Map[String, String]   = profiles.view.mapValues(p => PersonaLabel.label(p.persona)).toMap
  def feedbackHistory(userId: String, limit: Int = 50): Seq[FeedbackRecord] =
    feedbackLog.filter(_.userId == userId).takeRight(limit)
  def itemCatalogSize: Int  = items.size
  def profileCount: Int     = profiles.size
  def interactionCount: Int = interactions.size


  // ==================================================================
  // Private helpers
  // ==================================================================

  private def buildReason(itemId: String, profile: UserProfile): String = {
    val item = items.getOrElse(itemId, ItemProfile(id = itemId, title = itemId, category = "unknown"))
    val matchedTags = item.tags.filter(t => profile.preferenceVector.getOrElse(t, 0.0) > 0.5)
    if (matchedTags.nonEmpty) s"matches your interest in ${matchedTags.take(2).mkString(", ")}"
    else s"recommended based on your ${PersonaLabel.label(profile.persona)} activity profile"
  }

  private def moduleOperationalRecommendations(module: String, errorRate: Double,
      p95LatencyMs: Double, avgQueueDepth: Double, avgCpuPct: Double): List[RecommendationCard] =
    List(
      Option.when(errorRate >= 0.15)(RecommendationCard(s"rec-${safeId(module)}-error", "high",
        s"$module error rate is elevated; trigger incident playbook and inspect failing routes.")),
      Option.when(p95LatencyMs >= 1200.0)(RecommendationCard(s"rec-${safeId(module)}-latency", "high",
        s"$module latency p95 is above threshold; scale workers and optimize slow handlers.")),
      Option.when(avgQueueDepth >= 150.0)(RecommendationCard(s"rec-${safeId(module)}-queue", "medium",
        s"$module queue depth is growing; increase consumer concurrency or apply backpressure.")),
      Option.when(avgCpuPct >= 80.0)(RecommendationCard(s"rec-${safeId(module)}-cpu", "medium",
        s"$module compute utilization is high; evaluate horizontal scaling."))
    ).flatten

  private def moduleDiscoverInternal(module: String, metrics: Seq[ModuleMetricSample]): List[DiscoverCard] = {
    if (metrics.isEmpty) return Nil
    val providers = metrics.flatMap(_.metadata.get("provider")).distinct
    val routes    = metrics.flatMap(_.metadata.get("route")).groupBy(identity).view.mapValues(_.size).toMap
    val cards     = mutable.ListBuffer.empty[DiscoverCard]
    cards += DiscoverCard(s"discover-${safeId(module)}-perf", s"$module realtime telemetry available",
      f"${metrics.size} metric samples with avg latency ${average(metrics.map(_.latencyMs))}%.2f ms")
    if (providers.nonEmpty)
      cards += DiscoverCard(s"discover-${safeId(module)}-providers", s"$module provider coverage",
        providers.sorted.mkString(", "))
    routes.toList.sortBy(-_._2).headOption.foreach { case (route, count) =>
      cards += DiscoverCard(s"discover-${safeId(module)}-route", s"$module hottest route", s"$route ($count samples)")
    }
    cards.toList
  }

  private def moduleExploreInternal(module: String, errorRate: Double, blockedRate: Double): List[ExploreCard] = {
    val cards = mutable.ListBuffer.empty[ExploreCard]
    cards += ExploreCard(s"explore-${safeId(module)}-optimize", s"Explore $module optimization lane",
      s"Open $module performance board and test one throughput optimization experiment.")
    if (errorRate >= 0.10 || blockedRate >= 0.12)
      cards += ExploreCard(s"explore-${safeId(module)}-stability", s"Explore $module stabilization lane",
        s"Correlate $module incidents with upstream dependencies (kernel/host/server/gateway).")
    cards.toList
  }

  // Scala 2 shim for Option.when
  private implicit class OptionWhen[A](val a: A) extends AnyVal {
    def pipe[B](f: A => B): B = f(a)
  }
  private object Option {
    def when[A](cond: Boolean)(a: => A): Option[A] = if (cond) Some(a) else None
  }

  private def clamp(v: Double, lo: Double, hi: Double): Double = math.max(lo, math.min(hi, v))
  private def ratio(num: Double, den: Double): Double = if (den <= 0) 0.0 else num / den
  private def containsAny(value: String, parts: String*): Boolean =
    parts.exists(value.toLowerCase.contains)
  private def priorityRank(p: String): Int = p match {
    case "high" | "red" => 0; case "medium" | "amber" => 1; case _ => 2
  }
  private def safeId(input: String): String = input.toLowerCase.replaceAll("[^a-z0-9]+", "-")
  private def average(values: Seq[Double]): Double =
    if (values.isEmpty) 0.0 else values.sum / values.size
}
