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
    val tokens = tokenize(query.text)
    val filtered = filterByTagsAndMetadata(query)

    val scored = filtered.flatMap { doc =>
      val text = s"${doc.title} ${doc.body} ${doc.tags.mkString(" ")}".toLowerCase
      val hits = tokens.count(token => text.contains(token))
      if (hits == 0 && tokens.nonEmpty) None
      else {
        val highlights = tokens.filter(token => text.contains(token)).distinct
        Some(
          SearchMatch(
            document = doc,
            score = hits.toDouble + (doc.tags.intersect(query.tags).size * 0.5),
            highlights = highlights
          )
        )
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

  def filter(query: SearchQuery): List[SearchDocument] =
    filterByTagsAndMetadata(query).toList

  private def filterByTagsAndMetadata(query: SearchQuery): Vector[SearchDocument] = {
    documents.filter { doc =>
      val tagsOk =
        if (query.tags.isEmpty) true
        else query.tags.forall(tag => doc.tags.contains(tag))

      val metadataOk =
        if (query.metadata.isEmpty) true
        else query.metadata.forall { case (key, value) =>
          doc.metadata.get(key).contains(value)
        }

      tagsOk && metadataOk
    }
  }

  private def tokenize(input: String): List[String] =
    input.toLowerCase.split("\\s+").toList.filter(_.nonEmpty)
}
