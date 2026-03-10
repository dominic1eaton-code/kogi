package portfolio.graph.api

import portfolio.graph._
import com.sun.net.httpserver.{HttpServer, HttpExchange, HttpHandler}
import java.net.InetSocketAddress
import java.io.{InputStream, OutputStream}
import scala.util.{Try, Success, Failure}
import scala.collection.mutable


// =============================================================
//  GraphEngine HTTP API Server
//
//  Lightweight REST API over the GraphEngine – no frameworks,
//  only the JDK's built-in HttpServer.
//
//  Start:
//    scala GraphEngineAPI.scala [port]          (default 8080)
//
//  Endpoints:
//    POST /graph/load                   – load a new graph (JSON body)
//    GET  /graph/nodes                  – list all nodes
//    GET  /graph/edges                  – list all edges
//    GET  /graph/impact/:id             – impact analysis
//    GET  /graph/closure/:id            – dependency closure
//    GET  /graph/rdeps/:id              – reverse closure
//    GET  /graph/neighbors/:id          – neighborhood
//    GET  /graph/ancestors/:id          – hierarchy ancestors
//    GET  /graph/descendants/:id        – hierarchy descendants
//    GET  /graph/topo                   – topological sort
//    GET  /graph/critical               – critical path
//    GET  /graph/cycles                 – cycle detection
//    POST /graph/diff                   – diff against a second graph
//    GET  /health                       – health check
//
//  JSON body for POST /graph/load:
//  {
//    "edges": [
//      {"from":"A","to":"B","type":"Dependency","weight":1.0}
//    ],
//    "nodes": [
//      {"id":"A","duration":3.0}
//    ]
//  }
// =============================================================

object GraphEngineAPIServer extends App {

  val port    = Try(args.headOption.map(_.toInt).getOrElse(8080)).getOrElse(8080)
  val server  = HttpServer.create(new InetSocketAddress(port), 0)
  val state   = new GraphState()

  // ---- Route registration ----
  server.createContext("/health",              new HealthHandler())
  server.createContext("/graph/load",          new LoadHandler(state))
  server.createContext("/graph/nodes",         new NodesHandler(state))
  server.createContext("/graph/edges",         new EdgesHandler(state))
  server.createContext("/graph/impact/",       new ImpactHandler(state))
  server.createContext("/graph/closure/",      new ClosureHandler(state))
  server.createContext("/graph/rdeps/",        new RdepsHandler(state))
  server.createContext("/graph/neighbors/",    new NeighborsHandler(state))
  server.createContext("/graph/ancestors/",    new AncestorsHandler(state))
  server.createContext("/graph/descendants/",  new DescendantsHandler(state))
  server.createContext("/graph/topo",          new TopoHandler(state))
  server.createContext("/graph/critical",      new CriticalHandler(state))
  server.createContext("/graph/cycles",        new CyclesHandler(state))
  server.createContext("/graph/diff",          new DiffHandler(state))

  server.setExecutor(null)
  server.start()

  println(s"GraphEngine API running on http://localhost:$port")
  println("POST /graph/load  to load a graph, then query away.")
  println("Press Ctrl-C to stop.\n")


  // ----------------------------------------------------------------
  // Mutable graph state (thread-safe swap)
  // ----------------------------------------------------------------

  class GraphState {
    @volatile private var _engine: Option[GraphEngine] = None
    def set(e: GraphEngine): Unit = { _engine = Some(e) }
    def get: Either[String, GraphEngine] =
      _engine.toRight("No graph loaded. POST /graph/load first.")
  }


  // ----------------------------------------------------------------
  // Base handler with HTTP/JSON helpers
  // ----------------------------------------------------------------

  abstract class BaseHandler extends HttpHandler {

    def handle(ex: HttpExchange): Unit = {
      val method = ex.getRequestMethod.toUpperCase
      Try(dispatch(ex, method)) match {
        case Success(_) =>
        case Failure(e) => respond(ex, 500, errorJson(s"Internal error: ${e.getMessage}"))
      }
    }

    def dispatch(ex: HttpExchange, method: String): Unit

    def respond(ex: HttpExchange, status: Int, body: String): Unit = {
      val bytes = body.getBytes("UTF-8")
      ex.getResponseHeaders.set("Content-Type", "application/json; charset=utf-8")
      ex.getResponseHeaders.set("Access-Control-Allow-Origin", "*")
      ex.sendResponseHeaders(status, bytes.length)
      val os: OutputStream = ex.getResponseBody
      try os.write(bytes) finally os.close()
    }

    def readBody(ex: HttpExchange): String = {
      val is: InputStream = ex.getRequestBody
      try scala.io.Source.fromInputStream(is, "UTF-8").mkString
      finally is.close()
    }

    def pathSegment(ex: HttpExchange, prefix: String): String =
      ex.getRequestURI.getPath.stripPrefix(prefix).stripPrefix("/")

    // ---- Mini JSON builders ----
    def q(s: String)   = s""""$s""""
    def jsonArr(xs: Iterable[String])  = s"[${xs.map(q).mkString(",")}]"
    def ok(body: String)  = body
    def errorJson(msg: String) = s"""{"error":${q(msg)}}"""
    def engineOrError(state: GraphState)(f: GraphEngine => String): String =
      state.get match {
        case Right(e) => f(e)
        case Left(m)  => errorJson(m)
      }
  }


  // ----------------------------------------------------------------
  // /health
  // ----------------------------------------------------------------

  class HealthHandler extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit =
      respond(ex, 200, """{"status":"ok"}""")
  }


  // ----------------------------------------------------------------
  // POST /graph/load
  //
  // Body:
  // {
  //   "edges": [{"from":"A","to":"B","type":"Dependency","weight":1.0}],
  //   "nodes": [{"id":"A","duration":2.0}]
  // }
  // ----------------------------------------------------------------

  class LoadHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit =
      method match {
        case "POST" =>
          val body = readBody(ex)
          Try(parseGraph(body)) match {
            case Success(engine) =>
              state.set(engine)
              val nodeCount = engine.allNodeIds.size
              val edgeCount = engine.edges.size
              respond(ex, 200,
                s"""{"loaded":true,"nodes":$nodeCount,"edges":$edgeCount}""")
            case Failure(ex2) =>
              respond(ex, 400, errorJson(s"Parse error: ${ex2.getMessage}"))
          }
        case _ => respond(ex, 405, errorJson("Method not allowed"))
      }

    def parseGraph(json: String): GraphEngine = {
      // Minimal hand-rolled JSON parsing – swap for circe/upickle in production
      val edges = parseEdges(json)
      val nodes = parseNodes(json)
      GraphEngine(edges, nodes)
    }

    def parseEdges(json: String): Seq[GraphEdge] = {
      val edgesSection = extractArraySection(json, "edges")
      parseObjects(edgesSection).map { obj =>
        val from     = extractStr(obj, "from")
        val to       = extractStr(obj, "to")
        val edgeType = extractStr(obj, "type") match {
          case "Hierarchy"    => Hierarchy
          case "Relationship" => Relationship
          case _              => Dependency
        }
        val weight = extractDbl(obj, "weight", 1.0)
        GraphEdge(from, to, edgeType, weight)
      }
    }

    def parseNodes(json: String): Seq[GraphNode] = {
      Try(extractArraySection(json, "nodes")).toOption.toSeq.flatMap { section =>
        parseObjects(section).map { obj =>
          val id       = extractStr(obj, "id")
          val duration = extractDbl(obj, "duration", 0.0)
          GraphNode(id, duration)
        }
      }
    }

    // ---- Very small JSON primitives (no library dependency) ----
    def extractArraySection(json: String, key: String): String = {
      val marker = s""""$key"\\s*:\\s*\\[""".r
      val start  = marker.findFirstMatchIn(json).map(_.end)
        .getOrElse(throw new Exception(s"key '$key' not found"))
      var depth = 1; var i = start
      while (i < json.length && depth > 0) {
        if (json(i) == '[') depth += 1
        else if (json(i) == ']') depth -= 1
        i += 1
      }
      json.substring(start, i - 1)
    }

    def parseObjects(section: String): Seq[Map[String, String]] = {
      val objRe = """\{[^{}]*\}""".r
      objRe.findAllIn(section).map { obj =>
        val pairRe = """"(\w+)"\s*:\s*"?([^",}]+)"?""".r
        pairRe.findAllMatchIn(obj).map(m => m.group(1) -> m.group(2).trim).toMap
      }.toSeq
    }

    def extractStr(obj: Map[String, String], key: String): String =
      obj.getOrElse(key, throw new Exception(s"Missing field: $key"))

    def extractDbl(obj: Map[String, String], key: String, default: Double): Double =
      obj.get(key).flatMap(v => Try(v.toDouble).toOption).getOrElse(default)
  }


  // ----------------------------------------------------------------
  // GET /graph/nodes
  // ----------------------------------------------------------------

  class NodesHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val body = engineOrError(state) { e =>
        s"""{"nodes":${jsonArr(e.allNodeIds.toSeq.sorted)}}"""
      }
      respond(ex, if (body.contains("error") && !body.contains("nodes")) 503 else 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/edges
  // ----------------------------------------------------------------

  class EdgesHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val body = engineOrError(state) { e =>
        val edgeItems = e.edges.map { edge =>
          s"""{"from":${q(edge.from)},"to":${q(edge.to)},"type":${q(edge.edgeType.toString)},"weight":${edge.weight}}"""
        }
        s"""{"edges":[${edgeItems.mkString(",")}]}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/impact/:id
  // ----------------------------------------------------------------

  class ImpactHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val id   = pathSegment(ex, "/graph/impact")
      val body = engineOrError(state) { e =>
        val r = e.impactAnalysis(id)
        s"""{"root":${q(r.root)},"affected":${jsonArr(r.affected)},"dependents":${jsonArr(r.dependents)}}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/closure/:id
  // ----------------------------------------------------------------

  class ClosureHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val id   = pathSegment(ex, "/graph/closure")
      val body = engineOrError(state) { e =>
        s"""{"node":${q(id)},"closure":${jsonArr(e.dependencyClosure(id))}}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/rdeps/:id
  // ----------------------------------------------------------------

  class RdepsHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val id   = pathSegment(ex, "/graph/rdeps")
      val body = engineOrError(state) { e =>
        s"""{"node":${q(id)},"reverse_closure":${jsonArr(e.reverseDependencyClosure(id))}}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/neighbors/:id
  // ----------------------------------------------------------------

  class NeighborsHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val id   = pathSegment(ex, "/graph/neighbors")
      val body = engineOrError(state) { e =>
        s"""{"node":${q(id)},"neighbors":${jsonArr(e.neighborhood(id))}}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/ancestors/:id
  // ----------------------------------------------------------------

  class AncestorsHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val id   = pathSegment(ex, "/graph/ancestors")
      val body = engineOrError(state) { e =>
        s"""{"node":${q(id)},"ancestors":${jsonArr(e.ancestors(id))}}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/descendants/:id
  // ----------------------------------------------------------------

  class DescendantsHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val id   = pathSegment(ex, "/graph/descendants")
      val body = engineOrError(state) { e =>
        s"""{"node":${q(id)},"descendants":${jsonArr(e.descendants(id))}}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/topo
  // ----------------------------------------------------------------

  class TopoHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val body = engineOrError(state) { e =>
        e.topologicalSort() match {
          case Right(order) =>
            s"""{"ok":true,"order":${jsonArr(order)}}"""
          case Left(cycleNodes) =>
            s"""{"ok":false,"error":"cycle_detected","cycle_nodes":${jsonArr(cycleNodes)}}"""
        }
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/critical
  // ----------------------------------------------------------------

  class CriticalHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val body = engineOrError(state) { e =>
        e.criticalPath() match {
          case Some(r) =>
            val slackEntries = r.nodeSlack
              .map { case (k, v) => s"${q(k)}:$v" }.mkString(",")
            s"""{"ok":true,"path":${jsonArr(r.path)},"total_cost":${r.totalCost},"node_slack":{$slackEntries}}"""
          case None =>
            s"""{"ok":false,"error":"cycle_detected"}"""
        }
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // GET /graph/cycles
  // ----------------------------------------------------------------

  class CyclesHandler(state: GraphState) extends BaseHandler {
    def dispatch(ex: HttpExchange, method: String): Unit = {
      val body = engineOrError(state) { e =>
        val r          = e.detectCycles()
        val cyclesJson = r.cycles.map(c => s"[${c.map(q).mkString(",")}]").mkString(",")
        s"""{"has_cycles":${r.hasCycles},"cycles":[$cyclesJson]}"""
      }
      respond(ex, 200, body)
    }
  }


  // ----------------------------------------------------------------
  // POST /graph/diff
  //
  // Body: same format as /graph/load – the "other" snapshot to diff against
  // ----------------------------------------------------------------

  class DiffHandler(state: GraphState) extends BaseHandler {
    val loader = new LoadHandler(state)

    def dispatch(ex: HttpExchange, method: String): Unit =
      method match {
        case "POST" =>
          val body = readBody(ex)
          val result = for {
            current <- state.get
            other   <- Try(loader.parseGraph(body))
              .toEither.left.map(e => s"Parse error: ${e.getMessage}")
          } yield {
            val d = current.diff(other)
            s"""{
  "added_nodes":   ${jsonArr(d.addedNodes)},
  "removed_nodes": ${jsonArr(d.removedNodes)},
  "added_edges":   [${d.addedEdges.map(edgeObj).mkString(",")}],
  "removed_edges": [${d.removedEdges.map(edgeObj).mkString(",")}],
  "changed_edges": [${d.changedEdges.map { case (o, n) =>
      s"""{"old":${edgeObj(o)},"new":${edgeObj(n)}}"""}.mkString(",")}]
}"""
          }
          result match {
            case Right(json) => respond(ex, 200, json)
            case Left(err2)  => respond(ex, 400, errorJson(err2))
          }

        case _ => respond(ex, 405, errorJson("Method not allowed"))
      }

    def edgeObj(e: GraphEdge) =
      s"""{"from":${q(e.from)},"to":${q(e.to)},"type":${q(e.edgeType.toString)},"weight":${e.weight}}"""
  }

}
