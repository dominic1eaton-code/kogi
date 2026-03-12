// PersonalizationEngine — personalized recommendations and content delivery,
// profile preferences, user segmentation, persona building, labeling, and
// dynamic content adaptation.
//
// Future features: real-time personalization, A/B testing, multi-armed bandits,
// and integration with external recommendation systems.
//
// Architecture:
//   PersonalizationEngine composes over RecommendationEngine (for collaborative
//   / content-based signals) and adds the platform-level concerns:
//     • Content delivery adaptation (layout, density, ordering)
//     • User segmentation (cohort assignment + drift detection)
//     • A/B experiment scaffolding (variant assignment, exposure tracking)
//     • Multi-armed bandit slot (UCB1 implemented, extensible)
//     • Profile preference management (explicit + implicit)
//     • Persona labeling lifecycle (build → confirm → drift → update)
//     • Dynamic content adaptation rules (persona × context → template)
package kogi.engine

import scala.collection.mutable
import scala.util.Random


// ─────────────────────────────────────────────────────────────────────────────
// Preference & Profile types
// ─────────────────────────────────────────────────────────────────────────────

/** An explicit user preference setting. */
final case class UserPreference(
    key: String,             // e.g. "theme", "language", "content_density"
    value: String,           // e.g. "dark", "en", "compact"
    source: String = "explicit",  // "explicit" | "inferred" | "default"
    updatedMs: Long = System.currentTimeMillis()
)

/** Resolved preference with fallback chain: explicit → inferred → default. */
final case class ResolvedPreferences(
    profileId: String,
    preferences: Map[String, UserPreference],
    persona: PersonaLabel,
    segment: String,
    resolvedAtMs: Long
)


// ─────────────────────────────────────────────────────────────────────────────
// Content Delivery Adaptation
// ─────────────────────────────────────────────────────────────────────────────

/** Feed / layout density setting. */
sealed trait ContentDensity
case object Compact    extends ContentDensity   // PowerUser, Specialist
case object Standard   extends ContentDensity   // default
case object Spacious   extends ContentDensity   // CasualBrowser, Newcomer

/** Content ordering strategy. */
sealed trait ContentOrdering
case object RelevanceFirst  extends ContentOrdering   // Explorer, Specialist
case object RecencyFirst    extends ContentOrdering   // EarlyAdopter, PowerUser
case object TrendingFirst   extends ContentOrdering   // CasualBrowser, Newcomer
case object ValueFirst      extends ContentOrdering   // ValueSeeker
case object PersonalFirst   extends ContentOrdering   // Collaborator

/**
 * Adapted content delivery configuration derived from the user's persona,
 * preferences, and context.
 */
final case class ContentDeliveryConfig(
    profileId: String,
    density: ContentDensity,
    ordering: ContentOrdering,
    maxFeedItems: Int,
    showAdvancedFilters: Boolean,
    showAnalyticsSidebar: Boolean,
    autoExpandItems: Boolean,
    highlightNewContent: Boolean,
    suppressLowEngagement: Boolean,
    adaptedAtMs: Long
)


// ─────────────────────────────────────────────────────────────────────────────
// Segmentation
// ─────────────────────────────────────────────────────────────────────────────

/**
 * A user segment definition.
 *
 * @param id      Stable identifier (e.g. "high-value-investors").
 * @param name    Human-readable label.
 * @param rules   Map of attribute key → allowed values.  A user matches if
 *                all rules are satisfied.
 */
final case class Segment(
    id: String,
    name: String,
    rules: Map[String, Seq[String]],   // attribute → allowed values
    description: String = ""
)

/** Result of assigning a profile to one or more segments. */
final case class SegmentAssignment(
    profileId: String,
    segments: List[String],            // matched segment IDs
    primarySegment: Option[String],
    confidence: Double,
    assignedAtMs: Long
)


// ─────────────────────────────────────────────────────────────────────────────
// A/B Testing
// ─────────────────────────────────────────────────────────────────────────────

/** A single experiment variant. */
final case class ExperimentVariant(
    id: String,         // e.g. "control", "treatment-a"
    name: String,
    weight: Double = 1.0   // relative traffic weight
)

/** An A/B experiment definition. */
final case class Experiment(
    id: String,
    name: String,
    variants: List[ExperimentVariant],
    targetSegments: List[String] = Nil,   // empty = all users
    active: Boolean = true
)

/** The variant assignment for a specific user in a specific experiment. */
final case class ExperimentAssignment(
    experimentId: String,
    variantId: String,
    profileId: String,
    assignedAtMs: Long
)

/** Exposure event recorded when a user sees experiment content. */
final case class ExposureEvent(
    experimentId: String,
    variantId: String,
    profileId: String,
    convertedToInteraction: Boolean = false,
    timestampMs: Long = System.currentTimeMillis()
)


// ─────────────────────────────────────────────────────────────────────────────
// Multi-Armed Bandit (UCB1)
// ─────────────────────────────────────────────────────────────────────────────

/** Arm state for the bandit. */
private final case class BanditArm(
    id: String,
    pulls: Int = 0,
    totalReward: Double = 0.0
) {
  def avgReward: Double = if (pulls == 0) 0.0 else totalReward / pulls
  def ucb1(totalPulls: Int): Double =
    if (pulls == 0) Double.MaxValue
    else avgReward + math.sqrt(2.0 * math.log(totalPulls.toDouble) / pulls)
}

/**
 * Result of a bandit arm selection.
 *
 * @param armId        The chosen arm / variant.
 * @param expectedReward  UCB1 estimated reward.
 */
final case class BanditSelection(
    slotId: String,
    armId: String,
    expectedReward: Double
)


// ─────────────────────────────────────────────────────────────────────────────
// Personalization request / result types
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Unified personalization request.  At minimum provide profileId; supply
 * context, preferences, and persona to get richer adaptation.
 */
final case class PersonalizationRequest(
    profileId: String,
    context: Map[String, String] = Map.empty,     // device, location, timeOfDay
    preferences: Map[String, String] = Map.empty, // explicit prefs from UI
    persona: Option[PersonaLabel] = None          // override if known
)

/** A single personalization recommendation card. */
final case class PersonalizationRecommendation(
    id: String,
    priority: String,
    message: String
)

/**
 * Full personalization response: recommendations + delivery config +
 * segment info + active experiment assignments.
 */
final case class PersonalizationResult(
    profileId: String,
    recommendations: List[PersonalizationRecommendation],
    deliveryConfig: ContentDeliveryConfig,
    resolvedPreferences: ResolvedPreferences,
    segmentAssignment: SegmentAssignment,
    experimentAssignments: List[ExperimentAssignment],
    persona: PersonaLabel,
    personaTraits: List[String],
    personaConfidence: Double,
    generatedAtMs: Long
)

/** Lightweight result for feed / content ranking calls. */
final case class RankedContent(
    itemId: String,
    score: Double,
    ordering: ContentOrdering,
    personaBoosted: Boolean
)

/** Persona drift signal: flagged when behaviour has diverged from label. */
final case class PersonaDrift(
    profileId: String,
    currentPersona: PersonaLabel,
    suggestedPersona: PersonaLabel,
    driftScore: Double,           // 0 = no drift, 1 = full drift
    detectedAtMs: Long
)


// ─────────────────────────────────────────────────────────────────────────────
// PersonalizationEngine
// ─────────────────────────────────────────────────────────────────────────────

/**
 * PersonalizationEngine — orchestrates persona-aware content delivery,
 * segmentation, A/B testing, and multi-armed bandit slot management.
 *
 * Composes with [[RecommendationEngine]] for collaborative and content-based
 * signals; adds platform-level delivery adaptation and experiment tracking.
 *
 * @param recommendationEngine  Underlying recommendation substrate.
 * @param defaultFeedLimit      Default number of feed items.
 * @param driftThreshold        Drift score above which a persona re-evaluation
 *                               is triggered (0–1).
 * @param rng                   Random source for A/B assignment (injectable for
 *                               deterministic tests).
 */
final class PersonalizationEngine(
    recommendationEngine: RecommendationEngine = new RecommendationEngine(),
    defaultFeedLimit: Int = 20,
    driftThreshold: Double = 0.4,
    rng: Random = new Random()
) {

  // ── State ──────────────────────────────────────────────────────────────────

  // Explicit preference store: profileId → key → UserPreference
  private val preferenceStore: mutable.Map[String, mutable.Map[String, UserPreference]] =
    mutable.Map.empty

  // Default preferences (platform-wide fallbacks)
  private val defaultPreferences: Map[String, UserPreference] = Map(
    "theme"             -> UserPreference("theme",             "light",    "default"),
    "language"          -> UserPreference("language",          "en",       "default"),
    "content_density"   -> UserPreference("content_density",   "standard", "default"),
    "notifications"     -> UserPreference("notifications",     "on",       "default"),
    "feed_ordering"     -> UserPreference("feed_ordering",     "relevance","default"),
    "analytics_sidebar" -> UserPreference("analytics_sidebar", "off",      "default")
  )

  // Segment registry
  private val segments: mutable.Map[String, Segment] = mutable.Map.empty

  // Segment assignment cache: profileId → SegmentAssignment
  private val segmentCache: mutable.Map[String, SegmentAssignment] = mutable.Map.empty

  // Experiment registry + exposure log
  private val experiments: mutable.Map[String, Experiment] = mutable.Map.empty
  private val experimentAssignments: mutable.Map[String, mutable.Map[String, ExperimentAssignment]] =
    mutable.Map.empty  // profileId → experimentId → assignment
  private var exposureLog: Vector[ExposureEvent] = Vector.empty

  // Bandit slots: slotId → armId → BanditArm
  private val banditSlots: mutable.Map[String, mutable.Map[String, BanditArm]] = mutable.Map.empty


  // ══════════════════════════════════════════════════════════════════════════
  // 1.  Core Personalization Entry Point
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Produce a full PersonalizationResult for a profile.
   * This is the primary method callers should use.
   */
  def personalize(request: PersonalizationRequest): PersonalizationResult = {

    val profileId = request.profileId

    // Resolve persona: explicit override → engine profile → context inference
    val resolvedPersona = resolvePersona(request)

    // Infer preferences from context + persona if not explicitly set
    val inferredPrefs  = inferPreferences(resolvedPersona, request.context)
    val explicitPrefs  = preferenceStore.getOrElse(profileId, mutable.Map.empty).toMap
    val merged         = inferredPrefs ++ explicitPrefs   // explicit wins

    val resolvedPrefs  = ResolvedPreferences(
      profileId     = profileId,
      preferences   = (defaultPreferences ++ merged).view.mapValues(identity).toMap,
      persona       = resolvedPersona,
      segment       = segmentCache.get(profileId).flatMap(_.primarySegment).getOrElse("general"),
      resolvedAtMs  = System.currentTimeMillis()
    )

    // Delivery config
    val deliveryConfig = buildDeliveryConfig(profileId, resolvedPersona, resolvedPrefs)

    // Segmentation
    val segAssignment = assignSegments(profileId, resolvedPersona, request.context ++ request.preferences.map { case (k, v) => k -> v })

    // A/B assignments for active experiments targeting this user's segments
    val expAssignments = activeExperimentsFor(profileId, segAssignment.segments)
      .map(exp => assignExperiment(profileId, exp))

    // Persona build result from recommendation engine
    val personaBuild   = recommendationEngine.buildPersona(profileId)

    // Recommendations
    val recs = buildPersonalizationRecommendations(resolvedPersona, request.context,
      resolvedPrefs, segAssignment)

    PersonalizationResult(
      profileId             = profileId,
      recommendations       = recs,
      deliveryConfig        = deliveryConfig,
      resolvedPreferences   = resolvedPrefs,
      segmentAssignment     = segAssignment,
      experimentAssignments = expAssignments,
      persona               = resolvedPersona,
      personaTraits         = personaBuild.traits,
      personaConfidence     = personaBuild.confidence,
      generatedAtMs         = System.currentTimeMillis()
    )
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 2.  Preference Management
  // ══════════════════════════════════════════════════════════════════════════

  /** Set an explicit preference for a profile. */
  def setPreference(profileId: String, key: String, value: String): Unit = {
    val store = preferenceStore.getOrElseUpdate(profileId, mutable.Map.empty)
    store(key) = UserPreference(key, value, "explicit", System.currentTimeMillis())
  }

  /** Set multiple preferences in one call. */
  def setPreferences(profileId: String, prefs: Map[String, String]): Unit =
    prefs.foreach { case (k, v) => setPreference(profileId, k, v) }

  /** Remove a preference (reverts to inferred or default). */
  def clearPreference(profileId: String, key: String): Unit =
    preferenceStore.get(profileId).foreach(_ -= key)

  /** Get the raw explicit preferences for a profile. */
  def getExplicitPreferences(profileId: String): Map[String, UserPreference] =
    preferenceStore.getOrElse(profileId, mutable.Map.empty).toMap

  /**
   * Resolve all preferences for a profile applying the full fallback chain:
   * explicit → inferred → default.
   */
  def resolvePreferences(profileId: String,
                         context: Map[String, String] = Map.empty): ResolvedPreferences = {
    val persona       = resolvePersona(PersonalizationRequest(profileId, context))
    val inferred      = inferPreferences(persona, context)
    val explicit      = preferenceStore.getOrElse(profileId, mutable.Map.empty).toMap
    val merged        = (defaultPreferences ++ inferred ++ explicit)
    ResolvedPreferences(
      profileId    = profileId,
      preferences  = merged,
      persona      = persona,
      segment      = segmentCache.get(profileId).flatMap(_.primarySegment).getOrElse("general"),
      resolvedAtMs = System.currentTimeMillis()
    )
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 3.  Content Delivery Adaptation
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Build a ContentDeliveryConfig tuned to the profile's persona and
   * resolved preferences.
   */
  def buildDeliveryConfig(
      profileId: String,
      persona: PersonaLabel,
      resolvedPrefs: ResolvedPreferences
  ): ContentDeliveryConfig = {

    val prefDensity   = resolvedPrefs.preferences.get("content_density").map(_.value).getOrElse("standard")
    val prefOrdering  = resolvedPrefs.preferences.get("feed_ordering").map(_.value).getOrElse("relevance")
    val showSidebar   = resolvedPrefs.preferences.get("analytics_sidebar").map(_.value).contains("on")

    val density: ContentDensity = prefDensity match {
      case "compact"  => Compact
      case "spacious" => Spacious
      case _          => personaDensity(persona)
    }

    val ordering: ContentOrdering = prefOrdering match {
      case "recency"   => RecencyFirst
      case "trending"  => TrendingFirst
      case "value"     => ValueFirst
      case "personal"  => PersonalFirst
      case _           => personaOrdering(persona)
    }

    ContentDeliveryConfig(
      profileId              = profileId,
      density                = density,
      ordering               = ordering,
      maxFeedItems           = personaFeedLimit(persona),
      showAdvancedFilters    = persona == PowerUser || persona == Specialist,
      showAnalyticsSidebar   = showSidebar || persona == PowerUser,
      autoExpandItems        = persona == Explorer || persona == EarlyAdopter,
      highlightNewContent    = persona == EarlyAdopter || persona == Explorer,
      suppressLowEngagement  = persona == ValueSeeker || persona == Specialist,
      adaptedAtMs            = System.currentTimeMillis()
    )
  }

  /**
   * Rank a sequence of content item IDs for a profile.
   * Returns items ordered by personalized score descending.
   */
  def rankContent(
      profileId: String,
      itemIds: Seq[String],
      context: InteractionContext = InteractionContext()
  ): List[RankedContent] = {
    val result = recommendationEngine.hybridRecommendations(
      profileId, context, exclude = Set.empty, limit = itemIds.size.max(1)
    )
    val recScores = result.recommendations.map(r => r.itemId -> r.score).toMap
    val persona   = result.persona

    itemIds.map { id =>
      val baseScore      = recScores.getOrElse(id, 0.0)
      val (boost, order) = personaContentBoost(persona, id, context)
      RankedContent(id, baseScore * boost, order, boost > 1.0)
    }.sortBy(-_.score).toList
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 4.  User Segmentation
  // ══════════════════════════════════════════════════════════════════════════

  /** Register a segment definition. */
  def registerSegment(segment: Segment): Unit =
    segments(segment.id) = segment

  /** Register multiple segments. */
  def registerSegments(ss: Seq[Segment]): Unit = ss.foreach(registerSegment)

  /**
   * Assign a profile to matching segments, cache the result, and return it.
   *
   * @param attributes  Flat attribute map for the profile (can include persona,
   *                    role, location, interests, etc.).
   */
  def assignSegments(
      profileId: String,
      persona: PersonaLabel,
      attributes: Map[String, String]
  ): SegmentAssignment = {

    // Always include the persona label as an attribute for rule matching
    val enriched = attributes + ("persona" -> PersonaLabel.label(persona))

    val matched = segments.values.filter { seg =>
      seg.rules.forall { case (key, allowed) =>
        enriched.get(key).exists(v => allowed.contains(v))
      }
    }.toList.sortBy(_.id)

    val primary = matched.headOption.map(_.id)
    val confidence =
      if (matched.isEmpty) 0.0
      else math.min(1.0, matched.size.toDouble / segments.size.toDouble * 2.0)

    val assignment = SegmentAssignment(
      profileId      = profileId,
      segments       = matched.map(_.id),
      primarySegment = primary,
      confidence     = confidence,
      assignedAtMs   = System.currentTimeMillis()
    )
    segmentCache(profileId) = assignment
    assignment
  }

  /** Return the cached segment assignment for a profile (if any). */
  def getSegmentAssignment(profileId: String): Option[SegmentAssignment] =
    segmentCache.get(profileId)

  /** List all profiles in a given segment. */
  def profilesInSegment(segmentId: String): List[String] =
    segmentCache.values
      .filter(_.segments.contains(segmentId))
      .map(_.profileId)
      .toList


  // ══════════════════════════════════════════════════════════════════════════
  // 5.  Persona Labeling Lifecycle
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Build or refresh the persona for a profile.
   * Delegates to RecommendationEngine and optionally checks for drift.
   */
  def buildPersona(profileId: String): PersonaBuildResult =
    recommendationEngine.buildPersona(profileId)

  /**
   * Detect whether the profile's observed behaviour has drifted away from
   * its assigned persona.
   *
   * Drift is measured as 1 − personaSimilarity(current, suggested).
   */
  def detectPersonaDrift(profileId: String): Option[PersonaDrift] = {
    val current  = recommendationEngine.getProfile(profileId)
      .map(_.persona).getOrElse(return None)
    val fresh    = buildPersona(profileId)
    val suggested = fresh.persona
    if (current == suggested) return None

    val drift = personaDistance(current, suggested)
    if (drift < driftThreshold) None
    else Some(PersonaDrift(
      profileId        = profileId,
      currentPersona   = current,
      suggestedPersona = suggested,
      driftScore       = drift,
      detectedAtMs     = System.currentTimeMillis()
    ))
  }

  /**
   * Apply the suggested persona if drift is above threshold, then re-run
   * delivery adaptation.
   *
   * @return The updated PersonaBuildResult if the persona was changed.
   */
  def applyPersonaDriftIfNeeded(profileId: String): Option[PersonaBuildResult] =
    detectPersonaDrift(profileId).map { _ =>
      val build = buildPersona(profileId)
      recommendationEngine.getProfile(profileId).foreach { p =>
        recommendationEngine.upsertProfile(p.copy(
          persona           = build.persona,
          personaTraits     = build.traits,
          personaConfidence = build.confidence,
          dominantCategories = build.dominantCategories
        ))
      }
      build
    }


  // ══════════════════════════════════════════════════════════════════════════
  // 6.  A/B Testing
  // ══════════════════════════════════════════════════════════════════════════

  /** Register an experiment. */
  def registerExperiment(experiment: Experiment): Unit =
    experiments(experiment.id) = experiment

  /**
   * Assign a profile to a variant in a specific experiment.
   * Re-uses an existing assignment for the same experiment (sticky).
   */
  def assignExperiment(profileId: String, experiment: Experiment): ExperimentAssignment = {
    val cached = experimentAssignments
      .get(profileId)
      .flatMap(_.get(experiment.id))
    cached.getOrElse {
      val variant = weightedRandomVariant(experiment.variants)
      val assignment = ExperimentAssignment(
        experimentId = experiment.id,
        variantId    = variant.id,
        profileId    = profileId,
        assignedAtMs = System.currentTimeMillis()
      )
      val profileMap = experimentAssignments.getOrElseUpdate(profileId, mutable.Map.empty)
      profileMap(experiment.id) = assignment
      assignment
    }
  }

  /** Record an exposure event (and optionally a conversion). */
  def recordExposure(
      experimentId: String,
      variantId: String,
      profileId: String,
      converted: Boolean = false
  ): Unit = {
    exposureLog = (exposureLog :+ ExposureEvent(
      experimentId, variantId, profileId, converted
    )).takeRight(500000)
  }

  /**
   * Compute per-variant exposure and conversion rates for an experiment.
   * Returns Map[variantId → (exposures, conversions, convRate)].
   */
  def experimentStats(experimentId: String): Map[String, (Int, Int, Double)] = {
    val relevant = exposureLog.filter(_.experimentId == experimentId)
    relevant.groupBy(_.variantId).view.mapValues { events =>
      val total   = events.size
      val conv    = events.count(_.convertedToInteraction)
      val rate    = if (total == 0) 0.0 else conv.toDouble / total
      (total, conv, rate)
    }.toMap
  }

  /** Return all assignments for a profile across all experiments. */
  def experimentAssignmentsFor(profileId: String): List[ExperimentAssignment] =
    experimentAssignments.get(profileId).map(_.values.toList).getOrElse(Nil)


  // ══════════════════════════════════════════════════════════════════════════
  // 7.  Multi-Armed Bandit (UCB1)
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Register arms for a bandit slot.  Each arm is identified by a string ID
   * (e.g. a recommendation strategy name or content template ID).
   */
  def registerBanditSlot(slotId: String, armIds: Seq[String]): Unit = {
    val slot = banditSlots.getOrElseUpdate(slotId, mutable.Map.empty)
    armIds.foreach { id => slot.getOrElseUpdate(id, BanditArm(id)) }
  }

  /**
   * Select the best arm for a slot using UCB1.
   * Returns the arm with the highest upper confidence bound.
   */
  def selectBanditArm(slotId: String): Option[BanditSelection] = {
    val slot = banditSlots.getOrElse(slotId, return None)
    if (slot.isEmpty) return None
    val total = slot.values.map(_.pulls).sum
    val best  = slot.values.maxBy(_.ucb1(total.max(1)))
    Some(BanditSelection(slotId, best.id, best.ucb1(total.max(1))))
  }

  /**
   * Record the reward for a bandit arm pull (reward typically in [0, 1]).
   */
  def recordBanditReward(slotId: String, armId: String, reward: Double): Unit = {
    banditSlots.get(slotId).foreach { slot =>
      slot.get(armId).foreach { arm =>
        slot(armId) = arm.copy(
          pulls       = arm.pulls + 1,
          totalReward = arm.totalReward + reward
        )
      }
    }
  }

  /**
   * Return the current bandit stats for a slot.
   * Map[armId → (pulls, avgReward, ucb1)].
   */
  def banditStats(slotId: String, currentTotalPulls: Int = -1): Map[String, (Int, Double, Double)] = {
    val slot = banditSlots.getOrElse(slotId, return Map.empty)
    val total = if (currentTotalPulls < 0) slot.values.map(_.pulls).sum else currentTotalPulls
    slot.view.mapValues { arm =>
      (arm.pulls, arm.avgReward, arm.ucb1(total.max(1)))
    }.toMap
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 8.  Dynamic Content Adaptation Rules
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Adapt raw content item IDs using persona × context rules:
   * suppresses low-value items for ValueSeekers, promotes new items for
   * EarlyAdopters, etc.
   *
   * @param profileId  Requesting profile.
   * @param itemIds    Raw ordered item IDs from a feed or search.
   * @param context    Current interaction context.
   * @param itemMeta   Optional metadata per item (category, recency, value).
   * @return           Reordered and filtered item IDs.
   */
  def adaptContent(
      profileId: String,
      itemIds: Seq[String],
      context: InteractionContext = InteractionContext(),
      itemMeta: Map[String, Map[String, String]] = Map.empty
  ): List[String] = {

    val persona = resolvePersona(PersonalizationRequest(profileId, Map(
      "device"    -> context.device,
      "location"  -> context.location,
      "timeOfDay" -> context.timeOfDay
    )))

    val scored: Seq[(String, Double)] = itemIds.zipWithIndex.map { case (id, rank) =>
      val meta = itemMeta.getOrElse(id, Map.empty)
      val score = adaptationScore(persona, rank, meta, context)
      id -> score
    }

    persona match {
      case ValueSeeker =>
        // Suppress if flagged as low-value
        scored
          .filterNot { case (id, _) =>
            itemMeta.get(id).flatMap(_.get("value")).contains("low")
          }
          .sortBy(-_._2).map(_._1).toList

      case EarlyAdopter =>
        // Promote items flagged as new/experimental to the top
        val (newItems, rest) = scored.partition { case (id, _) =>
          val meta = itemMeta.getOrElse(id, Map.empty)
          meta.get("recency").contains("new") || meta.get("status").contains("experimental")
        }
        (newItems.sortBy(-_._2) ++ rest.sortBy(-_._2)).map(_._1).toList

      case CasualBrowser | Newcomer =>
        // Limit depth, prefer trending
        scored.sortBy(-_._2).take(defaultFeedLimit).map(_._1).toList

      case PowerUser | Specialist =>
        // Full list, scored order
        scored.sortBy(-_._2).map(_._1).toList

      case _ =>
        scored.sortBy(-_._2).map(_._1).toList
    }
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 9.  Interaction Passthrough to RecommendationEngine
  // ══════════════════════════════════════════════════════════════════════════

  /** Forward an interaction to the underlying recommendation engine. */
  def recordInteraction(interaction: UserInteraction): UserProfile =
    recommendationEngine.recordInteraction(interaction)

  /** Forward a batch of interactions. */
  def recordInteractions(batch: Seq[UserInteraction]): Unit =
    recommendationEngine.recordInteractions(batch)

  /** Register an item in the recommendation substrate. */
  def registerItem(item: ItemProfile): Unit =
    recommendationEngine.upsertItem(item)

  /** Get personalized recommendations via the recommendation engine. */
  def getRecommendations(
      profileId: String,
      context: InteractionContext = InteractionContext(),
      limit: Int = 8
  ): PersonalizedRecommendationResult =
    recommendationEngine.hybridRecommendations(profileId, context, Set.empty, limit)


  // ══════════════════════════════════════════════════════════════════════════
  // Private helpers
  // ══════════════════════════════════════════════════════════════════════════

  private def resolvePersona(request: PersonalizationRequest): PersonaLabel =
    request.persona
      .orElse(recommendationEngine.getProfile(request.profileId).map(_.persona))
      .orElse(request.context.get("persona").flatMap(PersonaLabel.fromString))
      .getOrElse(Newcomer)

  private def inferPreferences(
      persona: PersonaLabel,
      context: Map[String, String]
  ): Map[String, UserPreference] = {
    val inferred = mutable.Map[String, UserPreference]()

    def infer(key: String, value: String): Unit =
      inferred(key) = UserPreference(key, value, "inferred")

    persona match {
      case PowerUser =>
        infer("content_density", "compact")
        infer("feed_ordering", "recency")
        infer("analytics_sidebar", "on")
      case Specialist =>
        infer("content_density", "compact")
        infer("feed_ordering", "relevance")
        infer("analytics_sidebar", "on")
      case CasualBrowser | Newcomer =>
        infer("content_density", "spacious")
        infer("feed_ordering", "trending")
        infer("analytics_sidebar", "off")
      case Explorer | EarlyAdopter =>
        infer("content_density", "standard")
        infer("feed_ordering", "recency")
      case ValueSeeker =>
        infer("feed_ordering", "value")
      case Collaborator =>
        infer("feed_ordering", "personal")
      case _ =>
    }

    context.get("device").foreach {
      case "mobile" => infer("content_density", "spacious")
      case "desktop" => /* leave as persona default */
      case _ =>
    }

    inferred.toMap
  }

  private def buildPersonalizationRecommendations(
      persona: PersonaLabel,
      context: Map[String, String],
      prefs: ResolvedPreferences,
      segments: SegmentAssignment
  ): List[PersonalizationRecommendation] = {
    val buffer = mutable.ListBuffer[PersonalizationRecommendation]()

    persona match {
      case Newcomer =>
        buffer += PersonalizationRecommendation("per-onboard-001", "high",
          "Complete your profile to unlock personalized recommendations.")
        buffer += PersonalizationRecommendation("per-onboard-002", "medium",
          "Explore the marketplace to discover portfolio resources and opportunities.")

      case PowerUser =>
        buffer += PersonalizationRecommendation("per-power-001", "medium",
          "Enable advanced analytics sidebar for deeper portfolio insights.")
        buffer += PersonalizationRecommendation("per-power-002", "low",
          "Bulk-export your portfolio data via the developer API.")

      case Explorer =>
        buffer += PersonalizationRecommendation("per-explore-001", "medium",
          "Discover cross-category connections in your portfolio graph.")
        buffer += PersonalizationRecommendation("per-explore-002", "low",
          "Expand into adjacent community spaces aligned with your interests.")

      case CasualBrowser =>
        buffer += PersonalizationRecommendation("per-casual-001", "medium",
          "Simplify your dashboard: pin the top 3 modules you use most.")
        buffer += PersonalizationRecommendation("per-casual-002", "low",
          "Use the 'Quick Start' template to launch your first project in minutes.")

      case ValueSeeker =>
        buffer += PersonalizationRecommendation("per-value-001", "high",
          "Review high-value portfolio items flagged for cashflow improvement.")
        buffer += PersonalizationRecommendation("per-value-002", "medium",
          "Compare your portfolio against community benchmarks to surface gaps.")

      case Collaborator =>
        buffer += PersonalizationRecommendation("per-collab-001", "medium",
          "Invite collaborators to your top active projects.")
        buffer += PersonalizationRecommendation("per-collab-002", "low",
          "Join community spaces aligned with your portfolio categories.")

      case Specialist =>
        buffer += PersonalizationRecommendation("per-spec-001", "medium",
          "Deep-link your specialist index to external APIs for richer data feeds.")
        buffer += PersonalizationRecommendation("per-spec-002", "low",
          "Publish your playbook to the marketplace and monetize your expertise.")

      case EarlyAdopter =>
        buffer += PersonalizationRecommendation("per-early-001", "low",
          "You're among the first to access the new exchange module — share your feedback.")

      case _ =>
    }

    // Segment-based overlay
    segments.primarySegment.foreach {
      case "high-value-investors" =>
        buffer += PersonalizationRecommendation("per-seg-hvi-001", "high",
          "Curated high-yield portfolio assets available on the exchange this week.")
      case "new-creators" =>
        buffer += PersonalizationRecommendation("per-seg-nc-001", "medium",
          "Publish your first item to the marketplace and reach potential buyers.")
      case "active-collaborators" =>
        buffer += PersonalizationRecommendation("per-seg-ac-001", "low",
          "Your collaboration score is high — consider launching a collective project.")
      case _ =>
    }

    buffer.toList.sortBy(r => priorityRank(r.priority)).take(6)
  }

  /** Adaptation score for a single content item. */
  private def adaptationScore(
      persona: PersonaLabel,
      originalRank: Int,
      meta: Map[String, String],
      context: InteractionContext
  ): Double = {
    val rankScore  = 1.0 / (originalRank + 1.0)
    val isNew      = meta.get("recency").contains("new")
    val isHighVal  = meta.get("value").contains("high")
    val isTrending = meta.get("trending").contains("true")

    val boost = persona match {
      case EarlyAdopter  if isNew      => 2.0
      case ValueSeeker   if isHighVal  => 1.8
      case CasualBrowser if isTrending => 1.5
      case Newcomer      if isTrending => 1.4
      case _                           => 1.0
    }

    val timeBoost =
      if (meta.get("timeOfDay").contains(context.timeOfDay)) 1.2 else 1.0

    rankScore * boost * timeBoost
  }

  private def personaContentBoost(
      persona: PersonaLabel,
      itemId: String,
      context: InteractionContext
  ): (Double, ContentOrdering) =
    persona match {
      case PowerUser    => (1.2, RecencyFirst)
      case Specialist   => (1.15, RelevanceFirst)
      case Explorer     => (1.1, RelevanceFirst)
      case EarlyAdopter => (1.2, RecencyFirst)
      case ValueSeeker  => (1.0, ValueFirst)
      case Collaborator => (1.05, PersonalFirst)
      case CasualBrowser | Newcomer => (0.9, TrendingFirst)
      case _            => (1.0, RelevanceFirst)
    }

  private def personaDensity(persona: PersonaLabel): ContentDensity = persona match {
    case PowerUser | Specialist   => Compact
    case CasualBrowser | Newcomer => Spacious
    case _                        => Standard
  }

  private def personaOrdering(persona: PersonaLabel): ContentOrdering = persona match {
    case PowerUser | EarlyAdopter => RecencyFirst
    case Specialist | Explorer    => RelevanceFirst
    case CasualBrowser | Newcomer => TrendingFirst
    case ValueSeeker              => ValueFirst
    case Collaborator             => PersonalFirst
    case _                        => RelevanceFirst
  }

  private def personaFeedLimit(persona: PersonaLabel): Int = persona match {
    case PowerUser | Specialist => 50
    case Explorer | EarlyAdopter => 30
    case CasualBrowser | Newcomer => 15
    case _ => defaultFeedLimit
  }

  /** Select a variant using weighted random sampling. */
  private def weightedRandomVariant(variants: List[ExperimentVariant]): ExperimentVariant = {
    if (variants.isEmpty) return ExperimentVariant("control", "Control")
    val total  = variants.map(_.weight).sum
    val r      = rng.nextDouble() * total
    var cumsum = 0.0
    variants.find { v =>
      cumsum += v.weight
      r <= cumsum
    }.getOrElse(variants.last)
  }

  /** Return active experiments that target the given segments (or all users). */
  private def activeExperimentsFor(
      profileId: String,
      userSegments: List[String]
  ): List[Experiment] =
    experiments.values
      .filter(e => e.active &&
        (e.targetSegments.isEmpty || e.targetSegments.exists(userSegments.contains)))
      .toList

  /**
   * Distance between two personas: 0 = same, 1 = maximally different.
   * Mirrors personaSimilarity logic from MatchEngine.
   */
  private def personaDistance(a: PersonaLabel, b: PersonaLabel): Double = {
    if (a == b) return 0.0
    val groups: List[Set[PersonaLabel]] = List(
      Set(PowerUser, Specialist, EarlyAdopter),
      Set(Explorer, CasualBrowser),
      Set(Collaborator, ValueSeeker),
      Set(Newcomer, UnknownPersona)
    )
    if (groups.exists(g => g.contains(a) && g.contains(b))) 0.4
    else 0.8
  }

  private def priorityRank(p: String): Int = p match {
    case "high" | "red"     => 0
    case "medium" | "amber" => 1
    case _                  => 2
  }
}
