/*
 *  flow.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

/*
 * Copyright 2017 Google Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License") you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */

import { AppAuthError, FetchRequestor } from '@openid/appauth'
import { AuthorizationNotifier } from '@openid/appauth/built/authorization_request_handler'
import {
  GRANT_TYPE_AUTHORIZATION_CODE,
  GRANT_TYPE_REFRESH_TOKEN,
  TokenRequest,
} from '@openid/appauth/built/token_request'

import { AuthFlowRequestHandler } from './AuthFlowRequestHandler'
import { WebCrypto } from './crypto_utils'
import { TokenRequestHandlerWClientSecret } from './tokenHandler'
import { AuthorizationRequest } from '@openid/appauth/built/authorization_request'
import { AuthorizationServiceConfiguration } from '@openid/appauth/built/authorization_service_configuration'
import { TokenRequestHandler } from '@openid/appauth/built/token_request_handler'
import { TokenResponse, TokenResponseJson } from '@openid/appauth/built/token_response'
import { StringMap } from '@openid/appauth/built/types'
import EventEmitter from 'events'

export class AuthStateEmitter extends EventEmitter {
  static ON_TOKEN_RESPONSE = 'on_token_response'
}

const requestor = new FetchRequestor()

export class AuthFlow {
  private notifier: AuthorizationNotifier
  private authorizationHandler: AuthFlowRequestHandler
  private tokenHandler: TokenRequestHandler
  authStateEmitter: AuthStateEmitter

  // state
  private serviceConfig: AuthorizationServiceConfiguration

  private refreshToken: string | undefined
  private accessTokenResponse: TokenResponse | undefined

  config: OAuthProviderConfig

  private fetchedToken: Promise<void>
  private fetchedTokenResolver?: () => void

  constructor(config: OAuthProviderConfig, serviceConfig: AuthorizationServiceConfiguration, refreshToken = true) {
    if (!config.oAuthChannel && process.env.NODE_ENV === 'production')
      throw new Error('oAuth callback failed to register')

    this.config = config
    this.serviceConfig = serviceConfig

    this.notifier = new AuthorizationNotifier()
    this.authStateEmitter = new AuthStateEmitter()

    this.authorizationHandler = new AuthFlowRequestHandler(config.oAuthChannel)

    this.tokenHandler = new TokenRequestHandlerWClientSecret(config.clientSecret, requestor)

    // set notifier to deliver responses
    this.authorizationHandler.setAuthorizationNotifier(this.notifier)

    // set a listener to listen for authorization responses
    // make refresh and access token requests.

    this.fetchedToken = new Promise<void>((resolve) => {
      if (refreshToken) {
        this.fetchRefreshToken().then(resolve)
      } else {
        this.fetchedTokenResolver = resolve
      }
    })

    this.notifier.setAuthorizationListener((request, response) => {
      if (response) {
        let codeVerifier: string | undefined
        if (request.internal?.code_verifier) {
          codeVerifier = request.internal.code_verifier
        }

        this.makeRefreshTokenRequest(response.code, codeVerifier)
          .then(() => this.performWithFreshTokens())
          .then(() => {
            this.authStateEmitter.emit(AuthStateEmitter.ON_TOKEN_RESPONSE)
          })
      }
    })
  }

  private async fetchRefreshToken() {
    if (this.config) this.refreshToken = (await window.Store.getSecure(this.config.keytarService)) ?? undefined
  }

  private async storeRefreshToken() {
    if (this.config && this.refreshToken) return window.Store.setSecure(this.config.keytarService, this.refreshToken)
  }

  public async makeAuthorizationRequest(username?: string) {
    if (!this.config) throw new AppAuthError('config not yet initialized')

    if (!this.fetchedToken) throw new AppAuthError('service not yet initialized')

    const extras: StringMap = { prompt: 'consent', access_type: 'offline' }
    if (username) {
      extras['login_hint'] = username
    }

    // create a request
    const request = new AuthorizationRequest(
      {
        client_id: this.config.clientId,
        redirect_uri: this.config.redirectUri,
        scope: this.config.scope,
        response_type: AuthorizationRequest.RESPONSE_TYPE_CODE,
        extras: extras,
      },
      new WebCrypto(),
    )

    return this.authorizationHandler.performAuthorizationRequest(this.serviceConfig, request)
  }

  private async makeRefreshTokenRequest(code: string, codeVerifier: string | undefined): Promise<void> {
    if (!this.config) throw new AppAuthError('config not yet initialized')

    if (!this.fetchedToken) throw new AppAuthError('service not yet initialized')

    if (!this.serviceConfig) {
      return Promise.resolve()
    }

    const extras: StringMap = {}

    if (codeVerifier) {
      extras.code_verifier = codeVerifier
    }

    // use the code to make the token request.
    const request = new TokenRequest({
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
      grant_type: GRANT_TYPE_AUTHORIZATION_CODE,
      code: code,
      extras: extras,
    })

    try {
      const response = await this.tokenHandler.performTokenRequest(this.serviceConfig, request)
      this.refreshToken = response.refreshToken
      this.accessTokenResponse = response
      this.storeRefreshToken()
    } catch (err) {
      console.error(err)
      this.signOut()
    }
  }

  async loggedIn(): Promise<boolean> {
    await this.fetchedToken
    return !!this.accessTokenResponse && this.accessTokenResponse.isValid()
  }

  signOut() {
    if (!this.config) throw new AppAuthError('config not yet initialized')

    // forget all cached token state
    window.Store.removeSecure(this.config.keytarService)
    this.accessTokenResponse = undefined
    this.refreshToken = undefined
  }

  public async hasValidRefreshToken() {
    await this.fetchedToken
    return !!this.refreshToken
  }

  async performWithFreshTokens(): Promise<string | undefined> {
    if (!this.config) throw new AppAuthError('config not yet initialized')

    if (!this.fetchedToken) return Promise.reject('Service not initialized')

    if (!this.refreshToken && !this.accessTokenResponse) {
      return Promise.resolve('Missing refreshToken.')
    }

    if (this.accessTokenResponse?.isValid()) {
      return this.accessTokenResponse.accessToken
    }

    const request = new TokenRequest({
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
      grant_type: GRANT_TYPE_REFRESH_TOKEN,
      refresh_token: this.refreshToken,
    })

    try {
      const response = await this.tokenHandler.performTokenRequest(this.serviceConfig, request)
      this.accessTokenResponse = response
      return response.accessToken
    } catch (err) {
      console.error(err)
      this.signOut()
    }
  }

  public setToken(resp: TokenResponseJson) {
    this.accessTokenResponse = new TokenResponse(resp)
    if (this.fetchedTokenResolver) {
      this.fetchedTokenResolver()
    }
  }
}
