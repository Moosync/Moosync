namespace MusicBrainz {
  interface ArtistSearchResponse {
    created: string
    count: number
    offset: number
    artists: Artist[]
  }

  interface Artist {
    id: string
    type: string
    'type-id': string
    score: number
    name: string
    'sort-name': string
    country: string
    area: Area
    'begin-area': BeginArea
    isnis: string[]
    'life-span': LifeSpan
    aliases: Alias[]
    tags: Tag[]
  }

  interface Area {
    id: string
    type: string
    'type-id': string
    name: string
    'sort-name': string
    'life-span': LifeSpan
  }

  interface BeginArea {
    id: string
    type: string
    'type-id': string
    name: string
    'sort-name': string
    'life-span': LifeSpan2
  }

  interface LifeSpan {
    begin: string
    ended: string
  }

  interface Alias {
    'sort-name': string
    'type-id': string
    name: string
    locale: string
    type: string
    primary?: boolean
    'begin-date'?: string
    'end-date': string
  }

  interface Tag {
    count: number
    name: string
  }

  interface ArtistInfo {
    'begin-area': BeginArea
    disambiguation: string
    end_area: string
    ipis: string[]
    country: string
    gender: unknown
    'life-span': LifeSpan
    name: string
    begin_area: BeginArea2
    relations?: Relation[]
    id: string
    'sort-name': string
    type: string
    area: Area
    'type-id': string
    'end-area': string
    'gender-id': string
    isnis: string[]
  }

  interface Relation {
    end?: string
    url: Url
    'target-type': string
    type: 'image'
    'attribute-ids': AttributeIds
    direction: string
    'type-id': string
    begin: string
    ended: boolean
    'attribute-values': AttributeValues
    'source-credit': string
    attributes: string[]
    'target-credit': string
  }

  interface Url {
    resource: string
    id: string
  }
}

namespace Wikimedia {
  interface FileNameQuery {
    batchcomplete: string
    query: Query
  }

  interface Query {
    normalized: Normalized[]
    pages: Pages
  }

  interface Normalized {
    from: string
    to: string
  }

  type Pages = Record<string, N1739929>

  interface N1739929 {
    pageid: number
    ns: number
    title: string
  }
}
