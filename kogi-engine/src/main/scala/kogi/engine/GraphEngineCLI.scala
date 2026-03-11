package kogi.engine

import scala.util.{Try, Success, Failure}
import java.io.{File, PrintWriter}
import scala.io.Source

// =============================================================
//  GraphEngine CLI
//
//  Run commands directly from a terminal or shell script.
//
//  Usage:
//    scala GraphEngineCLI.scala <command> [options]
//
//  Commands:
//    impact       <nodeId>           – impact analysis for a node
//    closure      <nodeId>           – forward dependency closure
//    rdeps        <nodeId>           – reverse dependency closure
//    topo                            – topological sort
//    critical                        – critical path
//    cycles                          – cycle detection
//    neighbors    <nodeId>           – full neighborhood
//    ancestors    <nodeId>           – hierarchy ancestors
//    descendants  <nodeId>           – hierarchy descendants
//    diff         <snapshotFile>     – diff current graph vs snapshot
//    load         <graphFile>        – load graph from CSV
//    export       <outFile>          – export graph as DOT / CSV
//    help                            – print usage
//
//  Graph CSV format (edges.csv):
//    from,to,edgeType,weight
//    A,B,Dependency,1.0
//    A,C,Hierarchy,1.0
//
//  Node CSV format (nodes.csv):
//    id,duration
//    A,3.0
// =============================================================

object GraphEngineCLI extends App {

  // ----------------------------------------------------------------
  // Entry point – dispatch on first argument
  // ----------------------------------------------------------------

  if (args.isEmpty) {
    printHelp()
    sys.exit(0)
  }

  val (command, rest) = (args.head.toLowerCase, args.tail)

  // Default graph loaded from env or flag --graph / --nodes
  val graphFile = flagValue(rest, "--graph").getOrElse("edges.csv")
  val nodesFile = flagValue(rest, "--nodes").getOrElse("nodes.csv")
  val format    = flagValue(rest, "--format").getOrElse("text")   // text | json | dot

  val engine = loadEngine(graphFile, nodesFile) match {
    case Success(e) => e
    case Failure(ex) =>
      err(s"Failed to load graph: ${ex.getMessage}")
      sys.exit(1)
  }

  command match {
    case "impact"      => cmdImpact(engine, rest, format)
    case "closure"     => cmdClosure(engine, rest, format)
    case "rdeps"       => cmdRdeps(engine, rest, format)
    case "topo"        => cmdTopo(engine, format)
    case "critical"    => cmdCritical(engine, format)
    case "cycles"      => cmdCycles(engine, format)
    case "neighbors"   => cmdNeighbors(engine, rest, format)
    case "ancestors"   => cmdAncestors(engine, rest, format)
    case "descendants" => cmdDescendants(engine, rest, format)
    case "diff"        => cmdDiff(engine, rest, format)
    case "export"      => cmdExport(engine, rest)
    case "nodes"       => cmdNodes(engine, format)
    case "edges"       => cmdEdges(engine, format)
    case "help" | "--help" | "-h" => printHelp()
    case other =>
      err(s"Unknown command: '$other'")
      printHelp()
      sys.exit(1)
  }


  // ----------------------------------------------------------------
  // Command Implementations
  // ----------------------------------------------------------------

  def cmdImpact(engine: GraphEngine, args: Array[String], fmt: String): Unit = {
    val nodeId = requireArg(args, 0, "impact <nodeId>")
    val report = engine.impactAnalysis(nodeId)
    fmt match {
      case "json" =>
        println(s"""{
  "root": "${report.root}",
  "affected": [${report.affected.map(q).mkString(", ")}],
  "dependents": [${report.dependents.map(q).mkString(", ")}]
}""")
      case _ =>
        section(s"Impact Analysis: $nodeId")
        field("Downstream (what this affects)", report.affected)
        field("Upstream (what depends on this)", report.dependents)
    }
  }

  def cmdClosure(engine: GraphEngine, args: Array[String], fmt: String): Unit = {
    val nodeId = requireArg(args, 0, "closure <nodeId>")
    val result = engine.dependencyClosure(nodeId)
    fmt match {
      case "json" => println(jsonSet("dependency_closure", result))
      case _      =>
        section(s"Dependency Closure: $nodeId")
        field("Transitive dependencies", result)
    }
  }

  def cmdRdeps(engine: GraphEngine, args: Array[String], fmt: String): Unit = {
    val nodeId = requireArg(args, 0, "rdeps <nodeId>")
    val result = engine.reverseDependencyClosure(nodeId)
    fmt match {
      case "json" => println(jsonSet("reverse_closure", result))
      case _ =>
        section(s"Reverse Dependencies: $nodeId")
        field("Nodes that depend on this (transitively)", result)
    }
  }

  def cmdTopo(engine: GraphEngine, fmt: String): Unit = {
    engine.topologicalSort() match {
      case Right(order) =>
        fmt match {
          case "json" =>
            println(s"""{"topological_order": [${order.map(q).mkString(", ")}]}""")
          case _ =>
            section("Topological Order")
            order.zipWithIndex.foreach { case (n, i) => println(s"  ${i + 1}. $n") }
        }
      case Left(cycles) =>
        fmt match {
          case "json" =>
            println(s"""{"error": "cycle_detected", "nodes": [${cycles.map(q).mkString(", ")}]}""")
          case _ =>
            err("Cycle detected – cannot produce topological order")
            field("Nodes in cycle", cycles)
        }
        sys.exit(2)
    }
  }

  def cmdCritical(engine: GraphEngine, fmt: String): Unit = {
    engine.criticalPath() match {
      case Some(report) =>
        fmt match {
          case "json" =>
            val slackJson = report.nodeSlack
              .map { case (k, v) => s"""  ${q(k)}: $v""" }.mkString(",\n")
            println(s"""{
  "path": [${report.path.map(q).mkString(", ")}],
  "total_cost": ${report.totalCost},
  "node_slack": {
$slackJson
  }
}""")
          case _ =>
            section("Critical Path Analysis")
            println(s"  Path       : ${report.path.mkString(" → ")}")
            println(f"  Total cost : ${report.totalCost}%.2f")
            println()
            println("  Node slack (0 = on critical path):")
            report.nodeSlack.toSeq.sortBy(_._2).foreach { case (node, slack) =>
              val tag = if (slack < 1e-9) " ◄ CRITICAL" else ""
              println(f"    $node%-24s  $slack%.2f$tag")
            }
        }
      case None =>
        err("Critical path undefined: graph contains a cycle")
        sys.exit(2)
    }
  }

  def cmdCycles(engine: GraphEngine, fmt: String): Unit = {
    val report = engine.detectCycles()
    fmt match {
      case "json" =>
        val cyclesJson = report.cycles
          .map(c => s"[${c.map(q).mkString(", ")}]").mkString(", ")
        println(s"""{"has_cycles": ${report.hasCycles}, "cycles": [$cyclesJson]}""")
      case _ =>
        section("Cycle Detection")
        if (!report.hasCycles) {
          println("  ✓  No cycles detected")
        } else {
          println(s"  ✗  ${report.cycles.size} cycle(s) found:")
          report.cycles.zipWithIndex.foreach { case (c, i) =>
            println(s"    ${i + 1}. ${c.mkString(" → ")}")
          }
        }
    }
  }

  def cmdNeighbors(engine: GraphEngine, args: Array[String], fmt: String): Unit = {
    val nodeId = requireArg(args, 0, "neighbors <nodeId>")
    val result = engine.neighborhood(nodeId)
    fmt match {
      case "json" => println(jsonSet("neighborhood", result))
      case _ =>
        section(s"Neighborhood: $nodeId")
        field("Connected nodes (all directions)", result)
    }
  }

  def cmdAncestors(engine: GraphEngine, args: Array[String], fmt: String): Unit = {
    val nodeId = requireArg(args, 0, "ancestors <nodeId>")
    val result = engine.ancestors(nodeId)
    fmt match {
      case "json" => println(jsonSet("ancestors", result))
      case _ =>
        section(s"Ancestors: $nodeId")
        field("Parent nodes (hierarchy)", result)
    }
  }

  def cmdDescendants(engine: GraphEngine, args: Array[String], fmt: String): Unit = {
    val nodeId = requireArg(args, 0, "descendants <nodeId>")
    val result = engine.descendants(nodeId)
    fmt match {
      case "json" => println(jsonSet("descendants", result))
      case _ =>
        section(s"Descendants: $nodeId")
        field("Child nodes (hierarchy)", result)
    }
  }

  def cmdDiff(engine: GraphEngine, args: Array[String], fmt: String): Unit = {
    val snapshotFile = requireArg(args, 0, "diff <snapshotFile>")
    val other = loadEngine(snapshotFile, nodesFile) match {
      case Success(e) => e
      case Failure(ex) =>
        err(s"Failed to load snapshot: ${ex.getMessage}")
        sys.exit(1)
    }
    val d = engine.diff(other)
    fmt match {
      case "json" =>
        println(s"""{
  "added_nodes":   [${d.addedNodes.map(q).mkString(", ")}],
  "removed_nodes": [${d.removedNodes.map(q).mkString(", ")}],
  "added_edges":   [${d.addedEdges.map(edgeJson).mkString(", ")}],
  "removed_edges": [${d.removedEdges.map(edgeJson).mkString(", ")}],
  "changed_edges": [${d.changedEdges.map { case (o, n) =>
    s"""{"old": ${edgeJson(o)}, "new": ${edgeJson(n)}}"""}.mkString(", ")}]
}""")
      case _ =>
        section("Graph Diff")
        field("Added nodes",    d.addedNodes)
        field("Removed nodes",  d.removedNodes)
        field("Added edges",    d.addedEdges.map(e => s"${e.from}→${e.to}"))
        field("Removed edges",  d.removedEdges.map(e => s"${e.from}→${e.to}"))
        field("Changed edges",  d.changedEdges.map { case (o, n) =>
          s"${o.from}→${o.to} (${o.edgeType}→${n.edgeType})"
        })
    }
  }

  def cmdNodes(engine: GraphEngine, fmt: String): Unit = {
    fmt match {
      case "json" => println(s"""{"nodes": [${engine.allNodeIds.map(q).mkString(", ")}]}""")
      case _ =>
        section("All Nodes")
        engine.allNodeIds.toSeq.sorted.foreach(n => println(s"  - $n"))
    }
  }

  def cmdEdges(engine: GraphEngine, fmt: String): Unit = {
    fmt match {
      case "json" =>
        println(s"""{"edges": [${engine.edges.map(edgeJson).mkString(", ")}]}""")
      case _ =>
        section("All Edges")
        engine.edges.foreach(e =>
          println(f"  ${e.from}%-20s →  ${e.to}%-20s  [${e.edgeType}]  weight=${e.weight}"))
    }
  }

  def cmdExport(engine: GraphEngine, args: Array[String]): Unit = {
    val outFile  = requireArg(args, 0, "export <outFile>")
    val dotMode  = outFile.endsWith(".dot") || outFile.endsWith(".gv")

    val content =
      if (dotMode) exportDOT(engine)
      else         exportCSV(engine)

    val pw = new PrintWriter(new File(outFile))
    try pw.write(content) finally pw.close()
    println(s"Exported graph to $outFile")
  }


  // ----------------------------------------------------------------
  // Graph Loaders
  // ----------------------------------------------------------------

  def loadEngine(edgesPath: String, nodesPath: String): Try[GraphEngine] = Try {
    val edges = loadEdgesCSV(edgesPath)
    val nodes = if (new File(nodesPath).exists()) loadNodesCSV(nodesPath) else Seq.empty
    GraphEngine(edges, nodes)
  }

  def loadEdgesCSV(path: String): Seq[GraphEdge] = {
    val lines = Source.fromFile(path).getLines().toSeq
    val header = lines.head.trim.toLowerCase
    lines.tail
      .map(_.trim)
      .filter(_.nonEmpty)
      .map { line =>
        val cols = line.split(",", -1).map(_.trim)
        val edgeType = cols.lift(2).getOrElse("Dependency") match {
          case "Hierarchy"    => Hierarchy
          case "Relationship" => Relationship
          case _              => Dependency
        }
        val weight = cols.lift(3).flatMap(w => Try(w.toDouble).toOption).getOrElse(1.0)
        GraphEdge(cols(0), cols(1), edgeType, weight)
      }
  }

  def loadNodesCSV(path: String): Seq[GraphNode] = {
    Source.fromFile(path).getLines().toSeq
      .tail  // skip header
      .map(_.trim).filter(_.nonEmpty)
      .map { line =>
        val cols     = line.split(",", -1).map(_.trim)
        val duration = cols.lift(1).flatMap(d => Try(d.toDouble).toOption).getOrElse(0.0)
        GraphNode(cols(0), duration)
      }
  }


  // ----------------------------------------------------------------
  // Exporters
  // ----------------------------------------------------------------

  def exportCSV(engine: GraphEngine): String = {
    val header = "from,to,edgeType,weight"
    val rows   = engine.edges.map(e => s"${e.from},${e.to},${e.edgeType},${e.weight}")
    (header +: rows).mkString("\n")
  }

  def exportDOT(engine: GraphEngine): String = {
    val sb = new StringBuilder
    sb.append("digraph G {\n")
    sb.append("  rankdir=LR;\n")
    sb.append("  node [shape=box, fontname=Helvetica];\n\n")

    engine.edges.foreach { e =>
      val style = e.edgeType match {
        case Dependency   => "solid"
        case Hierarchy    => "dashed"
        case Relationship => "dotted"
      }
      val label = if (e.weight != 1.0) s""" label="${e.weight}"""" else ""
      sb.append(s"""  "${e.from}" -> "${e.to}" [style=$style$label];\n""")
    }

    sb.append("}\n")
    sb.toString()
  }


  // ----------------------------------------------------------------
  // Formatting Helpers
  // ----------------------------------------------------------------

  def section(title: String): Unit = {
    println(s"\n=== $title ===")
  }

  def field(label: String, values: Iterable[Any]): Unit = {
    if (values.isEmpty) println(s"  $label: (none)")
    else {
      println(s"  $label:")
      values.foreach(v => println(s"    - $v"))
    }
  }

  def q(s: String)          = s""""$s""""
  def jsonSet(key: String, values: Set[String]) =
    s"""{"$key": [${values.map(q).mkString(", ")}]}"""

  def edgeJson(e: GraphEdge) =
    s"""{"from": ${q(e.from)}, "to": ${q(e.to)}, "type": "${e.edgeType}", "weight": ${e.weight}}"""

  def err(msg: String): Unit = Console.err.println(s"[ERROR] $msg")

  def requireArg(args: Array[String], idx: Int, usage: String): String =
    args.lift(idx).getOrElse {
      err(s"Missing argument. Usage: $usage")
      sys.exit(1)
    }

  def flagValue(args: Array[String], flag: String): Option[String] = {
    val i = args.indexOf(flag)
    if (i >= 0 && i + 1 < args.length) Some(args(i + 1)) else None
  }

  def printHelp(): Unit = println("""
GraphEngine CLI
═══════════════════════════════════════════════════════════════

Usage:  scala GraphEngineCLI.scala <command> [nodeId] [options]

Commands
  impact      <nodeId>          Impact report (affected + dependents)
  closure     <nodeId>          Forward dependency closure
  rdeps       <nodeId>          Reverse dependency closure
  topo                          Topological execution order
  critical                      Critical path + node slack table
  cycles                        Detect cycles in the graph
  neighbors   <nodeId>          Full connected neighborhood
  ancestors   <nodeId>          Hierarchy ancestors
  descendants <nodeId>          Hierarchy descendants
  diff        <snapshotFile>    Diff current graph vs another CSV
  nodes                         List all node IDs
  edges                         List all edges
  export      <outFile>         Export as .csv or .dot/.gv

Options
  --graph   <file>   Edge CSV to load  (default: edges.csv)
  --nodes   <file>   Node CSV to load  (default: nodes.csv)
  --format  text     Output format: text | json  (default: text)

Edge CSV format:
  from,to,edgeType,weight
  A,B,Dependency,1.0
  A,C,Hierarchy,2.0

Node CSV format:
  id,duration
  A,3.0

Examples
  scala GraphEngineCLI.scala impact auth-service --graph services.csv
  scala GraphEngineCLI.scala critical --graph pipeline.csv --nodes jobs.csv
  scala GraphEngineCLI.scala topo --format json
  scala GraphEngineCLI.scala diff old_infra.csv --graph new_infra.csv
  scala GraphEngineCLI.scala export graph.dot
""")
}
