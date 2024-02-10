/*
 *  youtubeResponses.d.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

declare namespace YoutubeResponses {
  // These are only the resources used
  enum ApiResources {
    CHANNELS = 'channels',
    PLAYLISTS = 'playlists',
    PLAYLIST_ITEMS = 'playlistItems',
    VIDEO_DETAILS = 'videos',
    SEARCH = 'search',
  }

  type SearchRequest = {
    params: {
      part: ['id', 'snippet'?]
      relatedToVideoId?: string
      videoCategoryId?: 10
      type?: 'video' | 'channel' | 'playlist'
      videoDuration?: 'short'
      maxResults?: number
      videoEmbeddable?: true
      order?: 'date' | 'relevance'
      channelId?: string
      q?: string
      safeSearch?: 'none' | 'moderate' | 'strict'
      pageToken?: string | undefined
    }
  }

  type ChannelRequest = {
    params: {
      part: ['id', 'snippet'?]
      mine?: true
      id?: string
    }
  }

  type PlaylistRequest = {
    params: {
      id?: string
      part: ['id', 'contentDetails', 'snippet'?]
      mine?: true
      maxResults?: number
      pageToken?: string | undefined
    }
  }

  type PlaylistItemsRequest = {
    params: {
      part: ['id', 'snippet'?]
      maxResults: number
      playlistId: string
      pageToken?: string | undefined
    }
  }

  type VideoDetailsRequest = {
    params: {
      part: ['contentDetails', 'snippet']
      id: string[]
      maxResults: number
    }
  }

  type SearchObject<T extends ApiResources> = T extends ApiResources.CHANNELS
    ? ChannelRequest
    : T extends ApiResources.PLAYLISTS
    ? PlaylistRequest
    : T extends ApiResources.PLAYLIST_ITEMS
    ? PlaylistItemsRequest
    : T extends ApiResources.VIDEO_DETAILS
    ? VideoDetailsRequest
    : T extends ApiResources.SEARCH
    ? SearchRequest
    : undefined

  namespace Thumbnails {
    interface ThumbDefault {
      url: string
      width: number
      height: number
    }

    interface ThumbMedium {
      url: string
      width: number
      height: number
    }

    interface ThumbHigh {
      url: string
      width: number
      height: number
    }

    interface ThumbStandard {
      url: string
      width: number
      height: number
    }

    interface ThumbMaxres {
      url: string
      width: number
      height: number
    }

    interface Thumbnails {
      default: ThumbDefault
      medium: ThumbMedium
      high: ThumbHigh
      standard: ThumbStandard
      maxres: ThumbMaxres
    }
  }

  interface PageInfo {
    totalResults: number
    resultsPerPage: number
  }
  namespace PlaylistItems {
    interface ResourceId {
      kind: string
      videoId: string
    }

    interface Snippet {
      publishedAt: string
      channelId: string
      title: string
      description: string
      thumbnails: Thumbnails.Thumbnails
      channelTitle: string
      playlistId: string
      position: number
      resourceId: ResourceId
      videoOwnerChannelTitle: string
      videoOwnerChannelId: string
    }

    interface Items {
      kind: string
      etag: string
      id: string
      snippet?: Snippet
    }

    interface PlaylistItems {
      kind: string
      etag: string
      nextPageToken?: string
      prevPageToken?: string
      items: Items[]
      pageInfo: PageInfo
    }
  }

  namespace UserPlaylists {
    interface Localized {
      title: string
      description: string
    }

    interface Snippet {
      publishedAt: string
      channelId: string
      title: string
      description: string
      thumbnails: Thumbnails.Thumbnails
      channelTitle: string
      localized: Localized
    }

    interface contentDetails {
      itemCount: number
    }

    interface Item {
      kind: string
      etag: string
      id: string
      snippet?: Snippet
      contentDetails: contentDetails
    }

    interface UserPlaylists {
      kind: string
      etag: string
      nextPageToken?: string
      prevPageToken?: string
      pageInfo: PageInfo
      items: Item[]
    }
  }
  interface Localized {
    title: string
    description: string
  }

  namespace ChannelInfo {
    interface Snippet {
      title: string
      description: string
      publishedAt: string
      thumbnails: Thumbnails.Thumbnails
      localized: Localized
    }

    interface Item {
      kind: string
      etag: string
      id: string
      snippet?: Snippet
    }

    interface ChannelInfo {
      kind: string
      etag: string
      pageInfo: PageInfo
      items: Item[]
    }
  }

  namespace VideoDetails {
    interface RegionRestriction {
      allowed: string[]
    }

    interface ContentDetails {
      duration: string
      dimension: string
      definition: string
      caption: string
      licensedContent: boolean
      regionRestriction: RegionRestriction
      contentRating: ContentRating
      projection: string
    }

    interface Item {
      kind: string
      etag: string
      id: string
      contentDetails: ContentDetails
      snippet: Snippet
    }

    interface Snippet {
      publishedAt: string
      channelId: string
      title: string
      description: string
      thumbnails: Thumbnails.Thumbnails
      channelTitle: string
      tags: string[]
      categoryId: string
      liveBroadcastContent: string
      localized: Localized
    }

    interface VideoDetails {
      kind: string
      etag: string
      nextPageToken?: string
      prevPageToken?: string
      items: Item[]
      pageInfo: PageInfo
    }
  }

  namespace SearchDetails {
    interface ChannelInfo extends ChannelInfo.Snippet {
      channelId: string
      channelTitle: string
    }
    interface Item {
      id: {
        kind: string
        videoId?: string
        playlistId?: string
      }
      kind: string
      etag: string
      snippet: VideoDetails.Snippet | ChannelInfo
    }
    interface SearchDetails {
      items: Item[]
      nextPageToken: string
    }
  }

  type ResponseType<T extends ApiResources> = T extends ApiResources.CHANNELS
    ? ChannelInfo.ChannelInfo
    : T extends ApiResources.PLAYLISTS
    ? UserPlaylists.UserPlaylists
    : T extends ApiResources.PLAYLIST_ITEMS
    ? PlaylistItems.PlaylistItems
    : T extends ApiResources.VIDEO_DETAILS
    ? VideoDetails.VideoDetails
    : T extends ApiResources.SEARCH
    ? SearchDetails.SearchDetails
    : undefined
}
