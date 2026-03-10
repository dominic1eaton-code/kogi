package kogi.engine

import scala.collection.mutable

// -----------------------------------------------------
// Graph Data Structures
// -----------------------------------------------------

case class GraphEdge(
  from: String,
  to: String,
  edgeType: EdgeType,
  weight: Double = 1.0   // used for critical-path cost
)

sealed trait EdgeType
case object Dependency   extends EdgeType
case object Hierarchy    extends EdgeType
case object Relationship extends EdgeType

case class GraphNode(
  id: String,
  duration: Double = 0.0  // task duration for critical-path
)


// -----------------------------------------------------
// Result Types
// -----------------------------------------------------

case class ImpactReport(
  root:       String,
  affected:   Set[String],   // downstream (what this depends on)
  dependents: Set[String]    // upstream   (what depends on this)
)

case class CycleReport(
  hasCycles: Boolean,
  cycles:    Seq[Seq[String]]  // each inner Seq is one cycle path
)

case class CriticalPathReport(
  path:        Seq[String],
  totalCost:   Double,
  nodeSlack:   Map[String, Double]  // slack == 0  ⟹  node is on critical path
)

case class GraphDiff(
  addedNodes:    Set[String],
  removedNodes:  Set[String],
  addedEdges:    Set[GraphEdge],
  removedEdges:  Set[GraphEdge],
  changedEdges:  Set[(GraphEdge, GraphEdge)]  // (old, new) – same (from,to) but mutated
)


// -----------------------------------------------------
// Graph Engine
// -----------------------------------------------------

class GraphEngine(
  val edges: Seq[GraphEdge],
  val nodes: Seq[GraphNode] = Seq.empty
) {

  // Node duration map (defaults to 0 if node not registered)
  private val nodeDuration: Map[String, Double] =
    nodes.map(n => n.id -> n.duration).toMap.withDefaultValue(0.0)

  // All vertex ids (union of edge endpoints + registered nodes)
  val allNodeIds: Set[String] =
    edges.flatMap(e => Seq(e.from, e.to)).toSet ++ nodes.map(_.id)

  // Adjacency maps
  private val forward: Map[String, Set[GraphEdge]] =
    edges.groupBy(_.from).view.mapValues(_.toSet).toMap

  private val reverse: Map[String, Set[GraphEdge]] =
    edges.groupBy(_.to).view.mapValues(_.toSet).toMap


  // ===================================================
  // 1. Generic DFS Traversal (forward)
  // ===================================================

  def traverseFrom(
      start:      String,
      edgeFilter: EdgeType => Boolean
  ): Set[String] = {

    val visited = mutable.Set[String]()
    val stack   = mutable.Stack[String](start)

    while (stack.nonEmpty) {
      val node = stack.pop()
      if (!visited(node)) {
        visited += node
        forward.getOrElse(node, Set())
          .filter(e => edgeFilter(e.edgeType))
          .foreach(e => stack.push(e.to))
      }
    }
    visited.toSet - start
  }


  // ===================================================
  // 2. Generic DFS Traversal (reverse)
  // ===================================================

  def reverseTraverseFrom(
      start:      String,
      edgeFilter: EdgeType => Boolean
  ): Set[String] = {

    val visited = mutable.Set[String]()
    val stack   = mutable.Stack[String](start)

    while (stack.nonEmpty) {
      val node = stack.pop()
      if (!visited(node)) {
        visited += node
        reverse.getOrElse(node, Set())
          .filter(e => edgeFilter(e.edgeType))
          .foreach(e => stack.push(e.from))
      }
    }
    visited.toSet - start
  }


  // ===================================================
  // 3. Dependency Closure  (forward Dependency edges)
  // ===================================================

  def dependencyClosure(itemId: String): Set[String] =
    traverseFrom(itemId, _ == Dependency)

  def reverseDependencyClosure(itemId: String): Set[String] =
    reverseTraverseFrom(itemId, _ == Dependency)


  // ===================================================
  // 4. Impact Analysis
  // ===================================================

  def impactAnalysis(itemId: String): ImpactReport =
    ImpactReport(
      root       = itemId,
      affected   = dependencyClosure(itemId),
      dependents = reverseDependencyClosure(itemId)
    )


  // ===================================================
  // 5. Hierarchy Traversal
  // ===================================================

  def descendants(itemId: String): Set[String] =
    traverseFrom(itemId, _ == Hierarchy)

  def ancestors(itemId: String): Set[String] =
    reverseTraverseFrom(itemId, _ == Hierarchy)


  // ===================================================
  // 6. Full Neighbourhood
  // ===================================================

  def neighborhood(itemId: String): Set[String] =
    traverseFrom(itemId, _ => true) ++ reverseTraverseFrom(itemId, _ => true)


  // ===================================================
  // 7. Cycle Detection  (DFS colouring on a chosen
  //    sub-graph filtered by edge type)
  //
  //    Colours: 0 = unvisited, 1 = in-stack, 2 = done
  // ===================================================

  def detectCycles(
      edgeFilter: EdgeType => Boolean = _ => true
  ): CycleReport = {

    val colour  = mutable.Map[String, Int]().withDefaultValue(0)
    val parent  = mutable.Map[String, String]()
    val cycles  = mutable.ListBuffer[Seq[String]]()

    def reconstructCycle(start: String, end: String): Seq[String] = {
      val path  = mutable.ListBuffer[String](end)
      var cur   = end
      while (cur != start) {
        cur = parent(cur)
        path.prepend(cur)
      }
      path.toSeq
    }

    def dfs(node: String): Unit = {
      colour(node) = 1
      forward.getOrElse(node, Set())
        .filter(e => edgeFilter(e.edgeType))
        .foreach { e =>
          val next = e.to
          colour(next) match {
            case 1 =>  // back-edge → cycle found
              cycles += reconstructCycle(next, node)
            case 0 =>
              parent(next) = node
              dfs(next)
            case _ => // already fully explored
          }
        }
      colour(node) = 2
    }

    allNodeIds.foreach { n =>
      if (colour(n) == 0) dfs(n)
    }

    CycleReport(hasCycles = cycles.nonEmpty, cycles = cycles.toSeq)
  }


  // ===================================================
  // 8. Topological Sort / Scheduling
  //
  //    Uses Kahn's algorithm (BFS, in-degree).
  //    Returns Left(cycleNodes) if a cycle is present,
  //    Right(order) otherwise.
  // ===================================================

  def topologicalSort(
      edgeFilter: EdgeType => Boolean = _ == Dependency
  ): Either[Set[String], Seq[String]] = {

    val relevantEdges = edges.filter(e => edgeFilter(e.edgeType))

    val inDegree = mutable.Map[String, Int]()
      .withDefaultValue(0)
    allNodeIds.foreach(n => inDegree(n) = 0)
    relevantEdges.foreach(e => inDegree(e.to) += 1)

    val queue  = mutable.Queue[String]()
    inDegree.filter(_._2 == 0).keys.foreach(queue.enqueue(_))

    val result = mutable.ListBuffer[String]()

    while (queue.nonEmpty) {
      val node = queue.dequeue()
      result   += node
      relevantEdges
        .filter(_.from == node)
        .foreach { e =>
          inDegree(e.to) -= 1
          if (inDegree(e.to) == 0) queue.enqueue(e.to)
        }
    }

    if (result.size < allNodeIds.size) {
      // Remaining non-zero in-degree nodes are in cycles
      val cycleNodes = inDegree.filter(_._2 > 0).keySet.toSet
      Left(cycleNodes)
    } else {
      Right(result.toSeq)
    }
  }


  // ===================================================
  // 9. Critical Path Analysis  (CPM – longest path)
  //
  //    Works on a DAG of Dependency edges with weights.
  //    If cycles exist, returns None.
  //
  //    EST  = Earliest Start Time
  //    EFT  = Earliest Finish Time  = EST + duration
  //    LST  = Latest Start Time
  //    LFT  = Latest Finish Time
  //    Slack = LST – EST  (0 means on critical path)
  // ===================================================

  def criticalPath(
      edgeFilter: EdgeType => Boolean = _ == Dependency
  ): Option[CriticalPathReport] = {

    topologicalSort(edgeFilter) match {
      case Left(_) => None   // cycle → undefined

      case Right(order) =>
        val relevantEdges = edges.filter(e => edgeFilter(e.edgeType))

        // ---------- Forward pass (EST) ----------
        val est = mutable.Map[String, Double]().withDefaultValue(0.0)
        order.foreach { node =>
          val incoming = relevantEdges.filter(_.to == node)
          if (incoming.nonEmpty)
            est(node) = incoming.map(e => est(e.from) + e.weight).max
        }

        // EFT per node
        val eft = order.map(n => n -> (est(n) + nodeDuration(n))).toMap

        val projectEnd = eft.values.maxOption.getOrElse(0.0)

        // ---------- Backward pass (LST) ----------
        val lft = mutable.Map[String, Double]()
        order.reverse.foreach { node =>
          val outgoing = relevantEdges.filter(_.from == node)
          lft(node) =
            if (outgoing.isEmpty) projectEnd
            else outgoing.map(e => lft(e.to) - e.weight).min
        }
        val lst = order.map(n => n -> (lft(n) - nodeDuration(n))).toMap

        // ---------- Slack & critical path ----------
        val slack = order.map(n => n -> (lst(n) - est(n))).toMap

        // Reconstruct the critical path (nodes with 0 slack, in topo order)
        val criticalNodes = order.filter(n => slack(n) < 1e-9).toSeq

        Some(CriticalPathReport(
          path      = criticalNodes,
          totalCost = projectEnd,
          nodeSlack = slack
        ))
    }
  }


  // ===================================================
  // 10. Graph Diffing between two snapshots
  // ===================================================

  def diff(other: GraphEngine): GraphDiff = {

    val thisNodeSet  = this.allNodeIds
    val otherNodeSet = other.allNodeIds

    val thisEdgeSet  = this.edges.toSet
    val otherEdgeSet = other.edges.toSet

    // Edges keyed by (from, to) for change detection
    def edgeKey(e: GraphEdge) = (e.from, e.to)
    val thisEdgeMap  = this.edges.map(e  => edgeKey(e) -> e).toMap
    val otherEdgeMap = other.edges.map(e => edgeKey(e) -> e).toMap

    val sharedKeys     = thisEdgeMap.keySet intersect otherEdgeMap.keySet
    val changedEdges   = sharedKeys
      .flatMap { k =>
        val old = thisEdgeMap(k)
        val nw  = otherEdgeMap(k)
        if (old != nw) Some((old, nw)) else None
      }
      .toSet

    // Purely added / removed (ignoring mutated edges already in changedEdges)
    val changedKeys    = changedEdges.map { case (o, _) => edgeKey(o) }
    val purelyAdded    = otherEdgeSet -- thisEdgeSet -- changedEdges.map(_._2)
    val purelyRemoved  = thisEdgeSet  -- otherEdgeSet -- changedEdges.map(_._1)

    GraphDiff(
      addedNodes   = otherNodeSet  -- thisNodeSet,
      removedNodes = thisNodeSet   -- otherNodeSet,
      addedEdges   = purelyAdded,
      removedEdges = purelyRemoved,
      changedEdges = changedEdges
    )
  }
}


// -----------------------------------------------------
// Companion Object – convenient constructors
// -----------------------------------------------------

object GraphEngine {

  /** Build from edges only (no explicit node durations). */
  def apply(edges: Seq[GraphEdge]): GraphEngine =
    new GraphEngine(edges)

  /** Build with explicit node metadata (durations for CPM). */
  def apply(edges: Seq[GraphEdge], nodes: Seq[GraphNode]): GraphEngine =
    new GraphEngine(edges, nodes)
}


// -----------------------------------------------------
// Demo / Smoke-test  (remove before production use)
// -----------------------------------------------------

object GraphEngineDemo extends App {

  // ---- Build a small dependency DAG ----
  //
  //   A ──dep──► B ──dep──► D
  //   A ──dep──► C ──dep──► D
  //              C ──dep──► E

  val edges = Seq(
    GraphEdge("A", "B", Dependency, weight = 3),
    GraphEdge("A", "C", Dependency, weight = 1),
    GraphEdge("B", "D", Dependency, weight = 2),
    GraphEdge("C", "D", Dependency, weight = 4),
    GraphEdge("C", "E", Dependency, weight = 1),
    // Hierarchy edges
    GraphEdge("root", "A", Hierarchy),
    GraphEdge("root", "C", Hierarchy)
  )

  val nodes = Seq(
    GraphNode("A", duration = 2),
    GraphNode("B", duration = 3),
    GraphNode("C", duration = 1),
    GraphNode("D", duration = 2),
    GraphNode("E", duration = 1),
    GraphNode("root", duration = 0)
  )

  val engine = GraphEngine(edges, nodes)

  // Impact analysis
  val impact = engine.impactAnalysis("A")
  println(s"Impact of A → affected: ${impact.affected}, dependents: ${impact.dependents}")

  // Cycle detection (should be clean)
  val cycles = engine.detectCycles(_ == Dependency)
  println(s"Cycles detected: ${cycles.hasCycles}  ${cycles.cycles}")

  // Introduce a cycle for testing
  val cycleEdges = edges :+ GraphEdge("D", "A", Dependency)
  val cycleEngine = GraphEngine(cycleEdges, nodes)
  val cycleResult = cycleEngine.detectCycles(_ == Dependency)
  println(s"Cycle in cycleEngine: ${cycleResult.hasCycles}  paths: ${cycleResult.cycles}")

  // Topological sort
  engine.topologicalSort() match {
    case Right(order) => println(s"Topo order: $order")
    case Left(bad)    => println(s"Cannot sort – cycle nodes: $bad")
  }

  // Critical path
  engine.criticalPath() match {
    case Some(report) =>
      println(s"Critical path: ${report.path}")
      println(s"Project duration: ${report.totalCost}")
      println(s"Slack per node:   ${report.nodeSlack}")
    case None =>
      println("Critical path undefined (cycle present)")
  }

  // Graph diff
  val edges2 = edges.filterNot(e => e.from == "A" && e.to == "B") :+
    GraphEdge("A", "F", Dependency, weight = 5)
  val engine2 = GraphEngine(edges2, nodes)

  val d = engine.diff(engine2)
  println(s"Diff → added nodes: ${d.addedNodes}, removed nodes: ${d.removedNodes}")
  println(s"       added edges: ${d.addedEdges}")
  println(s"       removed edges: ${d.removedEdges}")
  println(s"       changed edges: ${d.changedEdges}")
}