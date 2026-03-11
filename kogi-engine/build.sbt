ThisBuild / scalaVersion := "2.13.14"
ThisBuild / version := "0.1.0"
ThisBuild / organization := "com.kogi"

lazy val root = (project in file("."))
  .settings(
    name := "kogi-engine",
    Compile / mainClass := Some("kogi.engine.Main"),
    libraryDependencies ++= Seq(
      "io.grpc" % "grpc-netty-shaded" % "1.65.0",
      "io.grpc" % "grpc-protobuf" % "1.65.0",
      "io.grpc" % "grpc-stub" % "1.65.0",
      "io.grpc" % "grpc-services" % "1.65.0",
      "com.google.protobuf" % "protobuf-java" % "3.25.3"
    )
  )
