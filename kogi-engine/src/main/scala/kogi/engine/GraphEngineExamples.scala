package portfolio.graph.examples

import portfolio.graph._

// =============================================================
//  GraphEngine – Real-World Usage Examples
//
//  1.  CI/CD Pipeline Scheduler
//  2.  Microservice Dependency Map
//  3.  Package Manager (like npm / sbt)
//  4.  Project Management (tasks & milestones)
//  5.  Database Schema Migration Planner
//  6.  Feature Flag / Config Dependency Tracker
//  7.  Infrastructure-as-Code Change Impact
// =============================================================


// -------------------------------------------------------------
// 1. CI/CD Pipeline Scheduler
//
//  Problem: jobs in a pipeline have prerequisites;
//  find the correct execution order and the slowest path
//  (the one that determines total build time).
// -------------------------------------------------------------

object CICDPipelineExample extends App {

  // Each node is a job; duration is estimated runtime in minutes
  val jobs = Seq(
    GraphNode("checkout",       duration = 1),
    GraphNode("install-deps",   duration = 3),
    GraphNode("lint",           duration = 2),
    GraphNode("unit-tests",     duration = 5),
    GraphNode("build",          duration = 4),
    GraphNode("docker-build",   duration = 6),
    GraphNode("integration",    duration = 8),
    GraphNode("security-scan",  duration = 3),
    GraphNode("deploy-staging", duration = 2),
    GraphNode("smoke-tests",    duration = 3),
    GraphNode("deploy-prod",    duration = 2)
  )

  // Edge weight = hand-off / queue overhead in minutes
  val pipeline = Seq(
    GraphEdge("checkout",       "install-deps",   Dependency, weight = 0),
    GraphEdge("install-deps",   "lint",           Dependency, weight = 0),
    GraphEdge("install-deps",   "unit-tests",     Dependency, weight = 0),
    GraphEdge("install-deps",   "build",          Dependency, weight = 0),
    GraphEdge("lint",           "build",          Dependency, weight = 0),
    GraphEdge("unit-tests",     "build",          Dependency, weight = 0),
    GraphEdge("build",          "docker-build",   Dependency, weight = 1),
    GraphEdge("build",          "security-scan",  Dependency, weight = 0),
    GraphEdge("docker-build",   "integration",    Dependency, weight = 0),
    GraphEdge("integration",    "deploy-staging", Dependency, weight = 0),
    GraphEdge("security-scan",  "deploy-staging", Dependency, weight = 0),
    GraphEdge("deploy-staging", "smoke-tests",    Dependency, weight = 0),
    GraphEdge("smoke-tests",    "deploy-prod",    Dependency, weight = 1)
  )

  val engine = GraphEngine(pipeline, jobs)

  // --- What order should the CI runner execute jobs? ---
  engine.topologicalSort() match {
    case Right(order) =>
      println("=== CI Job Execution Order ===")
      order.zipWithIndex.foreach { case (job, i) => println(s"  ${i + 1}. $job") }
    case Left(cycles) =>
      println(s"Pipeline has circular dependencies: $cycles")
  }

  // --- What is the minimum possible build time? ---
  engine.criticalPath() match {
    case Some(report) =>
      println(s"\n=== Critical Path (slowest lane) ===")
      println(s"  Path: ${report.path.mkString(" → ")}")
      println(s"  Minimum pipeline duration: ${report.totalCost} minutes")
      println("\n  Jobs that can be parallelised freely (high slack):")
      report.nodeSlack
        .filter(_._2 > 2.0)
        .toSeq.sortBy(-_._2)
        .foreach { case (job, slack) => println(s"    $job  (${slack}m of slack)") }
    case None => println("Cycle detected – cannot compute critical path")
  }

  // --- If 'build' breaks, what else is blocked? ---
  val impact = engine.impactAnalysis("build")
  println(s"\n=== If 'build' fails ===")
  println(s"  Blocked jobs   : ${impact.affected}")
  println(s"  Upstream causes: ${impact.dependents}")
}


// -------------------------------------------------------------
// 2. Microservice Dependency Map
//
//  Problem: before taking a service down for maintenance,
//  know exactly which other services will be affected.
// -------------------------------------------------------------

object MicroserviceMapExample extends App {

  // Services and their call dependencies
  val services = Seq(
    GraphEdge("api-gateway",     "auth-service",     Dependency),
    GraphEdge("api-gateway",     "user-service",     Dependency),
    GraphEdge("api-gateway",     "product-service",  Dependency),
    GraphEdge("user-service",    "auth-service",     Dependency),
    GraphEdge("user-service",    "notification-svc", Dependency),
    GraphEdge("product-service", "inventory-svc",    Dependency),
    GraphEdge("product-service", "pricing-svc",      Dependency),
    GraphEdge("order-service",   "product-service",  Dependency),
    GraphEdge("order-service",   "user-service",     Dependency),
    GraphEdge("order-service",   "payment-svc",      Dependency),
    GraphEdge("payment-svc",     "fraud-detection",  Dependency),
    // Ownership hierarchy (team → service)
    GraphEdge("team-platform",   "api-gateway",      Hierarchy),
    GraphEdge("team-platform",   "auth-service",     Hierarchy),
    GraphEdge("team-commerce",   "order-service",    Hierarchy),
    GraphEdge("team-commerce",   "payment-svc",      Hierarchy),
    GraphEdge("team-catalog",    "product-service",  Hierarchy),
    GraphEdge("team-catalog",    "inventory-svc",    Hierarchy)
  )

  val engine = GraphEngine(services)

  // --- Maintenance window planning ---
  val target = "auth-service"
  val impact = engine.impactAnalysis(target)
  println(s"=== Maintenance impact of taking down '$target' ===")
  println(s"  Services that WILL BREAK : ${impact.dependents}")
  println(s"  Services '$target' relies on: ${impact.affected}")

  // --- Find all services owned by a team ---
  val teamServices = engine.descendants("team-commerce")
  println(s"\n=== Services owned by team-commerce ===")
  println(s"  ${teamServices}")

  // --- Full blast radius for a cascading outage from 'product-service' ---
  val blast = engine.neighborhood("product-service")
  println(s"\n=== Full neighbourhood of product-service ===")
  println(s"  ${blast}")
}


// -------------------------------------------------------------
// 3. Package Manager (like npm / sbt)
//
//  Problem: resolve install order for packages,
//  detect version conflicts (cycles), and show
//  what a version bump of a low-level library breaks.
// -------------------------------------------------------------

object PackageManagerExample extends App {

  val packages = Seq(
    GraphNode("lodash@4.17",   duration = 0),
    GraphNode("axios@1.6",     duration = 0),
    GraphNode("react@18",      duration = 0),
    GraphNode("react-dom@18",  duration = 0),
    GraphNode("react-query@5", duration = 0),
    GraphNode("my-app",        duration = 0)
  )

  val deps = Seq(
    GraphEdge("my-app",        "react@18",      Dependency),
    GraphEdge("my-app",        "react-query@5", Dependency),
    GraphEdge("my-app",        "axios@1.6",     Dependency),
    GraphEdge("react-dom@18",  "react@18",      Dependency),
    GraphEdge("react-query@5", "react@18",      Dependency),
    GraphEdge("react-query@5", "axios@1.6",     Dependency),
    GraphEdge("axios@1.6",     "lodash@4.17",   Dependency)
  )

  val engine = GraphEngine(deps, packages)

  // --- Safe install order (leaves first) ---
  engine.topologicalSort() match {
    case Right(order) =>
      println("=== Package Install Order ===")
      order.filter(_ != "my-app")  // exclude app root
        .zipWithIndex.foreach { case (pkg, i) => println(s"  ${i + 1}. $pkg") }
    case Left(bad) =>
      println(s"Circular dependency detected in: $bad")
  }

  // --- If lodash is patched to a breaking version ---
  val bump = engine.impactAnalysis("lodash@4.17")
  println(s"\n=== Packages to re-test if lodash is upgraded ===")
  println(s"  ${bump.dependents}")

  // --- Detect accidental circular deps (common in monorepos) ---
  val withCircle = deps :+
    GraphEdge("axios@1.6", "react-query@5", Dependency)  // ← broken

  val circleEngine = GraphEngine(withCircle, packages)
  val cycleReport  = circleEngine.detectCycles()
  if (cycleReport.hasCycles) {
    println(s"\n=== Circular dependency found! ===")
    cycleReport.cycles.foreach(c => println(s"  ${c.mkString(" → ")}"))
  }
}


// -------------------------------------------------------------
// 4. Project Management (tasks & milestones)
//
//  Problem: plan a product launch – find the critical path,
//  see which tasks have float, and detect if a new requirement
//  creates a circular dependency.
// -------------------------------------------------------------

object ProjectManagementExample extends App {

  val tasks = Seq(
    GraphNode("requirements",     duration = 5),
    GraphNode("ux-design",        duration = 8),
    GraphNode("backend-api",      duration = 10),
    GraphNode("frontend",         duration = 7),
    GraphNode("db-schema",        duration = 3),
    GraphNode("integration",      duration = 4),
    GraphNode("qa-testing",       duration = 6),
    GraphNode("security-audit",   duration = 3),
    GraphNode("staging-deploy",   duration = 1),
    GraphNode("launch",           duration = 1)
  )

  val taskEdges = Seq(
    GraphEdge("requirements",   "ux-design",      Dependency, weight = 1),
    GraphEdge("requirements",   "db-schema",      Dependency, weight = 1),
    GraphEdge("ux-design",      "frontend",       Dependency, weight = 1),
    GraphEdge("db-schema",      "backend-api",    Dependency, weight = 1),
    GraphEdge("backend-api",    "integration",    Dependency, weight = 1),
    GraphEdge("frontend",       "integration",    Dependency, weight = 1),
    GraphEdge("integration",    "qa-testing",     Dependency, weight = 1),
    GraphEdge("integration",    "security-audit", Dependency, weight = 1),
    GraphEdge("qa-testing",     "staging-deploy", Dependency, weight = 1),
    GraphEdge("security-audit", "staging-deploy", Dependency, weight = 1),
    GraphEdge("staging-deploy", "launch",         Dependency, weight = 1)
  )

  val engine = GraphEngine(taskEdges, tasks)

  engine.criticalPath() match {
    case Some(report) =>
      println(s"=== Project Critical Path ===")
      println(s"  ${report.path.mkString(" → ")}")
      println(s"  Total project duration: ${report.totalCost} days")

      println(s"\n=== Task Float (days available to slip without delaying launch) ===")
      report.nodeSlack.toSeq.sortBy(_._2).foreach { case (task, slack) =>
        val marker = if (slack < 0.1) " ◄ CRITICAL" else ""
        println(f"  $task%-20s  ${slack}%.1f days$marker")
      }
    case None => println("Cycle detected")
  }

  // --- Simulate a late design change: 'backend-api' now also needs 'ux-design' ---
  val revisedEdges = taskEdges :+ GraphEdge("ux-design", "backend-api", Dependency, weight = 1)
  val revised      = GraphEngine(revisedEdges, tasks)
  revised.criticalPath().foreach { r =>
    println(s"\n=== After design-change: new critical path ===")
    println(s"  ${r.path.mkString(" → ")}")
    println(s"  New duration: ${r.totalCost} days")
  }
}


// -------------------------------------------------------------
// 5. Database Schema Migration Planner
//
//  Problem: migrations must run in the right order;
//  a dropped column must be safe before the migration fires.
// -------------------------------------------------------------

object SchemaMigrationExample extends App {

  val migrations = Seq(
    GraphNode("001_create_users",          duration = 1),
    GraphNode("002_create_accounts",       duration = 1),
    GraphNode("003_add_user_fk",           duration = 1),
    GraphNode("004_create_orders",         duration = 1),
    GraphNode("005_add_account_fk",        duration = 1),
    GraphNode("006_add_order_items",       duration = 1),
    GraphNode("007_drop_legacy_col",       duration = 1),
    GraphNode("008_add_audit_log",         duration = 1)
  )

  val migrationDeps = Seq(
    GraphEdge("001_create_users",    "002_create_accounts", Dependency),
    GraphEdge("001_create_users",    "003_add_user_fk",     Dependency),
    GraphEdge("002_create_accounts", "003_add_user_fk",     Dependency),
    GraphEdge("003_add_user_fk",     "004_create_orders",   Dependency),
    GraphEdge("002_create_accounts", "005_add_account_fk",  Dependency),
    GraphEdge("004_create_orders",   "005_add_account_fk",  Dependency),
    GraphEdge("005_add_account_fk",  "006_add_order_items", Dependency),
    GraphEdge("004_create_orders",   "007_drop_legacy_col", Dependency),
    GraphEdge("006_add_order_items", "008_add_audit_log",   Dependency)
  )

  val engine = GraphEngine(migrationDeps, migrations)

  engine.topologicalSort() match {
    case Right(order) =>
      println("=== Safe Migration Execution Order ===")
      order.zipWithIndex.foreach { case (m, i) => println(s"  ${i + 1}. $m") }
    case Left(bad) =>
      println(s"Cannot plan migrations – circular dependency in: $bad")
  }

  // --- What breaks if we roll back migration 003? ---
  val rollbackImpact = engine.impactAnalysis("003_add_user_fk")
  println(s"\n=== Rollback impact of 003_add_user_fk ===")
  println(s"  Must also roll back: ${rollbackImpact.affected}")
}


// -------------------------------------------------------------
// 6. Feature Flag / Config Dependency Tracker
//
//  Problem: feature flags reference each other (flag B is only
//  active when flag A is on). Detect conflicts and scheduling.
// -------------------------------------------------------------

object FeatureFlagExample extends App {

  val flags = Seq(
    GraphEdge("new-checkout",   "new-payment-ui",    Dependency),
    GraphEdge("new-checkout",   "loyalty-points",    Dependency),
    GraphEdge("new-payment-ui", "stripe-v3",         Dependency),
    GraphEdge("loyalty-points", "user-profiles-v2",  Dependency),
    GraphEdge("user-profiles-v2", "new-auth",        Dependency),
    GraphEdge("beta-dashboard", "user-profiles-v2",  Dependency),
    GraphEdge("beta-dashboard", "analytics-v2",      Dependency)
  )

  val engine = GraphEngine(flags)

  // --- Which flags must be enabled first before we can turn on 'new-checkout'? ---
  val prereqs = engine.dependencyClosure("new-checkout")
  println(s"=== Prerequisites for 'new-checkout' ===")
  println(s"  Must be ON first: $prereqs")

  // --- Which flags does enabling 'new-auth' affect? ---
  val impact = engine.reverseDependencyClosure("new-auth")
  println(s"\n=== Flags that depend on 'new-auth' (directly or transitively) ===")
  println(s"  $impact")

  // Safe rollout order
  engine.topologicalSort() match {
    case Right(order) =>
      println(s"\n=== Safe flag rollout order ===")
      order.zipWithIndex.foreach { case (f, i) => println(s"  ${i + 1}. $f") }
    case Left(bad) =>
      println(s"Flag conflict (circular reference): $bad")
  }
}


// -------------------------------------------------------------
// 7. Infrastructure-as-Code Change Impact (Terraform-style)
//
//  Problem: before applying a plan, show every resource that
//  will be recreated or modified as a result of a change,
//  and diff two snapshots of the infrastructure graph.
// -------------------------------------------------------------

object InfraChangeImpactExample extends App {

  def buildInfra(vpcCidr: String, dbInstance: String): GraphEngine = {
    val edges = Seq(
      GraphEdge("vpc",              "subnet-public",   Dependency),
      GraphEdge("vpc",              "subnet-private",  Dependency),
      GraphEdge("subnet-public",    "alb",             Dependency),
      GraphEdge("subnet-private",   s"rds-$dbInstance",Dependency),
      GraphEdge("alb",              "ecs-service",     Dependency),
      GraphEdge("ecs-service",      s"rds-$dbInstance",Dependency),
      GraphEdge("ecs-service",      "elasticache",     Dependency),
      GraphEdge("ecs-service",      "s3-bucket",       Dependency),
      GraphEdge("iam-role",         "ecs-service",     Dependency),
      GraphEdge("iam-role",         "s3-bucket",       Dependency),
      // Hierarchy: region owns everything
      GraphEdge("us-east-1", "vpc",      Hierarchy),
      GraphEdge("us-east-1", "iam-role", Hierarchy)
    )
    val nodes = Seq(GraphNode("vpc", duration = 0))
    GraphEngine(edges, nodes)
  }

  val currentInfra = buildInfra("10.0.0.0/16", "db.t3.medium")
  val plannedInfra = buildInfra("10.0.0.0/16", "db.t3.large")  // ← DB instance type bump

  // --- What changes between current and planned? ---
  val diff = currentInfra.diff(plannedInfra)
  println("=== Terraform Plan Diff ===")
  println(s"  Resources added  : ${diff.addedNodes}")
  println(s"  Resources removed: ${diff.removedNodes}")
  println(s"  Edges added      : ${diff.addedEdges.map(e => s"${e.from}→${e.to}")}")
  println(s"  Edges removed    : ${diff.removedEdges.map(e => s"${e.from}→${e.to}")}")
  println(s"  Edges changed    : ${diff.changedEdges.map { case (o,n) => s"${o.from}→${o.to}" }}")

  // --- Blast radius: if RDS is replaced, what must be re-deployed? ---
  val rdsImpact = currentInfra.reverseDependencyClosure("rds-db.t3.medium")
  println(s"\n=== Resources to redeploy if RDS is replaced ===")
  println(s"  ${rdsImpact}")

  // --- What does the ecs-service depend on transitively? ---
  val ecsDeps = currentInfra.dependencyClosure("ecs-service")
  println(s"\n=== All infrastructure 'ecs-service' depends on ===")
  println(s"  ${ecsDeps}")

  // --- Full region inventory (hierarchy traversal) ---
  val regionResources = currentInfra.descendants("us-east-1")
  println(s"\n=== Resources in us-east-1 (hierarchy) ===")
  println(s"  ${regionResources}")
}
