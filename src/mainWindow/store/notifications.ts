/*
 *  notifications.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { VuexModule } from './module'
import { mutation } from 'vuex-class-component'

export class NotifierStore extends VuexModule.With({ namespaced: 'notification' }) {
  private notificationStore: NotificationObject[] = []

  get notifications() {
    return this.notificationStore
  }

  @mutation
  public emit(notif: NotificationObject) {
    const index = this.notificationStore.findIndex((value) => value.id === notif.id)
    if (index !== -1) {
      this.notificationStore.splice(index, 1)
    }
    this.notificationStore.push(notif)
  }
}
