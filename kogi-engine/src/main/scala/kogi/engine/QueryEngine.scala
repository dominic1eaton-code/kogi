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

final class QueryEngine {
  def analyze(sql: String): QueryAnalysis = {
    val normalized = normalize(sql)
    val statementType = normalized.split("\\s+").headOption.getOrElse("UNKNOWN").toUpperCase
    val tables = extractTables(normalized)
    val warnings = buildWarnings(normalized, statementType)
    val estimatedCost = estimateCost(normalized, tables)

    QueryAnalysis(
      sql = sql,
      statementType = statementType,
      normalizedSql = normalized,
      tables = tables,
      warnings = warnings,
      estimatedCost = estimatedCost
    )
  }

  def optimize(sql: String): QueryPlan = {
    val analysis = analyze(sql)
    val hints = buildHints(analysis)
    val optimizedSql =
      if (analysis.statementType == "SELECT" && !analysis.normalizedSql.contains("limit"))
        s"${analysis.normalizedSql} LIMIT 1000"
      else analysis.normalizedSql

    QueryPlan(
      analysis = analysis,
      optimizedSql = optimizedSql,
      hints = hints,
      generatedAtMs = System.currentTimeMillis()
    )
  }

  private def normalize(sql: String): String =
    sql.trim.replaceAll("\\s+", " ").toLowerCase

  private def extractTables(normalized: String): List[String] = {
    val tokens = normalized.split("\\s+").toList
    val tableTokens = tokens.zipWithIndex.collect {
      case (token, idx) if token == "from" && idx + 1 < tokens.size => tokens(idx + 1)
      case (token, idx) if token == "join" && idx + 1 < tokens.size => tokens(idx + 1)
    }
    tableTokens.distinct
  }

  private def buildWarnings(normalized: String, statementType: String): List[String] = {
    val buffer = scala.collection.mutable.ListBuffer.empty[String]

    if (statementType == "SELECT" && normalized.contains("select *")) {
      buffer += "SELECT * detected; prefer projecting required columns."
    }
    if (statementType == "SELECT" && !normalized.contains("where")) {
      buffer += "SELECT without WHERE clause may scan full table."
    }
    if (statementType == "UPDATE" && !normalized.contains("where")) {
      buffer += "UPDATE without WHERE clause will update all rows."
    }
    if (statementType == "DELETE" && !normalized.contains("where")) {
      buffer += "DELETE without WHERE clause will delete all rows."
    }

    buffer.toList
  }

  private def estimateCost(normalized: String, tables: List[String]): Double = {
    val base = normalized.length.toDouble / 10.0
    val tableFactor = tables.size.max(1) * 12.5
    base + tableFactor
  }

  private def buildHints(analysis: QueryAnalysis): List[String] = {
    val buffer = scala.collection.mutable.ListBuffer.empty[String]
    if (analysis.statementType == "SELECT" && analysis.tables.nonEmpty) {
      buffer += "Consider indexes on frequently filtered columns."
    }
    if (analysis.estimatedCost >= 120.0) {
      buffer += "Query cost is high; review joins and predicates."
    }
    buffer.toList
  }
}
