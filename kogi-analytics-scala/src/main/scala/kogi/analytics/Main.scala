package kogi.analytics

object Main {
  def main(args: Array[String]): Unit = {
    val sample = PortfolioSignal(
      productivity = 72.0,
      cashFlow = 61.0,
      collaboration = 84.0,
      risk = 38.0
    )

    val score = PortfolioHealthPipeline.score(sample)

    println(s"kogi-analytics status=${score.status} overall=${score.overall}%.2f")
    score.recommendations.foreach(r => println(s"- $r"))
  }
}