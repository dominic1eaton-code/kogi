ThisBuild / scalaVersion := "2.13.14"
ThisBuild / version := "0.1.0"
ThisBuild / organization := "com.kogi"

lazy val root = (project in file("."))
  .settings(
    name := "kogi-analytics",
    Compile / mainClass := Some("kogi.analytics.Main")
  )