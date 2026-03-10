package portfolio.graph.api

import portfolio.graph._
import scala.util.{Try, Either, Right, Left}


// =============================================================
//  GraphEngineAPI  –  Programmatic Facade
//
//  A clean, safe Scala API that wraps GraphEngine behind a
//  builder, typed results, and error-handling layer.
//
//  Designed to be embedded in any application:
//    services, batch jobs, test suites, REPL sessions.
//
//  Usage pattern:
//    val api = GraphEngineAPI.builder
//                .addEdge("A", "B", Dependency)
//                .addNode("A", duration = 3.0)
//                .build()
//
//    api.impact("A")             // Right(ImpactReport)
//    api.criticalPath()          // Right(CriticalPathReport)
//    api.cycles()                // Right(CycleReport)
//    api.snapshot()              // returns a handle for diffing later
// =============================================================


// ─────────────────────────────────────────────────────────────
// Result type aliases
// ─────────────────────────────────────────────────────────────

type ApiResult[A] = Either[ApiError, A]

sealed trait ApiError
case class NodeNotFound(id: String)          extends ApiError
case class CyclePresent(nodes: Set[String])  extends ApiError
case class EngineError(message: String)      extends ApiError


// ─────────────────────────────────────────────────────────────
// Snapshot handle (returned by .snapshot())
// Used as the argument to .diff(snapshot)
// ─────────────────────────────────────────────────────────────

opaque type Snapshot = GraphEngine
object Snapshot:
  private[api] def apply(e: GraphEngine): Snapshot = e
  private[api] def engine(s: Snapshot): GraphEngine = s


// ─────────────────────────────────────────────────────────────
// Builder
// ─────────────────────────────────────────────────────────────

class GraphEngineBuilder private[api] (
  private val edgesBuf: Vector[GraphEdge] = Vector.empty,
  private val nodesBuf: Vector[GraphNode] = Vector.empty
) {

  // ---- Edge builders ----

  def addEdge(from: String, to: String, edgeType: EdgeType, weight: Double = 1.0): GraphEngineBuilder =
    new GraphEngineBuilder(edgesBuf :+ GraphEdge(from, to, edgeType, weight), nodesBuf)

  def addDependency(from: String, to: String, weight: Double = 1.0): GraphEngineBuilder =
    addEdge(from, to, Dependency, weight)

  def addHierarchy(parent: String, child: String): GraphEngineBuilder =
    addEdge(parent, child, Hierarchy)

  def addRelationship(a: String, b: String): GraphEngineBuilder =
    addEdge(a, b, Relationship)

  def addEdges(es: Seq[GraphEdge]): GraphEngineBuilder =
    new GraphEngineBuilder(edgesBuf ++ es, nodesBuf)

  // ---- Node builders ----

  def addNode(id: String, duration: Double = 0.0): GraphEngineBuilder =
    new GraphEngineBuilder(edgesBuf, nodesBuf :+ GraphNode(id, duration))

  def addNodes(ns: Seq[GraphNode]): GraphEngineBuilder =
    new GraphEngineBuilder(edgesBuf, nodesBuf ++ ns)

  // ---- Build ----

  def build(): GraphEngineAPI =
    new GraphEngineAPI(GraphEngine(edgesBuf, nodesBuf))
}


// ─────────────────────────────────────────────────────────────
// Main API class
// ─────────────────────────────────────────────────────────────

class GraphEngineAPI private[api] (private val engine: GraphEngine) {

  // ---- Node / Edge accessors ─────────────────────────────────

  /** All node IDs present in the graph. */
  def nodes: Set[String] = engine.allNodeIds

  /** All edges. */
  def edges: Seq[GraphEdge] = engine.edges

  /** Check if a node exists. */
  def contains(id: String): Boolean = engine.allNodeIds.contains(id)


  // ---- Impact Analysis ───────────────────────────────────────

  /**
   * Full impact report for a node.
   * Returns NodeNotFound if the id is not in the graph.
   */
  def impact(id: String): ApiResult[ImpactReport] =
    withNode(id)(engine.impactAnalysis)

  /**
   * Nodes this node transitively depends on (forward closure).
   */
  def dependencyClosure(id: String): ApiResult[Set[String]] =
    withNode(id)(engine.dependencyClosure)

  /**
   * Nodes that transitively depend on this node (reverse closure).
   */
  def reverseClosure(id: String): ApiResult[Set[String]] =
    withNode(id)(engine.reverseDependencyClosure)

  /**
   * All nodes reachable from this node in any direction.
   */
  def neighborhood(id: String): ApiResult[Set[String]] =
    withNode(id)(engine.neighborhood)


  // ---- Hierarchy Traversal ───────────────────────────────────

  /**
   * All hierarchy descendants of a node.
   */
  def descendants(id: String): ApiResult[Set[String]] =
    withNode(id)(engine.descendants)

  /**
   * All hierarchy ancestors of a node.
   */
  def ancestors(id: String): ApiResult[Set[String]] =
    withNode(id)(engine.ancestors)


  // ---- Topological Scheduling ────────────────────────────────

  /**
   * Returns a valid topological execution order for Dependency edges.
   * Returns CyclePresent if the graph is not a DAG.
   */
  def topologicalOrder(): ApiResult[Seq[String]] =
    engine.topologicalSort() match {
      case Right(order)  => Right(order)
      case Left(cycleNodes) => Left(CyclePresent(cycleNodes))
    }

  /**
   * Topological order filtered to a specific edge type.
   */
  def topologicalOrder(filter: EdgeType => Boolean): ApiResult[Seq[String]] =
    engine.topologicalSort(filter) match {
      case Right(order)     => Right(order)
      case Left(cycleNodes) => Left(CyclePresent(cycleNodes))
    }


  // ---- Cycle Detection ───────────────────────────────────────

  /**
   * Detect cycles across all edge types.
   */
  def cycles(): CycleReport =
    engine.detectCycles()

  /**
   * Detect cycles across a filtered subset of edges.
   */
  def cycles(filter: EdgeType => Boolean): CycleReport =
    engine.detectCycles(filter)

  /**
   * Returns true if the graph is a DAG (no cycles on Dependency edges).
   */
  def isDAG: Boolean =
    !engine.detectCycles(_ == Dependency).hasCycles


  // ---- Critical Path ─────────────────────────────────────────

  /**
   * Returns the critical path analysis.
   * Returns CyclePresent if the graph contains cycles.
   */
  def criticalPath(): ApiResult[CriticalPathReport] =
    engine.criticalPath() match {
      case Some(r) => Right(r)
      case None    =>
        val cycleNodes = engine.detectCycles(_ == Dependency).cycles.flatten.toSet
        Left(CyclePresent(cycleNodes))
    }

  /**
   * Returns only the ordered list of critical-path node IDs.
   * Convenience wrapper around criticalPath().
   */
  def criticalPathNodes(): ApiResult[Seq[String]] =
    criticalPath().map(_.path)

  /**
   * Returns the minimum project duration (total cost).
   */
  def minimumDuration(): ApiResult[Double] =
    criticalPath().map(_.totalCost)

  /**
   * Returns the slack for a specific node.
   * Slack = 0 means the node is on the critical path.
   */
  def slackFor(id: String): ApiResult[Double] =
    for {
      _      <- withNode(id)(identity)
      report <- criticalPath()
    } yield report.nodeSlack.getOrElse(id, 0.0)


  // ---- Snapshot & Diffing ────────────────────────────────────

  /**
   * Capture the current graph as an immutable snapshot.
   * Pass the snapshot to diff() later to see what changed.
   */
  def snapshot(): Snapshot = Snapshot(engine)

  /**
   * Diff this graph against an earlier snapshot.
   */
  def diff(earlier: Snapshot): GraphDiff =
    Snapshot.engine(earlier).diff(engine)

  /**
   * Diff this graph against another API instance.
   */
  def diffWith(other: GraphEngineAPI): GraphDiff =
    engine.diff(other.engine)


  // ---- Bulk Query Helpers ────────────────────────────────────

  /**
   * Run impactAnalysis on every node and return a map.
   * Useful for pre-computing a full impact matrix.
   */
  def allImpacts(): Map[String, ImpactReport] =
    engine.allNodeIds.map(id => id -> engine.impactAnalysis(id)).toMap

  /**
   * Find all root nodes (no incoming Dependency edges).
   */
  def roots(): Set[String] =
    engine.allNodeIds.filter { id =>
      engine.edges.forall(e => e.edgeType != Dependency || e.to != id)
    }

  /**
   * Find all leaf nodes (no outgoing Dependency edges).
   */
  def leaves(): Set[String] =
    engine.allNodeIds.filter { id =>
      engine.edges.forall(e => e.edgeType != Dependency || e.from != id)
    }

  /**
   * Compute the in-degree of every node (Dependency edges only).
   */
  def inDegrees(): Map[String, Int] =
    engine.allNodeIds.map { id =>
      id -> engine.edges.count(e => e.edgeType == Dependency && e.to == id)
    }.toMap

  /**
   * Compute the out-degree of every node (Dependency edges only).
   */
  def outDegrees(): Map[String, Int] =
    engine.allNodeIds.map { id =>
      id -> engine.edges.count(e => e.edgeType == Dependency && e.from == id)
    }.toMap

  /**
   * Return nodes sorted by their number of transitive dependents –
   * highest first. Useful for "most critical node" ranking.
   */
  def byImpactSize(): Seq[(String, Int)] =
    engine.allNodeIds.toSeq
      .map(id => id -> engine.reverseDependencyClosure(id).size)
      .sortBy(-_._2)


  // ---- Mutation – return a new API instance ──────────────────

  /**
   * Return a new API with an additional edge.
   */
  def withEdge(from: String, to: String, edgeType: EdgeType, weight: Double = 1.0): GraphEngineAPI =
    new GraphEngineAPI(GraphEngine(engine.edges :+ GraphEdge(from, to, edgeType, weight), engine.nodes))

  /**
   * Return a new API with an edge removed.
   */
  def withoutEdge(from: String, to: String): GraphEngineAPI =
    new GraphEngineAPI(GraphEngine(
      engine.edges.filterNot(e => e.from == from && e.to == to),
      engine.nodes
    ))

  /**
   * Return a new API with a node and all its edges removed.
   */
  def withoutNode(id: String): GraphEngineAPI =
    new GraphEngineAPI(GraphEngine(
      engine.edges.filterNot(e => e.from == id || e.to == id),
      engine.nodes.filterNot(_.id == id)
    ))


  // ---- Private helpers ───────────────────────────────────────

  private def withNode[A](id: String)(f: String => A): ApiResult[A] =
    if (engine.allNodeIds.contains(id)) Right(f(id))
    else Left(NodeNotFound(id))
}


// ─────────────────────────────────────────────────────────────
// Companion object entry point
// ─────────────────────────────────────────────────────────────

object GraphEngineAPI {

  /** Start building a new graph. */
  def builder: GraphEngineBuilder = new GraphEngineBuilder()

  /** Wrap an existing GraphEngine directly. */
  def fromEngine(engine: GraphEngine): GraphEngineAPI =
    new GraphEngineAPI(engine)

  /** Build from raw edge/node sequences directly. */
  def from(edges: Seq[GraphEdge], nodes: Seq[GraphNode] = Seq.empty): GraphEngineAPI =
    new GraphEngineAPI(GraphEngine(edges, nodes))
}


// ─────────────────────────────────────────────────────────────
// Usage Examples
// ─────────────────────────────────────────────────────────────

object GraphEngineAPIDemo extends App {

  // ---- Build with fluent builder ----
  val api = GraphEngineAPI.builder
    .addNode("checkout",    duration = 1)
    .addNode("payment",     duration = 3)
    .addNode("fraud-check", duration = 2)
    .addNode("fulfillment", duration = 4)
    .addNode("notify",      duration = 1)
    .addDependency("checkout",    "payment",     weight = 0)
    .addDependency("payment",     "fraud-check", weight = 0)
    .addDependency("fraud-check", "fulfillment", weight = 0)
    .addDependency("fulfillment", "notify",      weight = 0)
    .addHierarchy("order-flow", "checkout")
    .addHierarchy("order-flow", "notify")
    .build()


  // ---- Safe results with pattern match ----
  api.impact("payment") match {
    case Right(r) =>
      println(s"Impact of 'payment': affects=${r.affected}, dependents=${r.dependents}")
    case Left(NodeNotFound(id)) =>
      println(s"Node '$id' not in graph")
    case Left(err) =>
      println(s"Error: $err")
  }

  // ---- Critical path ----
  api.criticalPath() match {
    case Right(r) =>
      println(s"Critical path: ${r.path.mkString(" → ")}  (${r.totalCost} units)")
    case Left(CyclePresent(nodes)) =>
      println(s"Cycle in: $nodes")
    case Left(err) =>
      println(s"Error: $err")
  }

  // ---- Convenience accessors ----
  println(s"Roots : ${api.roots()}")
  println(s"Leaves: ${api.leaves()}")
  println(s"Is DAG: ${api.isDAG}")
  println(s"Ranked by impact: ${api.byImpactSize()}")

  // ---- Snapshot & diff ----
  val snap = api.snapshot()

  val api2 = api
    .withEdge("checkout", "fraud-check", Dependency)   // added shortcut
    .withoutEdge("payment", "fraud-check")             // removed original

  val d = api2.diff(snap)
  println(s"Changes since snapshot → added: ${d.addedEdges.size}, removed: ${d.removedEdges.size}")

  // ---- Mutation: simulate removing a node ----
  val withoutFraud = api.withoutNode("fraud-check")
  println(s"After removing fraud-check – nodes: ${withoutFraud.nodes}")

  // ---- Topological order ----
  api.topologicalOrder() match {
    case Right(order) => println(s"Execution order: ${order.mkString(" → ")}")
    case Left(CyclePresent(nodes)) => println(s"Cannot order – cycle: $nodes")
    case Left(err) => println(s"Error: $err")
  }
}
