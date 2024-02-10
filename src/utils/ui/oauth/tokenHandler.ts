/*
 *  tokenHandler.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import {
  AppAuthError,
  AuthorizationServiceConfiguration,
  BaseTokenRequestHandler,
  QueryStringUtils,
  Requestor,
  TokenError,
  TokenErrorJson,
  TokenRequest,
  TokenResponse,
  TokenResponseJson,
} from '@openid/appauth'

export class TokenRequestHandlerWClientSecret extends BaseTokenRequestHandler {
  private clientSecret: string

  private isTokenResponseExt(response: TokenResponseJson | TokenErrorJson): response is TokenResponseJson {
    return (response as TokenErrorJson).error === undefined
  }

  constructor(clientSecret: string, requestor?: Requestor, utils?: QueryStringUtils) {
    super(requestor, utils)
    this.clientSecret = clientSecret
  }

  async performTokenRequest(
    configuration: AuthorizationServiceConfiguration,
    request: TokenRequest,
  ): Promise<TokenResponse> {
    // Force client-secret in token fetch request
    const reqStrMap = request.toStringMap()
    reqStrMap.client_secret = this.clientSecret

    const tokenResponse = this.requestor.xhr<TokenResponseJson | TokenErrorJson>({
      url: configuration.tokenEndpoint,
      method: 'POST',
      dataType: 'json', // adding implicit dataType
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      data: this.utils.stringify(reqStrMap),
    })

    return tokenResponse.then((response) => {
      if (this.isTokenResponseExt(response)) {
        return new TokenResponse(response)
      } else {
        return Promise.reject<TokenResponse>(new AppAuthError(response.error, new TokenError(response)))
      }
    })
  }
}
