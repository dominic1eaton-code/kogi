package kogi.engine

final case class SearchDocument(
    id: String,
    title: String,
    body: String,
    tags: List[String] = Nil,
    metadata: Map[String, String] = Map.empty
)

final case class SearchQuery(
    text: String,
    tags: List[String] = Nil,
    metadata: Map[String, String] = Map.empty,
    limit: Int = 10
)

final case class SearchMatch(
    document: SearchDocument,
    score: Double,
    highlights: List[String]
)

final case class SearchResult(
    query: SearchQuery,
    total: Int,
    matches: List[SearchMatch],
    generatedAtMs: Long
)

final case class PersonalizedSearchQuery(
    query: SearchQuery,
    userId: String,
    preferenceVector: Map[String, Double] = Map.empty,
    dominantCategories: List[String] = Nil
)

final class SearchEngine(maxDocuments: Int = 50000) {
  private var documents: Vector[SearchDocument] = Vector.empty

  def index(document: SearchDocument): Unit = {
    documents = (documents.filterNot(_.id == document.id) :+ document).takeRight(maxDocuments)
  }

  def indexBatch(batch: Seq[SearchDocument]): Unit = batch.foreach(index)

  def remove(id: String): Unit = {
    documents = documents.filterNot(_.id == id)
  }

  def search(query: SearchQuery): SearchResult = {
    val tokens   = tokenize(query.text)
    val filtered = filterByTagsAndMetadata(query)

    val scored = filtered.flatMap { doc =>
      val text = s"${doc.title} ${doc.body} ${doc.tags.mkString(" ")}".toLowerCase
      val hits = tokens.count(token => text.contains(token))
      if (hits == 0 && tokens.nonEmpty) None
      else {
        val highlights = tokens.filter(token => text.contains(token)).distinct
        Some(SearchMatch(
          document = doc,
          score = hits.toDouble + (doc.tags.intersect(query.tags).size * 0.5),
          highlights = highlights
        ))
      }
    }

    val sorted = scored.sortBy(m => -m.score)
    SearchResult(
      query = query,
      total = sorted.size,
      matches = sorted.take(query.limit.max(0)).toList,
      generatedAtMs = System.currentTimeMillis()
    )
  }

  /**
   * Personalized search: re-rank results using the user's preference vector
   * and dominant categories so items aligned with their profile score higher.
   */
  def personalizedSearch(pq: PersonalizedSearchQuery): SearchResult = {
    val base = search(pq.query)

    val reranked = base.matches.map { m =>
      val doc = m.document

      // Boost: preference vector match on tags
      val tagBoost = doc.tags.foldLeft(0.0) { (acc, tag) =>
        acc + pq.preferenceVector.getOrElse(tag, 0.0).max(0.0)
      }

      // Boost: dominant category match
      val catKey = doc.metadata.getOrElse("category", "")
      val catBoost = if (pq.dominantCategories.contains(catKey)) 1.5 else 0.0

      m.copy(score = m.score + tagBoost * 0.4 + catBoost)
    }.sortBy(-_.score)

    base.copy(matches = reranked.take(pq.query.limit.max(0)))
  }

  def filter(query: SearchQuery): List[SearchDocument] =
    filterByTagsAndMetadata(query).toList

  private def filterByTagsAndMetadata(query: SearchQuery): Vector[SearchDocument] =
    documents.filter { doc =>
      val tagsOk =
        if (query.tags.isEmpty) true
        else query.tags.forall(tag => doc.tags.contains(tag))
      val metadataOk =
        if (query.metadata.isEmpty) true
        else query.metadata.forall { case (k, v) => doc.metadata.get(k).contains(v) }
      tagsOk && metadataOk
    }

  private def tokenize(input: String): List[String] =
    input.toLowerCase.split("\\s+").toList.filter(_.nonEmpty)
}
