namespace FanartTv {
  interface ArtistQuery {
    name: string
    mbid_id: string
    albums: Albums
    artistthumb: Artistthumb[]
    artistbackground: Artistbackground[]
    musiclogo: Musiclogo[]
    hdmusiclogo: Hdmusiclogo[]
    musicbanner: Musicbanner[]
  }

  type Albums = Record<string, AlbumArt>

  interface AlbumArt {
    albumcover: Albumcover[]
    cdart: Cdart[]
  }

  interface Albumcover {
    id: string
    url: string
    likes: string
  }

  interface Cdart {
    id: string
    url: string
    likes: string
    disc: string
    size: string
  }

  interface Artistthumb {
    id: string
    url: string
    likes: string
  }

  interface Artistbackground {
    id: string
    url: string
    likes: string
  }

  interface Musiclogo {
    id: string
    url: string
    likes: string
  }

  interface Hdmusiclogo {
    id: string
    url: string
    likes: string
  }

  interface Musicbanner {
    id: string
    url: string
    likes: string
  }
}
