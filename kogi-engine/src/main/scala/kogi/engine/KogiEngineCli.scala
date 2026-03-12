package kogi.engine

import scala.collection.mutable

object KogiEngineCli {
  def main(args: Array[String]): Unit = {
    val parsed = parseArgs(args.toList)
    val action = parsed.options.getOrElse("action", "snapshot").toLowerCase
    val engine = new KogiEngine()

    val response: Map[String, Any] = action match {

      // ----------------------------------------------------------------
      // Core engine control
      // ----------------------------------------------------------------

      case "control" =>
        val mode = parsed.options.getOrElse("mode", parsed.options.getOrElse("value", "start"))
        val status = engine.control(mode)
        controlResponse(status)

      case "status" =>
        controlResponse(engine.status)

      case "ingest" =>
        val topic       = parsed.options.getOrElse("topic", "engine.ingest")
        val source      = parsed.options.getOrElse("source", "kogi.network.engine")
        val target      = parsed.options.getOrElse("target", "kogi.engine")
        val flowId      = parsed.options.getOrElse("flow-id", s"flow-${safeId(topic)}")
        val timestampMs = parsed.options.get("timestamp-ms").flatMap(toLong).getOrElse(System.currentTimeMillis())
        val snap        = engine.ingestGatewayMessage(topic, parsed.payload, source, target, flowId, timestampMs)
        snapshotResponse(snap)

      case "snapshot" =>
        val hostId   = parsed.options.getOrElse("host-id", "kogi-host-001")
        val windowMs = parsed.options.get("window-ms").flatMap(toLong).getOrElse(5L * 60L * 1000L)
        val snap     = engine.snapshot(hostId = hostId, windowMs = windowMs)
        snapshotResponse(snap)

      case "query" =>
        val sql = parsed.options.getOrElse("sql", "")
        planResponse(engine.query(sql))

      case "search" =>
        val queryText = parsed.options.getOrElse("query", "")
        val result    = engine.search(SearchQuery(text = queryText, tags = parsed.tags, metadata = parsed.metadata))
        searchResponse(result)

      case "index" =>
        val id    = parsed.options.getOrElse("id", s"doc-${System.currentTimeMillis()}")
        val title = parsed.options.getOrElse("title", "Untitled")
        val body  = parsed.options.getOrElse("body", parsed.options.getOrElse("content", ""))
        engine.index(SearchDocument(id = id, title = title, body = body, tags = parsed.tags, metadata = parsed.metadata))
        Map("status" -> "ok", "action" -> "index", "id" -> id)

      // ----------------------------------------------------------------
      // Graph – mutation
      //
      //  --from A --to B --edge-type Dependency --weight 1.0
      // ----------------------------------------------------------------

      case "graph-add-edge" =>
        val from     = requireOpt(parsed, "from",     "graph-add-edge --from A --to B [--edge-type Dependency] [--weight 1.0]")
        val to       = requireOpt(parsed, "to",       "graph-add-edge --from A --to B [--edge-type Dependency] [--weight 1.0]")
        val edgeType = parseEdgeType(parsed.options.getOrElse("edge-type", "Dependency"))
        val weight   = parsed.options.get("weight").flatMap(toDouble).getOrElse(1.0)
        engine.graphAddEdge(from, to, edgeType, weight)
        Map("status" -> "ok", "action" -> "graph-add-edge",
            "from" -> from, "to" -> to,
            "edge_type" -> edgeType.toString, "weight" -> weight,
            "total_edges" -> engine.graphEdges.size)

      case "graph-add-node" =>
        val id       = requireOpt(parsed, "id",       "graph-add-node --id A [--duration 3.0]")
        val duration = parsed.options.get("duration").flatMap(toDouble).getOrElse(0.0)
        engine.graphAddNode(id, duration)
        Map("status" -> "ok", "action" -> "graph-add-node",
            "id" -> id, "duration" -> duration,
            "total_nodes" -> engine.graphNodes.size)

      case "graph-remove-edge" =>
        val from = requireOpt(parsed, "from", "graph-remove-edge --from A --to B")
        val to   = requireOpt(parsed, "to",   "graph-remove-edge --from A --to B")
        engine.graphRemoveEdge(from, to)
        Map("status" -> "ok", "action" -> "graph-remove-edge",
            "from" -> from, "to" -> to,
            "total_edges" -> engine.graphEdges.size)

      case "graph-remove-node" =>
        val id = requireOpt(parsed, "id", "graph-remove-node --id A")
        engine.graphRemoveNode(id)
        Map("status" -> "ok", "action" -> "graph-remove-node",
            "id" -> id, "total_nodes" -> engine.graphNodes.size)

      // ----------------------------------------------------------------
      // Graph – traversal queries
      //
      //  --node auth-service
      // ----------------------------------------------------------------

      case "graph-impact" =>
        val nodeId = requireOpt(parsed, "node", "graph-impact --node <id>")
        graphImpactResponse(engine.graphImpact(nodeId))

      case "graph-closure" =>
        val nodeId = requireOpt(parsed, "node", "graph-closure --node <id>")
        val result = engine.graphDependencyClosure(nodeId)
        Map("status" -> "ok", "action" -> "graph-closure",
            "node" -> nodeId, "closure" -> result.toSeq.sorted)

      case "graph-rdeps" =>
        val nodeId = requireOpt(parsed, "node", "graph-rdeps --node <id>")
        val result = engine.graphReverseClosure(nodeId)
        Map("status" -> "ok", "action" -> "graph-rdeps",
            "node" -> nodeId, "reverse_closure" -> result.toSeq.sorted)

      case "graph-neighbors" =>
        val nodeId = requireOpt(parsed, "node", "graph-neighbors --node <id>")
        val result = engine.graphNeighborhood(nodeId)
        Map("status" -> "ok", "action" -> "graph-neighbors",
            "node" -> nodeId, "neighbors" -> result.toSeq.sorted)

      case "graph-ancestors" =>
        val nodeId = requireOpt(parsed, "node", "graph-ancestors --node <id>")
        val result = engine.graphAncestors(nodeId)
        Map("status" -> "ok", "action" -> "graph-ancestors",
            "node" -> nodeId, "ancestors" -> result.toSeq.sorted)

      case "graph-descendants" =>
        val nodeId = requireOpt(parsed, "node", "graph-descendants --node <id>")
        val result = engine.graphDescendants(nodeId)
        Map("status" -> "ok", "action" -> "graph-descendants",
            "node" -> nodeId, "descendants" -> result.toSeq.sorted)

      // ----------------------------------------------------------------
      // Graph – analysis
      // ----------------------------------------------------------------

      case "graph-cycles" =>
        val report = engine.graphCycles()
        graphCyclesResponse(report)

      case "graph-topo" =>
        engine.graphTopologicalSort() match {
          case Right(order) =>
            Map("status" -> "ok", "action" -> "graph-topo",
                "has_cycle" -> false, "order" -> order)
          case Left(cycleNodes) =>
            Map("status" -> "error", "action" -> "graph-topo",
                "has_cycle" -> true, "cycle_nodes" -> cycleNodes.toSeq.sorted,
                "error" -> "topological sort requires a DAG")
        }

      case "graph-critical" =>
        engine.graphCriticalPath() match {
          case Some(report) => graphCriticalResponse(report)
          case None =>
            Map("status" -> "error", "action" -> "graph-critical",
                "error" -> "critical path is undefined: cycle detected in graph")
        }

      // ----------------------------------------------------------------
      // Graph – diff
      //
      //  Takes a snapshot before mutations then diffs:
      //  CLI encodes a "before" edge list in --snapshot-edges k=v,...
      //  For programmatic use, call graphSnapshot() / graphDiff() directly.
      // ----------------------------------------------------------------

      case "graph-diff" =>
        // Build the "before" engine from --before-edge flags, then diff against current
        val beforeEdges = parsed.options
          .collect { case (k, v) if k.startsWith("before-edge-") =>
            parseEdgeTuple(v)
          }
          .flatten.toSeq
        val beforeEngine = new GraphEngine(beforeEdges)
        val diff         = beforeEngine.diff(engine.graphEngine)
        graphDiffResponse(diff)

      // ----------------------------------------------------------------
      // Graph – introspection
      // ----------------------------------------------------------------

      case "graph-nodes" =>
        Map("status" -> "ok", "action" -> "graph-nodes",
            "count" -> engine.graphNodes.size,
            "nodes" -> engine.graphNodes.toSeq.sorted)

      case "graph-edges" =>
        Map("status" -> "ok", "action" -> "graph-edges",
            "count" -> engine.graphEdges.size,
            "edges" -> engine.graphEdges.map { e =>
              Map("from" -> e.from, "to" -> e.to,
                  "edge_type" -> e.edgeType.toString, "weight" -> e.weight)
            })

      case _ =>
        Map("status" -> "error", "error" -> s"unknown action: $action")
    }

    println(JsonPrinter.render(response))
  }

  // ----------------------------------------------------------------
  // Response builders
  // ----------------------------------------------------------------

  private def controlResponse(status: EngineControlStatus): Map[String, Any] =
    Map(
      "engine"       -> "kogi-engine",
      "status"       -> "ok",
      "mode"         -> status.mode,
      "changed"      -> status.changed,
      "timestamp_ms" -> status.timestampMs
    )

  private def snapshotResponse(snapshot: EngineFlowSnapshot): Map[String, Any] =
    Map(
      "engine"           -> "kogi-engine",
      "status"           -> "ok",
      "total_envelopes"  -> snapshot.totalEnvelopes,
      "observed_topics"  -> snapshot.observedTopics,
      "generated_at_ms"  -> snapshot.generatedAtMs,
      "host" -> Map(
        "host_id"        -> snapshot.system.host.hostId,
        "status"         -> snapshot.system.host.status,
        "saturation"     -> snapshot.system.host.saturationScore,
        "cpu_avg_pct"    -> snapshot.system.host.cpuAvgPct,
        "memory_avg_pct" -> snapshot.system.host.memoryAvgPct
      ),
      "recommendations"  -> snapshot.system.recommendations.map(_.message)
    )

  private def planResponse(plan: QueryPlan): Map[String, Any] =
    Map(
      "engine" -> "kogi-engine",
      "status" -> "ok",
      "query" -> Map(
        "statement"       -> plan.analysis.statementType,
        "normalized_sql"  -> plan.analysis.normalizedSql,
        "tables"          -> plan.analysis.tables,
        "warnings"        -> plan.analysis.warnings,
        "estimated_cost"  -> plan.analysis.estimatedCost,
        "optimized_sql"   -> plan.optimizedSql,
        "hints"           -> plan.hints
      ),
      "generated_at_ms" -> plan.generatedAtMs
    )

  private def searchResponse(result: SearchResult): Map[String, Any] =
    Map(
      "engine"  -> "kogi-engine",
      "status"  -> "ok",
      "query"   -> result.query.text,
      "total"   -> result.total,
      "matches" -> result.matches.map { m =>
        Map(
          "id"         -> m.document.id,
          "title"      -> m.document.title,
          "score"      -> m.score,
          "highlights" -> m.highlights
        )
      },
      "generated_at_ms" -> result.generatedAtMs
    )

  private def graphImpactResponse(r: ImpactReport): Map[String, Any] =
    Map(
      "status"     -> "ok",
      "action"     -> "graph-impact",
      "root"       -> r.root,
      "affected"   -> r.affected.toSeq.sorted,    // downstream (what root depends on)
      "dependents" -> r.dependents.toSeq.sorted   // upstream (what depends on root)
    )

  private def graphCyclesResponse(r: CycleReport): Map[String, Any] =
    Map(
      "status"     -> "ok",
      "action"     -> "graph-cycles",
      "has_cycles" -> r.hasCycles,
      "cycle_count"-> r.cycles.size,
      "cycles"     -> r.cycles.map(_.mkString(" → "))
    )

  private def graphCriticalResponse(r: CriticalPathReport): Map[String, Any] =
    Map(
      "status"       -> "ok",
      "action"       -> "graph-critical",
      "path"         -> r.path,
      "total_cost"   -> r.totalCost,
      "node_slack"   -> r.nodeSlack.map { case (k, v) => k -> v }
    )

  private def graphDiffResponse(d: GraphDiff): Map[String, Any] =
    Map(
      "status"        -> "ok",
      "action"        -> "graph-diff",
      "added_nodes"   -> d.addedNodes.toSeq.sorted,
      "removed_nodes" -> d.removedNodes.toSeq.sorted,
      "added_edges"   -> d.addedEdges.map(e => s"${e.from}→${e.to}").toSeq.sorted,
      "removed_edges" -> d.removedEdges.map(e => s"${e.from}→${e.to}").toSeq.sorted,
      "changed_edges" -> d.changedEdges.map { case (o, n) =>
        Map("from" -> o.from, "to" -> o.to,
            "old_type" -> o.edgeType.toString, "new_type" -> n.edgeType.toString,
            "old_weight" -> o.weight, "new_weight" -> n.weight)
      }.toSeq
    )

  // ----------------------------------------------------------------
  // Argument parsing
  // ----------------------------------------------------------------

  private final case class ParsedArgs(
      options:  Map[String, String],
      payload:  Map[String, String],
      tags:     List[String],
      metadata: Map[String, String]
  )

  private def parseArgs(args: List[String]): ParsedArgs = {
    val options  = mutable.Map.empty[String, String]
    val payload  = mutable.Map.empty[String, String]
    val tags     = mutable.ListBuffer.empty[String]
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
      target.update(value.substring(0, idx), value.substring(idx + 1))
    }
  }

  /** Parse "A:B:Dependency:1.5" into a GraphEdge. */
  private def parseEdgeTuple(value: String): Option[GraphEdge] = {
    val parts = value.split(":")
    if (parts.length >= 2) {
      val edgeType = if (parts.length >= 3) parseEdgeType(parts(2)) else Dependency
      val weight   = if (parts.length >= 4) parts(3).toDoubleOption.getOrElse(1.0) else 1.0
      Some(GraphEdge(parts(0), parts(1), edgeType, weight))
    } else None
  }

  private def parseEdgeType(raw: String): EdgeType = raw.trim.toLowerCase match {
    case "hierarchy"    => Hierarchy
    case "relationship" => Relationship
    case _              => Dependency
  }

  private def requireOpt(parsed: ParsedArgs, key: String, usage: String): String =
    parsed.options.getOrElse(key, {
      System.err.println(s"[error] missing --$key  usage: $usage")
      sys.exit(1)
    })

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")

  private def toLong(value: String): Option[Long]     = scala.util.Try(value.toLong).toOption
  private def toDouble(value: String): Option[Double] = scala.util.Try(value.toDouble).toOption
}

object JsonPrinter {
  def render(value: Any): String = value match {
    case null     => "null"
    case s: String  => "\"" + escape(s) + "\""
    case b: Boolean => b.toString
    case i: Int     => i.toString
    case l: Long    => l.toString
    case d: Double  => if (d.isNaN || d.isInfinity) "0.0" else d.toString
    case f: Float   => if (f.isNaN || f.isInfinity) "0.0" else f.toString
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
