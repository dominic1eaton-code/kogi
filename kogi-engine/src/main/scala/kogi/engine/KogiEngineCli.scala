
package kogi.engine

import scala.collection.mutable

import EngineRequestReader._

object KogiEngineCli {
  def main(args: Array[String]): Unit = {
    val parsed = parseArgs(args.toList)
    val action = parsed.options.getOrElse("action", "snapshot").toLowerCase
    val engine = new KogiEngine()

    val mergedTags = (parsed.tags ++ splitCsv(parsed.options.getOrElse("tags", ""))).distinct
    val mergedMetadata = parsed.metadata ++ parseMapString(parsed.options.getOrElse("metadata", ""))
    val mergedPayload = parsed.payload ++ parseMapString(parsed.options.getOrElse("payload", ""))

    val request: Map[String, Any] =
      parsed.options ++ Map(
        "tags" -> mergedTags,
        "metadata" -> mergedMetadata,
        "payload" -> mergedPayload
      )

    val response = EngineActionDispatcher.handle(engine, action, request)
    println(JsonPrinter.render(response))
  }

  // ----------------------------------------------------------------
  // Argument parsing
  // ----------------------------------------------------------------

  final case class ParsedArgs(
      options:  Map[String, String],
      payload:  Map[String, String],
      tags:     List[String],
      metadata: Map[String, String]
  )

  def parseArgs(args: List[String]): ParsedArgs = {
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
}

object JsonPrinter {
  def render(value: Any): String = value match {
    case null           => "null"
    case opt: Option[_] => render(opt.orNull)
    case p: Product     => render(EngineValueCodec.normalize(p))
    case s: String      => "\"" + escape(s) + "\""
    case b: Boolean     => b.toString
    case i: Int         => i.toString
    case l: Long        => l.toString
    case d: Double      => if (d.isNaN || d.isInfinity) "0.0" else d.toString
    case f: Float       => if (f.isNaN || f.isInfinity) "0.0" else f.toString
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
