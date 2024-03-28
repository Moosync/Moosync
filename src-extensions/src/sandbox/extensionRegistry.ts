/*
 *  extensionRegistry.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

export abstract class AbstractExtensionRegistry {
  abstract register(extension: ExtensionItem): void
  abstract get(options?: getExtensionOptions): Iterable<ExtensionItem>
  abstract setStarted(packageName: string, status: boolean): void
}

export class InMemoryRegistry extends AbstractExtensionRegistry {
  private extensionStore: ExtensionItem[] = []

  register(extension: ExtensionItem) {
    this.extensionStore.push(extension)
  }

  deregister(packageName: string) {
    const index = this.extensionStore.findIndex((val) => val.packageName === packageName)
    if (index !== -1) {
      this.extensionStore.splice(index, 1)
    }
  }

  private checkPackageName(packageName: string | undefined, item: ExtensionItem) {
    if (packageName) {
      return item.packageName === packageName
    }
    return true
  }

  private checkStarted(started: boolean | undefined, item: ExtensionItem) {
    if (started !== undefined) {
      return item.hasStarted === started
    }
    return true
  }

  get(options?: getExtensionOptions): Iterable<ExtensionItem> {
    return this.extensionStore.filter((val) =>
      options ? !!(this.checkPackageName(options.packageName, val) && this.checkStarted(options.started, val)) : true,
    )
  }

  setStarted(packageName: string, status: boolean) {
    const extension = this.extensionStore.find((val) => val.packageName === packageName)
    if (extension) extension.hasStarted = status
  }
}
