ThisBuild / scalaVersion := "2.13.14"
ThisBuild / version := "0.1.0"
ThisBuild / organization := "com.kogi"

lazy val root = (project in file("."))
  .settings(
    name := "kogi-engine",
    Compile / mainClass := Some("kogi.engine.Main")
  )
