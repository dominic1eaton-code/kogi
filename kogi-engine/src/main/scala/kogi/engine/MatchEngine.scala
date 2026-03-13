// MatchEngine — matching users (owners, investors, donors, etc.), resources,
// assets, portfolio components, and analytics artifacts (recommendations,
// searches, indexes, filters).
//
// Design goals:
//   • Multi-dimensional scoring: content similarity + persona alignment +
//     signal strength + contextual relevance.
//   • Type-safe match subjects via sealed trait hierarchy.
//   • Pluggable scoring strategies so callers can swap or extend weights.
//   • Persona-aware boosting that honours the labels defined in
//     RecommendationEngine.
//   • First-class support for the platform's portfolio component graph
//     (portfolios, programs, projects, resources, artifacts, assets).
package kogi.engine

import scala.collection.mutable


// ─────────────────────────────────────────────────────────────────────────────
// Subject types  — what can be matched
// ─────────────────────────────────────────────────────────────────────────────

/** Any entity that participates in a match operation. */
sealed trait MatchSubject {
  def id: String
  def tags: List[String]
  def attributes: Map[String, String]
}

// ── Users ─────────────────────────────────────────────────────────────────────

/** Broad user-role discriminant used for matching. */
sealed trait UserRole
case object Owner      extends UserRole
case object Investor   extends UserRole
case object Donor      extends UserRole
case object Subscriber extends UserRole
case object Watcher    extends UserRole
case object Follower   extends UserRole
case object Editor     extends UserRole
case object Contributor extends UserRole
case object AnyUserRole extends UserRole

object UserRole {
  def fromString(s: String): UserRole = s.trim.toLowerCase match {
    case "owner"       => Owner
    case "investor"    => Investor
    case "donor"       => Donor
    case "subscriber"  => Subscriber
    case "watcher"     => Watcher
    case "follower"    => Follower
    case "editor"      => Editor
    case "contributor" => Contributor
    case _             => AnyUserRole
  }
}

/**
 * A platform user as a match subject.
 *
 * @param role        Primary role on this platform context.
 * @param persona     Behavioural persona from RecommendationEngine.
 * @param skills      Free-text skill labels for labour / talent matching.
 * @param interests   Topic/category interest labels.
 * @param budget      Optional budget capacity (for investor / donor matching).
 * @param location    Coarse geo label (country or city) for proximity matching.
 */
final case class UserSubject(
    id: String,
    role: UserRole = AnyUserRole,
    persona: PersonaLabel = Newcomer,
    skills: List[String] = Nil,
    interests: List[String] = Nil,
    budget: Option[Double] = None,
    location: String = "unknown",
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject

// ── Portfolio components ───────────────────────────────────────────────────────

/** Discriminant for portfolio component types (from notes.md / portfolio.rs). */
sealed trait ComponentKind
case object PortfolioKind  extends ComponentKind
case object ProgramKind    extends ComponentKind
case object ProjectKind    extends ComponentKind
case object ResourceKind   extends ComponentKind
case object ArtifactKind   extends ComponentKind
case object AssetKind      extends ComponentKind
case object AnyComponentKind extends ComponentKind

object ComponentKind {
  def fromString(s: String): ComponentKind = s.trim.toLowerCase match {
    case "portfolio" => PortfolioKind
    case "program"   => ProgramKind
    case "project"   => ProjectKind
    case "resource"  => ResourceKind
    case "artifact"  => ArtifactKind
    case "asset"     => AssetKind
    case _           => AnyComponentKind
  }
}

/**
 * A portfolio component (item) as a match subject.
 *
 * @param kind         Fine-grained type (portfolio / program / project / …).
 * @param category     Domain category (e.g. "real-estate", "tech", "health").
 * @param status       Lifecycle status string ("active", "draft", "archived").
 * @param ownerIds     User IDs that own this component.
 * @param valueScore   Estimated value / quality signal (0–100).
 * @param riskScore    Risk signal (0–100; lower is safer).
 */
final case class ComponentSubject(
    id: String,
    kind: ComponentKind = AnyComponentKind,
    category: String = "general",
    status: String = "active",
    ownerIds: List[String] = Nil,
    valueScore: Double = 50.0,
    riskScore: Double = 50.0,
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject

// ── Resources ─────────────────────────────────────────────────────────────────

/**
 * A fungible or discrete resource (capital, labour slot, equipment, etc.).
 *
 * @param resourceType  Free label: "capital", "labour", "equipment", "service".
 * @param capacity      Available quantity / budget.
 * @param unit          Unit of measure: "usd", "hours", "units".
 */
final case class ResourceSubject(
    id: String,
    resourceType: String = "general",
    capacity: Double = 0.0,
    unit: String = "units",
    category: String = "general",
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject

// ── Assets ────────────────────────────────────────────────────────────────────

/**
 * A discrete asset (real estate, financial instrument, physical good, etc.).
 *
 * @param assetClass   "real-estate", "equity", "token", "good", etc.
 * @param marketValue  Current estimated market value.
 * @param liquidity    Liquidity score 0–100 (100 = immediately liquid).
 */
final case class AssetSubject(
    id: String,
    assetClass: String = "general",
    marketValue: Double = 0.0,
    liquidity: Double = 50.0,
    category: String = "general",
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject

// ── Analytics artifacts ────────────────────────────────────────────────────────

/** A search result, recommendation, index entry, or filter output. */
final case class AnalyticsArtifact(
    id: String,
    artifactType: String,   // "recommendation" | "search-result" | "index" | "filter"
    score: Double,
    sourceModule: String,
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject


/** Lightweight profile used for profile-to-resource matching. */
final case class Profile(
    id: String,
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject

/** Lightweight resource used for profile-to-resource matching. */
final case class Resource(
    id: String,
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject

/** Adapter to allow OptimizationPlan to participate in match results. */
final case class OptimizationPlanSubject(
    plan: OptimizationPlan,
    tags: List[String] = Nil,
    attributes: Map[String, String] = Map.empty
) extends MatchSubject {
  def id: String = plan.request.workloadId
}


// ─────────────────────────────────────────────────────────────────────────────
// Match result types
// ─────────────────────────────────────────────────────────────────────────────

/** A single match candidate with its composite score and explanation. */
final case class MatchCandidate[A <: MatchSubject](
    subject: A,
    score: Double,
    reasons: List[String],
    personaBoosted: Boolean = false,
    contextBoosted: Boolean = false
)

/** Result of a match operation, with metadata for observability. */
final case class MatchResult[A <: MatchSubject, B <: MatchSubject](
    query: A,
    candidates: List[MatchCandidate[B]],
    strategy: String,
    totalConsidered: Int,
    generatedAtMs: Long
)

/**
 * Aggregated cross-type match: a portfolio component paired with the
 * users, resources, and assets that are the best fit for it.
 */
final case class ComponentMatchBundle(
    component: ComponentSubject,
    matchedUsers: List[MatchCandidate[UserSubject]],
    matchedResources: List[MatchCandidate[ResourceSubject]],
    matchedAssets: List[MatchCandidate[AssetSubject]],
    generatedAtMs: Long
)

/**
 * Talent / labour match: a skills-based query user paired with candidate
 * users whose skill sets overlap.
 */
final case class TalentMatchResult(
    requester: UserSubject,
    candidates: List[MatchCandidate[UserSubject]],
    generatedAtMs: Long
)

/**
 * Workload–optimisation plan pairing (delegates to OptimizationEngine).
 */
final case class WorkloadMatchResult(
    workloadId: String,
    matchedPlans: List[MatchCandidate[OptimizationPlanSubject]],
    generatedAtMs: Long
)


// ─────────────────────────────────────────────────────────────────────────────
// Scoring strategy
// ─────────────────────────────────────────────────────────────────────────────

/** Weights used by the composite scorer.  Callers can override any field. */
final case class MatchWeights(
    tagOverlap: Double         = 0.30,
    attributeOverlap: Double   = 0.15,
    categoryMatch: Double      = 0.20,
    personaAlignment: Double   = 0.15,
    signalStrength: Double     = 0.10,
    contextBoost: Double       = 0.10
)

object MatchWeights {
  val default: MatchWeights = MatchWeights()

  /** Weights tuned for capital / investment matching. */
  val investment: MatchWeights = MatchWeights(
    tagOverlap = 0.20, attributeOverlap = 0.10, categoryMatch = 0.25,
    personaAlignment = 0.10, signalStrength = 0.25, contextBoost = 0.10
  )

  /** Weights tuned for talent / labour matching. */
  val talent: MatchWeights = MatchWeights(
    tagOverlap = 0.40, attributeOverlap = 0.20, categoryMatch = 0.15,
    personaAlignment = 0.10, signalStrength = 0.05, contextBoost = 0.10
  )

  /** Weights tuned for analytics artifact re-ranking. */
  val analytics: MatchWeights = MatchWeights(
    tagOverlap = 0.25, attributeOverlap = 0.10, categoryMatch = 0.15,
    personaAlignment = 0.20, signalStrength = 0.20, contextBoost = 0.10
  )
}


// ─────────────────────────────────────────────────────────────────────────────
// MatchEngine
// ─────────────────────────────────────────────────────────────────────────────

/**
 * MatchEngine — multi-dimensional, persona-aware matching across users,
 * resources, assets, portfolio components, and analytics artifacts.
 *
 * All heavy recommendation logic lives in [[RecommendationEngine]]; this
 * engine focuses on structural, attribute-level, and role-based matching
 * that complements the collaborative / content-based signals.
 *
 * @param defaultLimit      Default max candidates returned per query.
 * @param minScore          Candidates with score below this are excluded.
 * @param optimizationEngine Used for workload–plan matching.
 */
final class MatchEngine(
    defaultLimit: Int = 10,
    minScore: Double = 0.05,
    optimizationEngine: OptimizationEngine = new OptimizationEngine()
) {

  // ── Registry caches (populated via register* methods) ─────────────────────

  private val userRegistry:      mutable.Map[String, UserSubject]      = mutable.Map.empty
  private val componentRegistry: mutable.Map[String, ComponentSubject] = mutable.Map.empty
  private val resourceRegistry:  mutable.Map[String, ResourceSubject]  = mutable.Map.empty
  private val assetRegistry:     mutable.Map[String, AssetSubject]     = mutable.Map.empty


  // ══════════════════════════════════════════════════════════════════════════
  // 1.  Registry Management
  // ══════════════════════════════════════════════════════════════════════════

  def registerUser(user: UserSubject): Unit      = userRegistry(user.id) = user
  def registerUsers(users: Seq[UserSubject]): Unit = users.foreach(registerUser)

  def registerComponent(c: ComponentSubject): Unit           = componentRegistry(c.id) = c
  def registerComponents(cs: Seq[ComponentSubject]): Unit    = cs.foreach(registerComponent)

  def registerResource(r: ResourceSubject): Unit             = resourceRegistry(r.id) = r
  def registerResources(rs: Seq[ResourceSubject]): Unit      = rs.foreach(registerResource)

  def registerAsset(a: AssetSubject): Unit                   = assetRegistry(a.id) = a
  def registerAssets(as: Seq[AssetSubject]): Unit            = as.foreach(registerAsset)

  def userCount: Int      = userRegistry.size
  def componentCount: Int = componentRegistry.size
  def resourceCount: Int  = resourceRegistry.size
  def assetCount: Int     = assetRegistry.size


  // ══════════════════════════════════════════════════════════════════════════
  // 2.  User ↔ Component  (investors, donors, owners seeking projects, etc.)
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Find portfolio components that best match a user's interests, role,
   * and persona.
   */
  def matchUserToComponents(
      user: UserSubject,
      candidates: Seq[ComponentSubject] = componentRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): MatchResult[UserSubject, ComponentSubject] = {

    val scored = candidates
      .filterNot(c => c.id == user.id)
      .map { component =>
        val (score, reasons, personaBoosted, contextBoosted) =
          scoreUserToComponent(user, component, weights)
        MatchCandidate(component, score, reasons, personaBoosted, contextBoosted)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(user, scored, "user-to-component", candidates.size,
      System.currentTimeMillis())
  }

  /**
   * Inverse: find users who are the best fit for a given component
   * (e.g. find investors for a project).
   */
  def matchComponentToUsers(
      component: ComponentSubject,
      role: UserRole = AnyUserRole,
      candidates: Seq[UserSubject] = userRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): MatchResult[ComponentSubject, UserSubject] = {

    val pool = if (role == AnyUserRole) candidates
               else candidates.filter(_.role == role)

    val scored = pool.map { user =>
        val (score, reasons, pb, cb) = scoreUserToComponent(user, component, weights)
        MatchCandidate(user, score, reasons, pb, cb)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(component, scored, s"component-to-users[${UserRole}]", pool.size,
      System.currentTimeMillis())
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 3.  Asset ↔ Component
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Match assets to portfolio components (e.g. assets available for a project
   * to acquire, or a portfolio to hold).
   */
  def matchAssetsToComponent(
      component: ComponentSubject,
      candidates: Seq[AssetSubject] = assetRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): MatchResult[ComponentSubject, AssetSubject] = {

    val scored = candidates.map { asset =>
        val (score, reasons) = scoreAssetToComponent(asset, component, weights)
        MatchCandidate(asset, score, reasons)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(component, scored, "assets-to-component", candidates.size,
      System.currentTimeMillis())
  }

  /**
   * Match components to a single asset (which portfolios / projects would be
   * interested in this asset?).
   */
  def matchComponentsToAsset(
      asset: AssetSubject,
      candidates: Seq[ComponentSubject] = componentRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): MatchResult[AssetSubject, ComponentSubject] = {

    val scored = candidates.map { component =>
        val (score, reasons) = scoreAssetToComponent(asset, component, weights)
        MatchCandidate(component, score, reasons)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(asset, scored, "component-to-asset", candidates.size,
      System.currentTimeMillis())
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 4.  Resource ↔ Component
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Match available resources to a component's needs (what resources fit
   * this project / program?).
   */
  def matchResourcesToComponent(
      component: ComponentSubject,
      candidates: Seq[ResourceSubject] = resourceRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): MatchResult[ComponentSubject, ResourceSubject] = {

    val scored = candidates.map { resource =>
        val (score, reasons) = scoreResourceToComponent(resource, component, weights)
        MatchCandidate(resource, score, reasons)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(component, scored, "resources-to-component", candidates.size,
      System.currentTimeMillis())
  }

  /**
   * Inverse: which components need this resource most?
   */
  def matchComponentsToResource(
      resource: ResourceSubject,
      candidates: Seq[ComponentSubject] = componentRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): MatchResult[ResourceSubject, ComponentSubject] = {

    val scored = candidates.map { component =>
        val (score, reasons) = scoreResourceToComponent(resource, component, weights)
        MatchCandidate(component, score, reasons)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(resource, scored, "resource-to-components", candidates.size,
      System.currentTimeMillis())
  }


  // ══════════════════════════════════════════════════════════════════════════

  // ------------------------------------------------------------------
  // 4b. Profile ? Resource (lightweight matching)
  // ------------------------------------------------------------------

  def matchProfilesToResources(
      profiles: Seq[Profile],
      resources: Seq[Resource],
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): Map[Profile, Seq[Resource]] = {

    profiles.map { profile =>
      val scored = resources.map { resource =>
          val (score, reasons) = scoreProfileToResource(profile, resource, weights)
          MatchCandidate(resource, score, reasons)
        }
        .filter(_.score >= minScore)
        .sortBy(-_.score)
        .take(limit)
        .map(_.subject)
      profile -> scored
    }.toMap
  }


  // 5.  User ↔ User  (talent, skills, persona-based grouping)
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Talent / skills match: find users whose skill set overlaps with the
   * requester's needed skills (expressed in the requester's interests list).
   */
  def matchTalent(
      requester: UserSubject,
      candidates: Seq[UserSubject] = userRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.talent,
      limit: Int = defaultLimit
  ): TalentMatchResult = {

    val scored = candidates
      .filterNot(_.id == requester.id)
      .map { candidate =>
        val (score, reasons, pb, cb) = scoreUserToUser(requester, candidate, weights)
        MatchCandidate(candidate, score, reasons, pb, cb)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    TalentMatchResult(requester, scored, System.currentTimeMillis())
  }

  /**
   * Persona grouping: find users who share the same persona (for cohort
   * analysis, A/B targeting, community building, etc.).
   */
  def matchByPersona(
      persona: PersonaLabel,
      candidates: Seq[UserSubject] = userRegistry.values.toSeq,
      limit: Int = defaultLimit
  ): List[UserSubject] =
    candidates
      .filter(_.persona == persona)
      .take(limit)
      .toList

  /**
   * Persona-similarity match: score candidates by how close their persona
   * is to the query user's persona (useful for community recommendations).
   */
  def matchSimilarUsers(
      user: UserSubject,
      candidates: Seq[UserSubject] = userRegistry.values.toSeq,
      weights: MatchWeights = MatchWeights.default,
      limit: Int = defaultLimit
  ): MatchResult[UserSubject, UserSubject] = {

    val scored = candidates
      .filterNot(_.id == user.id)
      .map { candidate =>
        val (score, reasons, pb, cb) = scoreUserToUser(user, candidate, weights)
        MatchCandidate(candidate, score, reasons, pb, cb)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(user, scored, "user-similarity", candidates.size,
      System.currentTimeMillis())
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 6.  Workload ↔ OptimizationPlan
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Match a set of optimization requests to their best plans, ranked by plan
   * score and relevance.
   */
  def matchWorkloadsToPlans(
      requests: Seq[OptimizationRequest],
      limit: Int = defaultLimit
  ): List[WorkloadMatchResult] =
    requests.map { req =>
      val plan = optimizationEngine.optimize(req)
      val subject = OptimizationPlanSubject(
        plan = plan,
        tags = plan.request.objectives,
        attributes = Map("score" -> f"${plan.score}%.2f")
      )
      val candidate = MatchCandidate(
        subject = subject,
        score   = plan.score / 100.0,
        reasons = plan.recommendations.take(2).map(_.message)
      )
      WorkloadMatchResult(req.workloadId, List(candidate), System.currentTimeMillis())
    }.toList


  // ══════════════════════════════════════════════════════════════════════════
  // 7.  Analytics Artifact Re-Ranking
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Re-rank analytics artifacts (search results, recommendations, index items,
   * filter outputs) for a given user using persona + tag alignment.
   */
  def matchArtifactsToUser(
      user: UserSubject,
      artifacts: Seq[AnalyticsArtifact],
      weights: MatchWeights = MatchWeights.analytics,
      limit: Int = defaultLimit
  ): MatchResult[UserSubject, AnalyticsArtifact] = {

    val scored = artifacts.map { artifact =>
        val tagSim     = jaccardSimilarity(user.tags ++ user.interests, artifact.tags)
        val attrSim    = attributeOverlap(user.attributes, artifact.attributes)
        val personaBst = personaBoostForArtifact(user.persona, artifact)

        val raw = tagSim     * weights.tagOverlap +
                  attrSim    * weights.attributeOverlap +
                  (artifact.score / 100.0) * weights.signalStrength

        val boosted = raw * (1.0 + personaBst * weights.personaAlignment)

        val reasons = mutable.ListBuffer[String]()
        if (tagSim > 0.3)    reasons += f"tag overlap ${tagSim * 100.0}%.0f%%"
        if (personaBst > 0)  reasons += s"aligned with ${PersonaLabel.label(user.persona)} persona"
        reasons += s"artifact score ${artifact.score.toInt}"

        MatchCandidate(artifact, clamp(boosted), reasons.toList,
          personaBoosted = personaBst > 0)
      }
      .filter(_.score >= minScore)
      .sortBy(-_.score)
      .take(limit)
      .toList

    MatchResult(user, scored, "artifacts-to-user", artifacts.size,
      System.currentTimeMillis())
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 8.  Compound Match: Component Bundle
  //     (users + resources + assets all matched to one component)
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Produce a full match bundle for a portfolio component: best-fit users
   * (by role), best-fit resources, and best-fit assets in a single call.
   */
  def componentBundle(
      component: ComponentSubject,
      role: UserRole = AnyUserRole,
      userCandidates: Seq[UserSubject]         = userRegistry.values.toSeq,
      resourceCandidates: Seq[ResourceSubject] = resourceRegistry.values.toSeq,
      assetCandidates: Seq[AssetSubject]       = assetRegistry.values.toSeq,
      limit: Int = 5
  ): ComponentMatchBundle = {

    val users     = matchComponentToUsers(component, role, userCandidates,
                      MatchWeights.default, limit).candidates
    val resources = matchResourcesToComponent(component, resourceCandidates,
                      MatchWeights.default, limit).candidates
    val assets    = matchAssetsToComponent(component, assetCandidates,
                      MatchWeights.default, limit).candidates

    ComponentMatchBundle(component, users, resources, assets,
      System.currentTimeMillis())
  }


  // ══════════════════════════════════════════════════════════════════════════
  // 9.  Marketplace Exchange Matching
  //     (items listed on the exchange matched to interested buyers/investors)
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Match exchange-listed components to users who have expressed interest
   * in their category / asset class (via attributes["exchange_interest"]).
   */
  def matchExchangeListings(
      listings: Seq[ComponentSubject],
      buyers: Seq[UserSubject],
      limit: Int = defaultLimit
  ): Map[String, List[MatchCandidate[UserSubject]]] =
    listings.map { listing =>
      val result = matchComponentToUsers(listing, AnyUserRole, buyers,
        MatchWeights.investment, limit)
      listing.id -> result.candidates
    }.toMap


  // ══════════════════════════════════════════════════════════════════════════
  // Private: Composite Scorers
  // ══════════════════════════════════════════════════════════════════════════

  /**
   * Score a user ↔ component pair.
   * Returns (score, reasons, personaBoosted, contextBoosted).
   */
  private def scoreUserToComponent(
      user: UserSubject,
      component: ComponentSubject,
      weights: MatchWeights
  ): (Double, List[String], Boolean, Boolean) = {

    val reasons = mutable.ListBuffer[String]()

    // Tag overlap between user interests and component tags
    val allUserTags = (user.tags ++ user.interests ++ user.skills).distinct
    val tagSim      = jaccardSimilarity(allUserTags, component.tags)
    if (tagSim > 0.2) reasons += f"tag alignment ${tagSim * 100.0}%.0f%%"

    // Attribute overlap (shared key-value metadata)
    val attrSim = attributeOverlap(user.attributes, component.attributes)
    if (attrSim > 0.1) reasons += "metadata match"

    // Category alignment (user interest → component category)
    val catScore =
      if (user.interests.contains(component.category) ||
          user.tags.contains(component.category)) 1.0
      else if (fuzzyContains(user.interests ++ user.tags, component.category)) 0.5
      else 0.0
    if (catScore > 0) reasons += s"category match: ${component.category}"

    // Role → component kind alignment
    val roleKindScore = roleToKindScore(user.role, component.kind)
    if (roleKindScore > 0) reasons += s"role ${user.role} fits ${component.kind}"

    // Persona alignment
    val (personaScore, personaBoosted) = personaComponentScore(user.persona, component)
    if (personaBoosted) reasons += s"${PersonaLabel.label(user.persona)} persona boost"

    // Budget / value signal (investors)
    val budgetScore = user.budget match {
      case Some(b) if b >= component.valueScore => 0.8
      case Some(b) if b >= component.valueScore * 0.5 => 0.4
      case Some(_) => 0.1
      case None    => 0.5   // neutral
    }

    // Signal strength: component health (low risk = higher signal)
    val signalScore = clamp((100.0 - component.riskScore) / 100.0)

    val raw =
      tagSim         * weights.tagOverlap +
      attrSim        * weights.attributeOverlap +
      catScore       * weights.categoryMatch +
      roleKindScore  * 0.10 +
      personaScore   * weights.personaAlignment +
      budgetScore    * 0.05 +
      signalScore    * weights.signalStrength

    val contextBoosted = false   // location/time context can be added by callers

    (clamp(raw), reasons.toList, personaBoosted, contextBoosted)
  }

  /** Score a user ↔ user pair (talent / community matching). */
  private def scoreUserToUser(
      query: UserSubject,
      candidate: UserSubject,
      weights: MatchWeights
  ): (Double, List[String], Boolean, Boolean) = {

    val reasons = mutable.ListBuffer[String]()

    val skillSim    = jaccardSimilarity(query.interests ++ query.skills,
                        candidate.skills ++ candidate.interests)
    if (skillSim > 0.2) reasons += f"skill overlap ${skillSim * 100.0}%.0f%%"

    val tagSim      = jaccardSimilarity(query.tags, candidate.tags)
    if (tagSim > 0.2) reasons += "shared tags"

    val attrSim     = attributeOverlap(query.attributes, candidate.attributes)

    val personaScore =
      if (query.persona == candidate.persona) 1.0
      else personaSimilarity(query.persona, candidate.persona)
    val personaBoosted = personaScore > 0.5
    if (personaBoosted) reasons += s"similar persona: ${PersonaLabel.label(candidate.persona)}"

    val locationScore =
      if (query.location != "unknown" && query.location == candidate.location) 0.8
      else 0.0
    if (locationScore > 0) reasons += s"same location: ${candidate.location}"

    val roleScore =
      if (query.role == candidate.role) 0.5
      else if (complementaryRoles(query.role, candidate.role)) 1.0
      else 0.1

    val raw =
      skillSim     * weights.tagOverlap +
      tagSim       * weights.attributeOverlap +
      attrSim      * weights.attributeOverlap * 0.5 +
      personaScore * weights.personaAlignment +
      locationScore * weights.contextBoost +
      roleScore    * weights.signalStrength

    (clamp(raw), reasons.toList, personaBoosted, locationScore > 0)
  }

  /** Score a resource ↔ component pair. */
  private def scoreResourceToComponent(
      resource: ResourceSubject,
      component: ComponentSubject,
      weights: MatchWeights
  ): (Double, List[String]) = {

    val reasons = mutable.ListBuffer[String]()

    val tagSim = jaccardSimilarity(resource.tags, component.tags)
    if (tagSim > 0.2) reasons += f"tag overlap ${tagSim * 100.0}%.0f%%"

    val catScore =
      if (resource.category == component.category) 1.0
      else if (fuzzyContains(List(resource.category), component.category)) 0.5
      else 0.0
    if (catScore > 0) reasons += s"category match: ${component.category}"

    val capacityScore = clamp(resource.capacity / 100.0)
    val attrSim       = attributeOverlap(resource.attributes, component.attributes)

    val raw =
      tagSim        * weights.tagOverlap +
      catScore      * weights.categoryMatch +
      capacityScore * weights.signalStrength +
      attrSim       * weights.attributeOverlap

    if (raw >= minScore) reasons += s"resource type: ${resource.resourceType}"
    (clamp(raw), reasons.toList)
  }

  /** Score an asset ↔ component pair. */

  /** Score a profile ? resource pair (lightweight matching). */
  private def scoreProfileToResource(
      profile: Profile,
      resource: Resource,
      weights: MatchWeights
  ): (Double, List[String]) = {

    val reasons = mutable.ListBuffer[String]()

    val tagSim = jaccardSimilarity(profile.tags, resource.tags)
    if (tagSim > 0.2) reasons += f"tag overlap ${tagSim * 100.0}%.0f%%"

    val attrSim = attributeOverlap(profile.attributes, resource.attributes)
    if (attrSim > 0.1) reasons += "metadata match"

    val raw = tagSim * weights.tagOverlap + attrSim * weights.attributeOverlap
    (clamp(raw), reasons.toList)
  }

  /** Score an asset ??? component pair. */
  private def scoreAssetToComponent(
      asset: AssetSubject,
      component: ComponentSubject,
      weights: MatchWeights
  ): (Double, List[String]) = {

    val reasons = mutable.ListBuffer[String]()

    val tagSim = jaccardSimilarity(asset.tags, component.tags)
    if (tagSim > 0.2) reasons += f"tag overlap ${tagSim * 100.0}%.0f%%"

    val catScore =
      if (asset.category == component.category || asset.assetClass == component.category) 1.0
      else if (fuzzyContains(List(asset.category, asset.assetClass), component.category)) 0.5
      else 0.0
    if (catScore > 0) reasons += s"asset class matches ${component.category}"

    val valueScore    = clamp(asset.marketValue / 100000.0)  // normalise to [0,1]
    val liquidityScore = asset.liquidity / 100.0
    val attrSim        = attributeOverlap(asset.attributes, component.attributes)

    val raw =
      tagSim         * weights.tagOverlap +
      catScore       * weights.categoryMatch +
      valueScore     * weights.signalStrength * 0.5 +
      liquidityScore * weights.signalStrength * 0.5 +
      attrSim        * weights.attributeOverlap

    if (raw >= minScore) reasons += s"asset class: ${asset.assetClass}"
    (clamp(raw), reasons.toList)
  }


  // ══════════════════════════════════════════════════════════════════════════
  // Private: Persona helpers
  // ══════════════════════════════════════════════════════════════════════════

  /** Returns (score, boosted) for a persona ↔ component combination. */
  private def personaComponentScore(
      persona: PersonaLabel,
      component: ComponentSubject
  ): (Double, Boolean) = persona match {
    case PowerUser if component.riskScore < 40.0 =>
      (1.0, true)   // investors prefer low-risk assets
    case Explorer =>
      (0.8, true)   // explorers engage with diverse component types
    case Specialist if component.category == component.attributes.getOrElse("specialty", "") =>
      (1.0, true)
    case ValueSeeker if component.valueScore > 70.0 =>
      (0.9, true)
    case Collaborator if component.attributes.get("collaborative").contains("true") =>
      (0.9, true)
    case EarlyAdopter if component.status == "draft" || component.status == "experimental" =>
      (0.85, true)
    case CasualBrowser =>
      (0.3, false)   // casual browsers have low intent signal
    case Newcomer =>
      (0.4, false)
    case _ =>
      (0.5, false)
  }

  private def personaBoostForArtifact(persona: PersonaLabel, artifact: AnalyticsArtifact): Double =
    persona match {
      case PowerUser   if artifact.tags.contains("advanced")     => 0.3
      case Explorer    if artifact.tags.contains("discovery")    => 0.3
      case Specialist  if artifact.tags.contains("deep-dive")    => 0.3
      case ValueSeeker if artifact.tags.contains("high-value")   => 0.25
      case Collaborator if artifact.tags.contains("shared")      => 0.2
      case _                                                      => 0.0
    }

  /**
   * Rough numeric similarity between two personas (0 = unrelated, 1 = same).
   * Used for community / user–user matching.
   */
  private def personaSimilarity(a: PersonaLabel, b: PersonaLabel): Double = {
    val groups: List[Set[PersonaLabel]] = List(
      Set(PowerUser, Specialist, EarlyAdopter),
      Set(Explorer, CasualBrowser),
      Set(Collaborator, ValueSeeker),
      Set(Newcomer, UnknownPersona)
    )
    if (a == b) 1.0
    else if (groups.exists(g => g.contains(a) && g.contains(b))) 0.6
    else 0.2
  }

  /** True if the two roles are naturally complementary (buyer/seller, etc.). */
  private def complementaryRoles(a: UserRole, b: UserRole): Boolean =
    (a, b) match {
      case (Investor, Owner) | (Owner, Investor)     => true
      case (Donor, Owner)    | (Owner, Donor)        => true
      case (Contributor, Owner) | (Owner, Contributor) => true
      case (Subscriber, Editor) | (Editor, Subscriber) => true
      case _                                          => false
    }

  /**
   * Role → ComponentKind affinity score.
   * e.g. an Investor best matches AssetKind / PortfolioKind.
   */
  private def roleToKindScore(role: UserRole, kind: ComponentKind): Double =
    (role, kind) match {
      case (Investor, AssetKind)    => 1.0
      case (Investor, PortfolioKind) => 0.9
      case (Donor, ProjectKind)     => 1.0
      case (Donor, ProgramKind)     => 0.8
      case (Owner, PortfolioKind)   => 1.0
      case (Owner, ProgramKind)     => 0.9
      case (Owner, ProjectKind)     => 0.9
      case (Editor, ProjectKind)    => 0.8
      case (Editor, ArtifactKind)   => 0.9
      case (Contributor, ProjectKind) => 0.8
      case (Contributor, ResourceKind) => 0.9
      case (Watcher | Follower | Subscriber, _) => 0.3
      case (AnyUserRole, _)         => 0.5
      case _                        => 0.3
    }


  // ══════════════════════════════════════════════════════════════════════════
  // Private: General similarity utilities
  // ══════════════════════════════════════════════════════════════════════════

  private def jaccardSimilarity(a: Seq[String], b: Seq[String]): Double = {
    val setA = a.map(_.toLowerCase).toSet
    val setB = b.map(_.toLowerCase).toSet
    if (setA.isEmpty && setB.isEmpty) return 0.0
    val inter = (setA intersect setB).size.toDouble
    val union = (setA union setB).size.toDouble
    if (union == 0.0) 0.0 else inter / union
  }

  private def attributeOverlap(a: Map[String, String], b: Map[String, String]): Double = {
    if (a.isEmpty || b.isEmpty) return 0.0
    val shared = a.count { case (k, v) => b.get(k).contains(v) }
    shared.toDouble / math.max(a.size, b.size).toDouble
  }

  private def fuzzyContains(haystack: Seq[String], needle: String): Boolean =
    haystack.exists(h =>
      h.toLowerCase.contains(needle.toLowerCase) ||
      needle.toLowerCase.contains(h.toLowerCase)
    )

  private def clamp(v: Double): Double = math.max(0.0, math.min(1.0, v))
}
