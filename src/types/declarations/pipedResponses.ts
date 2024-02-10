/*
 *  youtubeResponses.d.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

declare namespace PipedResponses {
  // These are only the resources used
  enum PipedResources {
    SEARCH = 'search',
    PLAYLIST_DETAILS = 'playlists/${playlist_id}',
    PLAYLIST_DETAILS_NEXT = 'nextpage/playlists/${playlist_id}',
    CHANNEL_DETAILS = 'channel/${channel_id}',
    CHANNEL_DETAILS_NEXT = 'nextpage/channel/${channel_id}',
    STREAM_DETAILS = 'streams/${video_id}',
    LOGIN = 'login',
    USER_PLAYLISTS = 'user/playlists',
  }

  type SearchFilters = 'music_songs' | 'channels' | 'playlists' | 'music_albums'

  type PlaylistDetailsRequest = {
    playlistId: string
    nextpage?: string
  }

  type ChannelDetailsRequest = {
    channelId: string
    nextpage?: string
  }

  type SearchRequest<K extends SearchFilters> = {
    q: string
    filter: K
  }

  type StreamRequest = {
    videoId: string
  }

  type LoginRequest = {
    username: string
    password: string
  }

  interface VideoDetails {
    url: string
    title: string
    thumbnail: string
    uploaderName: string
    uploaderUrl: string
    uploaderAvatar: string
    uploadedDate: string
    shortDescription: string
    duration: number
    views: number
    uploaded: number
    uploaderVerified: boolean
  }

  interface ChannelDetails {
    description: string
    name: string
    subscribers: number
    thumbnail: string
    url: string
    verified: boolean
    videos: number
  }

  interface PlaylistDetails {
    name: string
    thumbnail: string
    uploaderName: string
    url: string
    videos: number
  }

  interface AlbumDetails {
    name: string
    thumbnail: string
    uploaderName: string
    url: string
    videos: number
  }

  namespace SearchResult {
    type SearchResultResponse<T extends SearchFilters> = {
      items: (T extends 'music_songs'
        ? VideoDetails
        : T extends 'channels'
        ? ChannelDetails
        : T extends 'playlists'
        ? PlaylistDetails
        : T extends 'music_albums'
        ? AlbumDetails
        : unknown)[]
      nextpage?: string
      suggestion?: string[]
      corrected: boolean
    }
  }

  namespace PlaylistDetailsExtended {
    interface Root {
      bannerUrl: string
      name: string
      nextpage?: string
      relatedStreams: VideoDetails[]
      thumbnailUrl: string
      uploader: string
      uploaderAvatar: string
      uploaderUrl: string
      videos: number
    }
  }

  namespace ChannelDetailsExtended {
    interface Root {
      avatarUrl: string
      bannerUrl: string
      description: string
      id: string
      name: string
      nextpage?: string
      relatedStreams: VideoDetails[]
      verified: boolean
      subscriberCount: number
    }
  }

  namespace VideoStreamDetails {
    interface Root {
      title: string
      description: string
      uploadDate: string
      uploader: string
      uploaderUrl: string
      uploaderAvatar: string
      thumbnailUrl: string
      hls: string
      dash: unknown
      lbryId: unknown
      uploaderVerified: boolean
      duration: number
      views: number
      likes: number
      dislikes: number
      uploaderSubscriberCount: number
      audioStreams: AudioStream[]
      videoStreams: VideoStream[]
      relatedStreams: VideoDetails[]
      subtitles: unknown[]
      livestream: boolean
      proxyUrl: string
      chapters: unknown[]
    }

    interface AudioStream {
      url: string
      format: string
      quality: string
      mimeType: string
      codec: string
      videoOnly: boolean
      bitrate: number
      initStart: number
      initEnd: number
      indexStart: number
      indexEnd: number
      width: number
      height: number
      fps: number
    }

    interface VideoStream {
      url: string
      format: string
      quality: string
      mimeType: string
      codec?: string
      videoOnly: boolean
      bitrate: number
      initStart: number
      initEnd: number
      indexStart: number
      indexEnd: number
      width: number
      height: number
      fps: number
    }
  }

  interface TokenResponse {
    token: string
  }

  namespace UserPlaylistDetails {
    interface Root {
      id: string
      name: string
      shortDescription?: string
      thumbnail: string
      videos: number
    }
  }

  type SearchObject<T extends PipedResources, K extends SearchFilters> = T extends PipedResources.SEARCH
    ? SearchRequest<K>
    : T extends PipedResources.PLAYLIST_DETAILS | PipedResources.PLAYLIST_DETAILS_NEXT
    ? PlaylistDetailsRequest
    : T extends PipedResources.CHANNEL_DETAILS | PipedResources.CHANNEL_DETAILS_NEXT
    ? ChannelDetailsRequest
    : T extends PipedResources.STREAM_DETAILS
    ? StreamRequest
    : T extends PipedResources.LOGIN
    ? LoginRequest
    : undefined

  type ResponseType<T extends PipedResources, K extends SearchFilters> = T extends PipedResources.SEARCH
    ? SearchResult.SearchResultResponse<K>
    : T extends PipedResources.PLAYLIST_DETAILS | PipedResources.PLAYLIST_DETAILS_NEXT
    ? PlaylistDetailsExtended.Root
    : T extends PipedResources.CHANNEL_DETAILS | PipedResources.CHANNEL_DETAILS_NEXT
    ? ChannelDetailsExtended.Root
    : T extends PipedResources.STREAM_DETAILS
    ? VideoStreamDetails.Root
    : T extends PipedResources.LOGIN
    ? TokenResponse
    : T extends PipedResources.USER_PLAYLISTS
    ? UserPlaylistDetails.Root[]
    : undefined
}
