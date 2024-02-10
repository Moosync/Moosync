/*
 *  oauth.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

type OAuthProviderConfig = {
  openIdConnectUrl: string
  clientId: string
  clientSecret: string
  redirectUri: string
  scope: string
  keytarService: string
  oAuthChannel: string
}
