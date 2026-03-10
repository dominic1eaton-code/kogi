package kogi.engine

final case class QueryAnalysis(
    sql: String,
    statementType: String,
    normalizedSql: String,
    tables: List[String],
    warnings: List[String],
    estimatedCost: Double
)

final case class QueryPlan(
    analysis: QueryAnalysis,
    optimizedSql: String,
    hints: List[String],
    generatedAtMs: Long
)

final case class QueryProfile(
    userId: String,
    persona: PersonaLabel = Newcomer,
    preferredLimit: Int = 1000,          // persona-driven default row cap
    allowAnalyticHints: Boolean = false  // PowerUser / Specialist get deeper hints
)

final class QueryEngine {
  def analyze(sql: String): QueryAnalysis = {
    val normalized    = normalize(sql)
    val statementType = normalized.split("\\s+").headOption.getOrElse("UNKNOWN").toUpperCase
    val tables        = extractTables(normalized)
    val warnings      = buildWarnings(normalized, statementType)
    val estimatedCost = estimateCost(normalized, tables)

    QueryAnalysis(sql, statementType, normalized, tables, warnings, estimatedCost)
  }

  def optimize(sql: String): QueryPlan = {
    val analysis = analyze(sql)
    val hints    = buildHints(analysis)
    val optimizedSql =
      if (analysis.statementType == "SELECT" && !analysis.normalizedSql.contains("limit"))
        s"${analysis.normalizedSql} LIMIT 1000"
      else analysis.normalizedSql

    QueryPlan(analysis, optimizedSql, hints, generatedAtMs = System.currentTimeMillis())
  }

  /**
   * Profile-aware optimisation: adjusts the auto-LIMIT and hint verbosity
   * based on the user's persona.
   *  - PowerUser / Specialist  → higher row cap, analytic hints enabled
   *  - CasualBrowser           → lower row cap, simplified hints
   *  - Default                 → standard behaviour
   */
  def optimizeWithProfile(sql: String, profile: QueryProfile): QueryPlan = {
    val base      = analyze(sql)
    val rowCap    = profile.persona match {
      case PowerUser   => 10000
      case Specialist  => 5000
      case CasualBrowser => 200
      case _           => profile.preferredLimit
    }
    val capped    =
      if (base.statementType == "SELECT" && !base.normalizedSql.contains("limit"))
        s"${base.normalizedSql} LIMIT $rowCap"
      else base.normalizedSql

    val hints     = buildHints(base) ++ (
      if (profile.allowAnalyticHints || profile.persona == PowerUser || profile.persona == Specialist)
        analyticHints(base) else Nil
    )

    QueryPlan(base, capped, hints, generatedAtMs = System.currentTimeMillis())
  }

  private def analyticHints(analysis: QueryAnalysis): List[String] = {
    val h = scala.collection.mutable.ListBuffer.empty[String]
    if (analysis.tables.size > 2)
      h += "Multiple joins detected; consider materialized views for frequently run reports."
    if (analysis.estimatedCost > 200.0)
      h += "High-cost query; evaluate query result caching or async execution."
    h.toList
  }

  private def normalize(sql: String): String =
    sql.trim.replaceAll("\\s+", " ").toLowerCase

  private def extractTables(normalized: String): List[String] = {
    val tokens = normalized.split("\\s+").toList
    tokens.zipWithIndex.collect {
      case (token, idx) if token == "from" && idx + 1 < tokens.size => tokens(idx + 1)
      case (token, idx) if token == "join" && idx + 1 < tokens.size => tokens(idx + 1)
    }.distinct
  }

  private def buildWarnings(normalized: String, statementType: String): List[String] = {
    val buffer = scala.collection.mutable.ListBuffer.empty[String]
    if (statementType == "SELECT" && normalized.contains("select *"))
      buffer += "SELECT * detected; prefer projecting required columns."
    if (statementType == "SELECT" && !normalized.contains("where"))
      buffer += "SELECT without WHERE clause may scan full table."
    if (statementType == "UPDATE" && !normalized.contains("where"))
      buffer += "UPDATE without WHERE clause will update all rows."
    if (statementType == "DELETE" && !normalized.contains("where"))
      buffer += "DELETE without WHERE clause will delete all rows."
    buffer.toList
  }

  private def estimateCost(normalized: String, tables: List[String]): Double = {
    val base        = normalized.length.toDouble / 10.0
    val tableFactor = tables.size.max(1) * 12.5
    base + tableFactor
  }

  private def buildHints(analysis: QueryAnalysis): List[String] = {
    val buffer = scala.collection.mutable.ListBuffer.empty[String]
    if (analysis.statementType == "SELECT" && analysis.tables.nonEmpty)
      buffer += "Consider indexes on frequently filtered columns."
    if (analysis.estimatedCost >= 120.0)
      buffer += "Query cost is high; review joins and predicates."
    buffer.toList
  }
}
