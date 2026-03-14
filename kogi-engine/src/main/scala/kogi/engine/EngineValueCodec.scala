package kogi.engine

import scala.jdk.CollectionConverters._

/**
 * Shared helpers for CLI + gRPC payload parsing and response shaping.
 * Keep inputs flexible (strings, numbers, maps) and outputs JSON-friendly.
 */
object EngineValueCodec {
  def normalize(value: Any): Any = value match {
    case null => null
    case opt: Option[_] => opt.map(normalize).orNull
    case p: Product if p.productArity == 0 =>
      // case objects (e.g., enum-like) -> string label
      p.toString
    case p: Product =>
      p.productElementNames
        .zip(p.productIterator)
        .map { case (k, v) => k -> normalize(v) }
        .toMap
    case m: Map[_, _] =>
      m.map { case (k, v) => k.toString -> normalize(v) }
    case it: Iterable[_] =>
      it.iterator.map(normalize).toList
    case l: java.util.List[_] =>
      l.asScala.iterator.map(normalize).toList
    case d: Double =>
      if (d.isNaN || d.isInfinity) 0.0 else d
    case f: Float =>
      if (f.isNaN || f.isInfinity) 0.0 else f.toDouble
    case b: Boolean => b
    case i: Int => i
    case l: Long => l
    case s: String => s
    case other => other.toString
  }

  def normalizeMap(value: Any): Map[String, Any] =
    normalize(value) match {
      case m: Map[_, _] => m.asInstanceOf[Map[String, Any]]
      case other => Map("value" -> other)
    }
}

object EngineRequestReader {
  def readString(map: Map[String, Any], keys: Seq[String], default: String = ""): String =
    keys.iterator
      .flatMap(key => map.get(key).map(asString))
      .find(_.nonEmpty)
      .getOrElse(default)

  def readOptString(map: Map[String, Any], keys: Seq[String]): Option[String] =
    keys.iterator
      .flatMap(key => map.get(key).map(asString))
      .find(_.nonEmpty)

  def readBoolean(map: Map[String, Any], keys: Seq[String], default: Boolean = false): Boolean =
    keys.iterator
      .flatMap(key => map.get(key).map(asBooleanOpt))
      .find(_.isDefined)
      .flatten
      .getOrElse(default)

  def readInt(map: Map[String, Any], keys: Seq[String], default: Int = 0): Int =
    keys.iterator
      .flatMap(key => map.get(key).map(asIntOpt))
      .find(_.isDefined)
      .flatten
      .getOrElse(default)

  def readLong(map: Map[String, Any], keys: Seq[String], default: Long = 0L): Long =
    keys.iterator
      .flatMap(key => map.get(key).map(asLongOpt))
      .find(_.isDefined)
      .flatten
      .getOrElse(default)

  def readDouble(map: Map[String, Any], keys: Seq[String], default: Double = 0.0): Double =
    keys.iterator
      .flatMap(key => map.get(key).map(asDoubleOpt))
      .find(_.isDefined)
      .flatten
      .getOrElse(default)

  def readStringList(map: Map[String, Any], keys: Seq[String]): List[String] =
    keys.iterator
      .flatMap(key => map.get(key).map(asStringList))
      .find(_.nonEmpty)
      .getOrElse(Nil)

  def readStringSet(map: Map[String, Any], keys: Seq[String]): Set[String] =
    readStringList(map, keys).map(_.trim).filter(_.nonEmpty).toSet

  def readStringMap(map: Map[String, Any], keys: Seq[String]): Map[String, String] =
    keys.iterator
      .flatMap(key => map.get(key).map(asStringMap))
      .find(_.nonEmpty)
      .getOrElse(Map.empty)

  def readMap(map: Map[String, Any], keys: Seq[String]): Map[String, Any] =
    keys.iterator
      .flatMap(key => map.get(key).map(asMap))
      .find(_.nonEmpty)
      .getOrElse(Map.empty)

  def readMapList(map: Map[String, Any], keys: Seq[String]): List[Map[String, Any]] =
    keys.iterator
      .flatMap(key => map.get(key).map(asMapList))
      .find(_.nonEmpty)
      .getOrElse(Nil)

  def asString(value: Any): String = value match {
    case null => ""
    case s: String => s
    case b: Boolean => b.toString
    case i: Int => i.toString
    case l: Long => l.toString
    case d: Double => if (d.isNaN || d.isInfinity) "0" else d.toString
    case f: Float => if (f.isNaN || f.isInfinity) "0" else f.toString
    case other => other.toString
  }

  def asBooleanOpt(value: Any): Option[Boolean] = value match {
    case b: Boolean => Some(b)
    case s: String =>
      s.trim.toLowerCase match {
        case "true" | "1" | "yes" | "on"  => Some(true)
        case "false" | "0" | "no" | "off" => Some(false)
        case _ => None
      }
    case i: Int => Some(i != 0)
    case l: Long => Some(l != 0L)
    case d: Double => Some(d != 0.0)
    case _ => None
  }

  def asIntOpt(value: Any): Option[Int] = value match {
    case i: Int => Some(i)
    case l: Long => Some(l.toInt)
    case d: Double => Some(d.toInt)
    case f: Float => Some(f.toInt)
    case s: String => s.trim.toIntOption
    case _ => None
  }

  def asLongOpt(value: Any): Option[Long] = value match {
    case l: Long => Some(l)
    case i: Int => Some(i.toLong)
    case d: Double => Some(d.toLong)
    case f: Float => Some(f.toLong)
    case s: String => s.trim.toLongOption
    case _ => None
  }

  def asDoubleOpt(value: Any): Option[Double] = value match {
    case d: Double => Some(d)
    case f: Float => Some(f.toDouble)
    case i: Int => Some(i.toDouble)
    case l: Long => Some(l.toDouble)
    case s: String => s.trim.toDoubleOption
    case _ => None
  }

  def asStringList(value: Any): List[String] = value match {
    case null => Nil
    case s: String => splitCsv(s)
    case it: Iterable[_] => it.map(asString).toList
    case arr: Array[_] => arr.map(asString).toList
    case other => splitCsv(asString(other))
  }

  def asMap(value: Any): Map[String, Any] = value match {
    case null => Map.empty
    case m: Map[_, _] =>
      m.map { case (k, v) => k.toString -> v }
    case s: String => parseMapString(s).view.mapValues(identity[Any]).toMap
    case other => Map("value" -> other)
  }

  def asMapList(value: Any): List[Map[String, Any]] = value match {
    case null => Nil
    case it: Iterable[_] => it.map(asMap).toList
    case arr: Array[_] => arr.map(asMap).toList
    case m: Map[_, _] => List(asMap(m))
    case s: String =>
      val entries = s.split("\|").toList.map(_.trim).filter(_.nonEmpty)
      val maps = entries.map(e => parseMapString(e.replace(';', ','))).filter(_.nonEmpty)
      if (maps.isEmpty) Nil else maps.map(m => m.view.mapValues(identity[Any]).toMap)
    case other => List(Map("value" -> other))
  }

  def asStringMap(value: Any): Map[String, String] = value match {
    case null => Map.empty
    case m: Map[_, _] =>
      m.map { case (k, v) => k.toString -> asString(v) }
    case s: String => parseMapString(s)
    case other => Map("value" -> asString(other))
  }

  def parseMapString(raw: String): Map[String, String] = {
    if (raw == null || raw.trim.isEmpty) return Map.empty
    splitCsv(raw).flatMap { entry =>
      val idx = entry.indexOf('=')
      if (idx <= 0 || idx >= entry.length - 1) None
      else Some(entry.substring(0, idx) -> entry.substring(idx + 1))
    }.toMap
  }

  def splitCsv(raw: String): List[String] =
    raw.split(",").map(_.trim).filter(_.nonEmpty).toList
}
