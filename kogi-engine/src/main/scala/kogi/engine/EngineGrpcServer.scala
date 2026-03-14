package kogi.engine

import com.google.protobuf.{DescriptorProtos, Descriptors, ListValue, NullValue, Struct, StructProto, Value}
import io.grpc.{BindableService, MethodDescriptor, ServerBuilder, ServerServiceDefinition, ServiceDescriptor}
import io.grpc.protobuf.{ProtoFileDescriptorSupplier, ProtoMethodDescriptorSupplier, ProtoServiceDescriptorSupplier, ProtoUtils}
import io.grpc.protobuf.services.ProtoReflectionService
import io.grpc.stub.{ServerCalls, StreamObserver}

import java.nio.file.{Files, Paths}
import scala.jdk.CollectionConverters._

final case class EngineGrpcConfig(
    port: Int,
    reflectionEnabled: Boolean,
    protoPath: Option[String]
)

object EngineGrpcConfig {
  def fromEnv(): EngineGrpcConfig = {
    val port = sys.env.get("KOGI_ENGINE_GRPC_PORT").flatMap(_.toIntOption).getOrElse(9100)
    val reflectionEnabled = sys.env
      .get("KOGI_ENGINE_GRPC_REFLECTION")
      .map(_.trim.toLowerCase)
      .forall {
        case "" => true
        case "0" | "false" | "off" | "disabled" => false
        case _ => true
      }
    val protoPath = sys.env.get("KOGI_ENGINE_GRPC_PROTO").map(_.trim).filter(_.nonEmpty)
    EngineGrpcConfig(port, reflectionEnabled, protoPath)
  }
}

object EngineGrpcServer {
  def main(args: Array[String]): Unit = {
    val config = EngineGrpcConfig.fromEnv()
    val engine = new KogiEngine()
    val service = new EngineGrpcService(engine, config)

    val builder = ServerBuilder
      .forPort(config.port)
      .addService(service)

    if (config.reflectionEnabled) {
      builder.addService(ProtoReflectionService.newInstance())
    }

    val server = builder.build().start()

    println(s"kogi-engine grpc listening on 0.0.0.0:${config.port}")
    println(s"reflection=${if (config.reflectionEnabled) "enabled" else "disabled"}")
    config.protoPath.foreach { path =>
      val exists = Files.exists(Paths.get(path))
      println(s"proto=$path (exists=$exists)")
    }

    sys.addShutdownHook {
      server.shutdown()
    }

    server.awaitTermination()
  }
}

final class EngineGrpcService(engine: KogiEngine, config: EngineGrpcConfig) extends BindableService {
  import EngineGrpcCodec._

  private val serviceName = "kogi.engine.v1.EngineService"
  private val actions: Seq[String] = Seq(
    "control",
    "status",
    "ingest",
    "ingest-envelope",
    "snapshot",
    "flow-ledger",
    "flow-envelopes",
    "analytics-module-catalog",
    "analytics-ingest-event",
    "analytics-ingest-batch",
    "analytics-ingest-module-metric",
    "analytics-ingest-host-metric",
    "analytics-ingest-realtime",
    "analytics-ingest-bus",
    "analytics-snapshot-profile",
    "analytics-module-activity",
    "analytics-module-snapshot",
    "analytics-host-snapshot",
    "analytics-system-snapshot",
    "stream-ingest",
    "stream-ingest-batch",
    "stream-recent",
    "stream-all",
    "stream-by-profile",
    "stream-by-module",
    "stream-since",
    "stream-by-module-since",
    "stream-size",
    "rec-upsert-item",
    "rec-upsert-items",
    "rec-upsert-profile",
    "rec-record-interaction",
    "rec-record-interactions",
    "rec-record-rating",
    "rec-record-review",
    "rec-record-dwell",
    "rec-record-search",
    "rec-build-persona",
    "rec-profile",
    "rec-personas",
    "rec-hybrid",
    "rec-collab",
    "rec-content",
    "rec-contextual",
    "rec-cold-start",
    "rec-feedback",
    "rec-profile-summary",
    "rec-personalized-search",
    "rec-stats",
    "personalize",
    "personalization-set-preference",
    "personalization-set-preferences",
    "personalization-clear-preference",
    "personalization-preferences",
    "personalization-resolve-preferences",
    "personalization-delivery-config",
    "personalization-rank-content",
    "personalization-register-segment",
    "personalization-register-segments",
    "personalization-assign-segments",
    "personalization-segment-assignment",
    "personalization-profiles-in-segment",
    "personalization-build-persona",
    "personalization-detect-drift",
    "personalization-apply-drift",
    "personalization-register-experiment",
    "personalization-assign-experiment",
    "personalization-record-exposure",
    "personalization-experiment-stats",
    "personalization-experiment-assignments",
    "personalization-register-bandit",
    "personalization-select-bandit",
    "personalization-record-bandit",
    "personalization-bandit-stats",
    "personalization-adapt-content",
    "personalization-record-interaction",
    "personalization-record-interactions",
    "personalization-register-item",
    "personalization-recommendations",
    "search",
    "index",
    "search-index-batch",
    "search-remove",
    "search-filter",
    "search-personalized",
    "query",
    "query-analyze",
    "query-optimize",
    "query-optimize-profile",
    "risk-score",
    "risk-score-persona",
    "risk-optimize",
    "risk-manage",
    "optimize",
    "policy-register-simple",
    "policy-unregister",
    "policy-list",
    "policy-clear",
    "policy-evaluate",
    "policy-evaluate-report",
    "policy-request-approval",
    "policy-resolve-approval",
    "policy-approval",
    "policy-approvals",
    "match-register-user",
    "match-register-users",
    "match-register-component",
    "match-register-components",
    "match-register-resource",
    "match-register-resources",
    "match-register-asset",
    "match-register-assets",
    "match-counts",
    "match-user-to-components",
    "match-component-to-users",
    "match-assets-to-component",
    "match-components-to-asset",
    "match-resources-to-component",
    "match-components-to-resource",
    "match-profiles-resources",
    "match-talent",
    "match-by-persona",
    "match-similar-users",
    "match-workloads-to-plans",
    "match-artifacts-to-user",
    "match-component-bundle",
    "match-exchange-listings",
    "allocation-score",
    "allocation-allocate",
    "incentive-register-participant",
    "incentive-profile",
    "incentive-balance",
    "incentive-apply-event",
    "incentive-apply-incentive",
    "incentive-apply-incentives",
    "incentive-earn",
    "incentive-penalize",
    "incentive-redeem",
    "incentive-incentives-for-allocation",
    "incentive-ledger",
    "game-register-participant",
    "game-register-participants",
    "game-participant",
    "game-participants",
    "game-upsert-listing",
    "game-listing",
    "game-close-listing",
    "game-suspend-listing",
    "game-listings",
    "game-submit-bid",
    "game-withdraw-bid",
    "game-bids",
    "game-match-listing",
    "game-score-bids",
    "game-allocate-listing",
    "game-award-incentives",
    "game-snapshot",
    "game-events",
    "graph-add-edge",
    "graph-add-node",
    "graph-remove-edge",
    "graph-remove-node",
    "graph-load",
    "graph-traverse",
    "graph-rtraverse",
    "graph-impact",
    "graph-closure",
    "graph-rdeps",
    "graph-neighbors",
    "graph-ancestors",
    "graph-descendants",
    "graph-cycles",
    "graph-topo",
    "graph-critical",
    "graph-diff",
    "graph-nodes",
    "graph-edges"
  )

  private val methodActions: Seq[(String, String)] =
    actions.map(action => actionToMethod(action) -> action)

  private val fileDescriptor =
    EngineGrpcDescriptors.fileDescriptor(config.protoPath, methodActions.map(_._1))

  private val methodDescriptors: Seq[(MethodDescriptor[Struct, Struct], String)] =
    methodActions.map { case (methodName, action) => unaryMethod(methodName) -> action }

  private val serviceDescriptor: ServiceDescriptor =
    methodDescriptors.foldLeft(
      ServiceDescriptor
        .newBuilder(serviceName)
        .setSchemaDescriptor(new EngineProtoSchemaDescriptor(fileDescriptor))
    ) { case (builder, (method, _)) => builder.addMethod(method) }
      .build()

  override def bindService(): ServerServiceDefinition = {
    methodDescriptors.foldLeft(ServerServiceDefinition.builder(serviceDescriptor)) {
      case (builder, (method, action)) =>
        builder.addMethod(method, ServerCalls.asyncUnaryCall(handleAction(action)))
    }.build()
  }

  private def unaryMethod(name: String): MethodDescriptor[Struct, Struct] =
    MethodDescriptor
      .newBuilder[Struct, Struct]()
      .setType(MethodDescriptor.MethodType.UNARY)
      .setFullMethodName(MethodDescriptor.generateFullMethodName(serviceName, name))
      .setSchemaDescriptor(new EngineProtoMethodDescriptorSupplier(fileDescriptor, name))
      .setRequestMarshaller(ProtoUtils.marshaller(Struct.getDefaultInstance))
      .setResponseMarshaller(ProtoUtils.marshaller(Struct.getDefaultInstance))
      .build()

  private def handleAction(action: String): ServerCalls.UnaryMethod[Struct, Struct] =
    new ServerCalls.UnaryMethod[Struct, Struct] {
      override def invoke(request: Struct, responseObserver: StreamObserver[Struct]): Unit = {
        val requestMap = structToMap(request)
        val response = EngineActionDispatcher.handle(engine, action, requestMap)
        responseObserver.onNext(structFromMap(response))
        responseObserver.onCompleted()
      }
    }

  private def actionToMethod(action: String): String =
    action
      .split("[-_]")
      .toList
      .filter(_.nonEmpty)
      .map(part => part.substring(0, 1).toUpperCase + part.substring(1))
      .mkString
}

object EngineGrpcDescriptors {
  private val packageName = "kogi.engine.v1"
  private val serviceSimpleName = "EngineService"

  def fileDescriptor(protoPath: Option[String], methodNames: Seq[String]): Descriptors.FileDescriptor = {
    val fileName = protoPath.flatMap(extractFileName).getOrElse("kogi_engine.proto")
    val serviceBuilder = DescriptorProtos.ServiceDescriptorProto
      .newBuilder()
      .setName(serviceSimpleName)

    methodNames.foreach(name => serviceBuilder.addMethod(method(name)))

    val fileProto = DescriptorProtos.FileDescriptorProto
      .newBuilder()
      .setSyntax("proto3")
      .setName(fileName)
      .setPackage(packageName)
      .addDependency("google/protobuf/struct.proto")
      .addService(serviceBuilder)
      .build()

    Descriptors.FileDescriptor.buildFrom(fileProto, Array(StructProto.getDescriptor))
  }

  private def method(name: String): DescriptorProtos.MethodDescriptorProto =
    DescriptorProtos.MethodDescriptorProto
      .newBuilder()
      .setName(name)
      .setInputType(".google.protobuf.Struct")
      .setOutputType(".google.protobuf.Struct")
      .build()

  private def extractFileName(path: String): Option[String] =
    try Option(Paths.get(path).getFileName).map(_.toString)
    catch { case _: Exception => None }
}

final class EngineProtoSchemaDescriptor(fileDescriptor: Descriptors.FileDescriptor)
    extends ProtoFileDescriptorSupplier
    with ProtoServiceDescriptorSupplier {
  override def getFileDescriptor: Descriptors.FileDescriptor = fileDescriptor
  override def getServiceDescriptor: Descriptors.ServiceDescriptor =
    fileDescriptor.findServiceByName("EngineService")
}

final class EngineProtoMethodDescriptorSupplier(
    fileDescriptor: Descriptors.FileDescriptor,
    methodName: String
) extends ProtoMethodDescriptorSupplier {
  override def getFileDescriptor: Descriptors.FileDescriptor =
    fileDescriptor

  override def getServiceDescriptor: Descriptors.ServiceDescriptor =
    fileDescriptor.findServiceByName("EngineService")

  override def getMethodDescriptor: Descriptors.MethodDescriptor = {
    val service = getServiceDescriptor
    service.findMethodByName(methodName)
  }
}

object EngineGrpcCodec {
  def structFromMap(data: Map[String, Any]): Struct = {
    val builder = Struct.newBuilder()
    data.foreach { case (key, value) =>
      builder.putFields(key, valueFromAny(value))
    }
    builder.build()
  }

  def structToMap(struct: Struct): Map[String, Any] =
    struct.getFieldsMap.asScala.map { case (k, v) => k -> valueToAny(v) }.toMap

  private def valueFromAny(value: Any): Value = {
    EngineValueCodec.normalize(value) match {
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
      case seq: Iterable[_] =>
        val list = ListValue.newBuilder()
        seq.foreach(item => list.addValues(valueFromAny(item)))
        Value.newBuilder().setListValue(list).build()
      case other =>
        Value.newBuilder().setStringValue(other.toString).build()
    }
  }

  private def valueToAny(value: Value): Any =
    if (value == null) null
    else if (value.hasNullValue) null
    else if (value.hasStringValue) value.getStringValue
    else if (value.hasBoolValue) value.getBoolValue
    else if (value.hasNumberValue) value.getNumberValue
    else if (value.hasStructValue) structToMap(value.getStructValue)
    else if (value.hasListValue)
      value.getListValue.getValuesList.asScala.map(valueToAny).toList
    else null
}
