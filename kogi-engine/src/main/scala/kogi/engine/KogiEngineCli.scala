package kogi.engine

import scala.collection.mutable

object KogiEngineCli {
  def main(args: Array[String]): Unit = {
    val parsed = parseArgs(args.toList)
    val action = parsed.options.getOrElse("action", "snapshot").toLowerCase
    val engine = new KogiEngine()

    val response: Map[String, Any] = action match {
      case "control" =>
        val mode = parsed.options.getOrElse("mode", parsed.options.getOrElse("value", "start"))
        val status = engine.control(mode)
        controlResponse(status)
      case "status" =>
        controlResponse(engine.status)
      case "ingest" =>
        val topic = parsed.options.getOrElse("topic", "engine.ingest")
        val source = parsed.options.getOrElse("source", "kogi.services.engine")
        val target = parsed.options.getOrElse("target", "kogi.engine")
        val flowId = parsed.options.getOrElse("flow-id", s"flow-${safeId(topic)}")
        val timestampMs = parsed.options.get("timestamp-ms").flatMap(toLong).getOrElse(System.currentTimeMillis())
        val snapshot = engine.ingestGatewayMessage(topic, parsed.payload, source, target, flowId, timestampMs)
        snapshotResponse(snapshot)
      case "snapshot" =>
        val hostId = parsed.options.getOrElse("host-id", "kogi-host-001")
        val windowMs = parsed.options.get("window-ms").flatMap(toLong).getOrElse(5L * 60L * 1000L)
        val snapshot = engine.snapshot(hostId = hostId, windowMs = windowMs)
        snapshotResponse(snapshot)
      case "query" =>
        val sql = parsed.options.getOrElse("sql", "")
        planResponse(engine.query(sql))
      case "search" =>
        val queryText = parsed.options.getOrElse("query", "")
        val result = engine.search(SearchQuery(text = queryText, tags = parsed.tags, metadata = parsed.metadata))
        searchResponse(result)
      case "index" =>
        val id = parsed.options.getOrElse("id", s"doc-${System.currentTimeMillis()}")
        val title = parsed.options.getOrElse("title", "Untitled")
        val body = parsed.options.getOrElse("body", parsed.options.getOrElse("content", ""))
        engine.index(SearchDocument(id = id, title = title, body = body, tags = parsed.tags, metadata = parsed.metadata))
        Map("status" -> "ok", "action" -> "index", "id" -> id)
      case _ =>
        Map("status" -> "error", "error" -> s"unknown action: $action")
    }

    println(JsonPrinter.render(response))
  }

  private def controlResponse(status: EngineControlStatus): Map[String, Any] =
    Map(
      "engine" -> "kogi-engine",
      "status" -> "ok",
      "mode" -> status.mode,
      "changed" -> status.changed,
      "timestamp_ms" -> status.timestampMs
    )

  private def snapshotResponse(snapshot: EngineFlowSnapshot): Map[String, Any] =
    Map(
      "engine" -> "kogi-engine",
      "status" -> "ok",
      "total_envelopes" -> snapshot.totalEnvelopes,
      "observed_topics" -> snapshot.observedTopics,
      "generated_at_ms" -> snapshot.generatedAtMs,
      "host" -> Map(
        "host_id" -> snapshot.system.host.hostId,
        "status" -> snapshot.system.host.status,
        "saturation" -> snapshot.system.host.saturationScore,
        "cpu_avg_pct" -> snapshot.system.host.cpuAvgPct,
        "memory_avg_pct" -> snapshot.system.host.memoryAvgPct
      ),
      "recommendations" -> snapshot.system.recommendations.map(_.message)
    )

  private def planResponse(plan: QueryPlan): Map[String, Any] =
    Map(
      "engine" -> "kogi-engine",
      "status" -> "ok",
      "query" -> Map(
        "statement" -> plan.analysis.statementType,
        "normalized_sql" -> plan.analysis.normalizedSql,
        "tables" -> plan.analysis.tables,
        "warnings" -> plan.analysis.warnings,
        "estimated_cost" -> plan.analysis.estimatedCost,
        "optimized_sql" -> plan.optimizedSql,
        "hints" -> plan.hints
      ),
      "generated_at_ms" -> plan.generatedAtMs
    )

  private def searchResponse(result: SearchResult): Map[String, Any] =
    Map(
      "engine" -> "kogi-engine",
      "status" -> "ok",
      "query" -> result.query.text,
      "total" -> result.total,
      "matches" -> result.matches.map { m =>
        Map(
          "id" -> m.document.id,
          "title" -> m.document.title,
          "score" -> m.score,
          "highlights" -> m.highlights
        )
      },
      "generated_at_ms" -> result.generatedAtMs
    )

  private final case class ParsedArgs(
      options: Map[String, String],
      payload: Map[String, String],
      tags: List[String],
      metadata: Map[String, String]
  )

  private def parseArgs(args: List[String]): ParsedArgs = {
    val options = mutable.Map.empty[String, String]
    val payload = mutable.Map.empty[String, String]
    val tags = mutable.ListBuffer.empty[String]
    val metadata = mutable.Map.empty[String, String]

    var i = 0
    while (i < args.length) {
      args(i) match {
        case "--payload" if i + 1 < args.length =>
          parseKeyValue(args(i + 1), payload)
          i += 2
        case "--tag" if i + 1 < args.length =>
          tags += args(i + 1)
          i += 2
        case "--meta" if i + 1 < args.length =>
          parseKeyValue(args(i + 1), metadata)
          i += 2
        case opt if opt.startsWith("--") && i + 1 < args.length =>
          options += opt.drop(2) -> args(i + 1)
          i += 2
        case _ =>
          i += 1
      }
    }

    ParsedArgs(options.toMap, payload.toMap, tags.toList, metadata.toMap)
  }

  private def parseKeyValue(value: String, target: mutable.Map[String, String]): Unit = {
    val idx = value.indexOf('=')
    if (idx > 0 && idx < value.length - 1) {
      val key = value.substring(0, idx)
      val data = value.substring(idx + 1)
      target.update(key, data)
    }
  }

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")

  private def toLong(value: String): Option[Long] =
    scala.util.Try(value.toLong).toOption
}

object JsonPrinter {
  def render(value: Any): String = value match {
    case null => "null"
    case s: String => "\"" + escape(s) + "\""
    case b: Boolean => b.toString
    case i: Int => i.toString
    case l: Long => l.toString
    case d: Double =>
      if (d.isNaN || d.isInfinity) "0.0" else d.toString
    case f: Float =>
      if (f.isNaN || f.isInfinity) "0.0" else f.toString
    case m: Map[_, _] =>
      val fields = m.map { case (k, v) => render(k.toString) + ":" + render(v) }
      "{" + fields.mkString(",") + "}"
    case seq: Iterable[_] =>
      "[" + seq.map(render).mkString(",") + "]"
    case other => "\"" + escape(other.toString) + "\""
  }

  private def escape(value: String): String =
    value
      .replace("\\", "\\\\")
      .replace("\"", "\\\"")
      .replace("\n", "\\n")
      .replace("\r", "\\r")
      .replace("\t", "\\t")
}
