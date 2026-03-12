package kogi.engine

import com.google.protobuf.{ListValue, NullValue, Struct, Value}
import io.grpc.{BindableService, MethodDescriptor, ServerBuilder, ServerServiceDefinition}
import io.grpc.protobuf.ProtoUtils
import io.grpc.protobuf.services.ProtoReflectionService
import io.grpc.stub.{ServerCalls, StreamObserver}

import scala.jdk.CollectionConverters._

object EngineGrpcServer {
  def main(args: Array[String]): Unit = {
    val port = sys.env.get("KOGI_ENGINE_GRPC_PORT").flatMap(_.toIntOption).getOrElse(9100)
    val engine = new KogiEngine()
    val server = ServerBuilder
      .forPort(port)
      .addService(new EngineGrpcService(engine))
      .addService(ProtoReflectionService.newInstance())
      .build()
      .start()

    println(s"kogi-engine grpc listening on 0.0.0.0:$port")

    sys.addShutdownHook {
      server.shutdown()
    }

    server.awaitTermination()
  }
}

final class EngineGrpcService(engine: KogiEngine) extends BindableService {
  import EngineGrpcCodec._

  private val serviceName = "kogi.engine.v1.EngineService"

  private val controlMethod = unaryMethod("Control")
  private val statusMethod = unaryMethod("Status")
  private val ingestMethod = unaryMethod("Ingest")
  private val snapshotMethod = unaryMethod("Snapshot")

  override def bindService(): ServerServiceDefinition = {
    ServerServiceDefinition
      .builder(serviceName)
      .addMethod(controlMethod, ServerCalls.asyncUnaryCall(handleControl))
      .addMethod(statusMethod, ServerCalls.asyncUnaryCall(handleStatus))
      .addMethod(ingestMethod, ServerCalls.asyncUnaryCall(handleIngest))
      .addMethod(snapshotMethod, ServerCalls.asyncUnaryCall(handleSnapshot))
      .build()
  }

  private def unaryMethod(name: String): MethodDescriptor[Struct, Struct] =
    MethodDescriptor
      .newBuilder[Struct, Struct]()
      .setType(MethodDescriptor.MethodType.UNARY)
      .setFullMethodName(MethodDescriptor.generateFullMethodName(serviceName, name))
      .setRequestMarshaller(ProtoUtils.marshaller(Struct.getDefaultInstance))
      .setResponseMarshaller(ProtoUtils.marshaller(Struct.getDefaultInstance))
      .build()

  private val handleControl = new ServerCalls.UnaryMethod[Struct, Struct] {
    override def invoke(request: Struct, responseObserver: StreamObserver[Struct]): Unit = {
      val action = readString(request, Seq("action", "mode", "value"), "start")
      val status = engine.control(action)
      responseObserver.onNext(structFromMap(controlResponse(status)))
      responseObserver.onCompleted()
    }
  }

  private val handleStatus = new ServerCalls.UnaryMethod[Struct, Struct] {
    override def invoke(request: Struct, responseObserver: StreamObserver[Struct]): Unit = {
      responseObserver.onNext(structFromMap(controlResponse(engine.status)))
      responseObserver.onCompleted()
    }
  }

  private val handleIngest = new ServerCalls.UnaryMethod[Struct, Struct] {
    override def invoke(request: Struct, responseObserver: StreamObserver[Struct]): Unit = {
      val topic = readString(request, Seq("topic"), "engine.ingest")
      val source = readString(request, Seq("source"), "kogi.network.engine")
      val target = readString(request, Seq("target"), "kogi.engine")
      val flowId = readString(request, Seq("flow_id", "flow-id", "flowId"), s"flow-${safeId(topic)}")
      val timestampMs = readLong(request, Seq("timestamp_ms", "timestamp-ms", "timestampMs"), System.currentTimeMillis())
      val payload = readPayload(request)

      val snapshot = engine.ingestGatewayMessage(
        topic = topic,
        payload = payload,
        source = source,
        target = target,
        flowId = flowId,
        timestampMs = timestampMs
      )

      responseObserver.onNext(structFromMap(snapshotResponse(snapshot)))
      responseObserver.onCompleted()
    }
  }

  private val handleSnapshot = new ServerCalls.UnaryMethod[Struct, Struct] {
    override def invoke(request: Struct, responseObserver: StreamObserver[Struct]): Unit = {
      val hostId = readString(request, Seq("host_id", "host-id", "hostId"), "kogi-host-001")
      val windowMs = readLong(request, Seq("window_ms", "window-ms", "windowMs"), 5L * 60L * 1000L)
      val snapshot = engine.snapshot(hostId = hostId, windowMs = windowMs)
      responseObserver.onNext(structFromMap(snapshotResponse(snapshot)))
      responseObserver.onCompleted()
    }
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

  private def safeId(input: String): String =
    input.toLowerCase.replaceAll("[^a-z0-9]+", "-")
}

object EngineGrpcCodec {
  def structFromMap(data: Map[String, Any]): Struct = {
    val builder = Struct.newBuilder()
    data.foreach { case (key, value) =>
      builder.putFields(key, valueFromAny(value))
    }
    builder.build()
  }

  private def valueFromAny(value: Any): Value = value match {
    case null =>
      Value.newBuilder().setNullValue(NullValue.NULL_VALUE).build()
    case s: String =>
      Value.newBuilder().setStringValue(s).build()
    case b: Boolean =>
      Value.newBuilder().setBoolValue(b).build()
    case i: Int =>
      Value.newBuilder().setNumberValue(i.toDouble).build()
    case l: Long =>
      Value.newBuilder().setNumberValue(l.toDouble).build()
    case d: Double =>
      Value.newBuilder().setNumberValue(d).build()
    case f: Float =>
      Value.newBuilder().setNumberValue(f.toDouble).build()
    case m: Map[_, _] =>
      val casted = m.asInstanceOf[Map[String, Any]]
      Value.newBuilder().setStructValue(structFromMap(casted)).build()
    case seq: Seq[_] =>
      val list = ListValue.newBuilder()
      seq.foreach(item => list.addValues(valueFromAny(item)))
      Value.newBuilder().setListValue(list).build()
    case other =>
      Value.newBuilder().setStringValue(other.toString).build()
  }

  def readString(request: Struct, keys: Seq[String], default: String): String = {
    keys
      .iterator
      .flatMap { key => Option(request.getFieldsMap.get(key)).map(valueToString) }
      .find(_.nonEmpty)
      .getOrElse(default)
  }

  def readLong(request: Struct, keys: Seq[String], default: Long): Long = {
    keys
      .iterator
      .flatMap { key => Option(request.getFieldsMap.get(key)).map(valueToLong) }
      .find(_ != Long.MinValue)
      .getOrElse(default)
  }

  def readPayload(request: Struct): Map[String, String] = {
    val value = request.getFieldsMap.get("payload")
    if (value == null) {
      Map.empty
    } else if (value.hasStructValue) {
      value
        .getStructValue
        .getFieldsMap
        .asScala
        .view
        .mapValues(valueToString)
        .toMap
    } else {
      Map("raw" -> valueToString(value))
    }
  }

  private def valueToString(value: Value): String =
    if (value.hasStringValue) value.getStringValue
    else if (value.hasNumberValue) value.getNumberValue.toString
    else if (value.hasBoolValue) value.getBoolValue.toString
    else if (value.hasStructValue) value.getStructValue.toString
    else if (value.hasListValue) value.getListValue.toString
    else ""

  private def valueToLong(value: Value): Long =
    if (value.hasNumberValue) value.getNumberValue.toLong
    else if (value.hasStringValue) value.getStringValue.toLongOption.getOrElse(Long.MinValue)
    else Long.MinValue
}
