/*
 *  ThemeHandler.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { ThemeStore } from '@/mainWindow/store/themes'
import { Component } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'

type StyleElement = {
  sheet: CSSStyleSheet
}

@Component
export default class ThemeHandler extends Vue {
  private root = document.documentElement
  private _themeStore: ThemeStore | undefined

  public setColorsToRoot(theme: ThemeDetails | undefined) {
    const colors = theme?.theme
    if (!colors) {
      this.root.style.cssText = ''
    }
    if (colors) {
      for (const [key, value] of Object.entries(colors)) {
        this.root.style.setProperty(`--${key}`, value)
      }
    }

    this.setRGBValues()
    this.setCheckboxValues()
    if (theme?.theme.customCSS) {
      this.loadCss(theme?.theme.customCSS)
    } else {
      this.unloadCss()
    }
  }

  private async loadCss(cssPath: string) {
    let css = await this.transformCSS(cssPath)

    // now substitute %themeDir% within css definition
    let themeDir = cssPath.replaceAll('\\', '/')
    themeDir = themeDir.substring(0, themeDir.lastIndexOf('/'))
    css = css.replaceAll('%themeDir%', themeDir)

    const customStylesheet = (document.getElementById('custom-css') as HTMLStyleElement) ?? this.createStyleNode()
    customStylesheet.innerHTML = css

    document.head.append(customStylesheet)
  }

  private async unloadCss() {
    const customStylesheet = document.getElementById('custom-css') as HTMLStyleElement
    if (customStylesheet) {
      document.head.removeChild(customStylesheet)
    }
  }

  private transformCSS(cssPath: string) {
    return window.ThemeUtils.transformCSS(cssPath)
  }

  private createStyleNode() {
    const style = document.createElement('style')
    style.id = 'custom-css'
    return style
  }

  get themeStore() {
    return this._themeStore
  }

  set themeStore(vxm: ThemeStore | undefined) {
    this._themeStore = vxm
  }

  private rgba2hex(rgba: string) {
    let tmp: String

    if (rgba.startsWith('rgb(')) {
      tmp = rgba.replaceAll('rgb(', '').replaceAll(')', '')
    } else if (rgba.startsWith('rgba(')) {
      tmp = rgba.replaceAll('rgba(', '').replaceAll(')', '')
    } else {
      return rgba
    }

    const split = tmp.split(',').map((val) => val.trim())

    const r = parseInt(split[0])
    const g = parseInt(split[1])
    const b = parseInt(split[2])

    return `#${r.toString(16)}${g.toString(16)}${b.toString(16)}`
  }

  private setCheckboxValues() {
    const docStyle = getComputedStyle(this.root)
    let style = document.getElementById('checkbox-stylesheet')
    if (!style) {
      style = document.createElement('style')
      style.id = 'checkbox-stylesheet'
      document.head.appendChild(style)
    }

    const sheet = (style as unknown as StyleElement).sheet
    if (sheet.cssRules.length > 0) {
      sheet.deleteRule(0)
    }

    let textPrimary = docStyle.getPropertyValue('--textPrimary')
    if (textPrimary) {
      textPrimary = this.rgba2hex(textPrimary)
    }
    sheet.insertRule(
      `.custom-checkbox .custom-control-input:checked ~ .custom-control-label::after { background-image: url("data:image/svg+xml,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 width=%278%27 height=%278%27 viewBox=%270 0 8 8%27%3e%3cpath fill=%27%23${textPrimary
        .replace('#', '')
        .trim()
        .toLowerCase()}%27 d=%27M6.564.75l-3.59 3.612-1.538-1.55L0 4.26l2.974 2.99L8 2.193z%27/%3e%3c/svg%3e") !important; }`,
    )
  }

  private setRGBValues() {
    const docStyle = getComputedStyle(this.root)
    const keys = [
      'primary',
      'secondary',
      'tertiary',
      'textPrimary',
      'textSecondary',
      'textInverse',
      'accent',
      'divider',
    ]
    for (const key of keys) {
      this.root.style.setProperty(`--${key}-rgb`, this.hexToRgb(docStyle.getPropertyValue(`--${key}`).trim()))
    }
  }

  private hexToRgb(hex: string) {
    let parsedHex = hex
    if (hex.startsWith('#')) {
      parsedHex = hex.substring(1)
    }

    const r = parseInt(parsedHex.substring(0, 2), 16)
    const g = parseInt(parsedHex.substring(2, 4), 16)
    const b = parseInt(parsedHex.substring(4, 6), 16)

    return [r, g, b].join(',')
  }

  public fetchThemeFromID() {
    window.ThemeUtils.getActiveTheme().then((theme) => this.setColorsToRoot(theme))
  }

  public fetchSongView() {
    window.ThemeUtils.getSongView().then(
      (view) => this.themeStore && (this._themeStore as ThemeStore).songView === view,
    )
  }

  private listenTempTheme() {
    window.ThemeUtils.onThemeRefresh((theme) => {
      this.setColorsToRoot(theme)
    })
  }

  mounted() {
    this.fetchSongView()
    this.fetchThemeFromID()
    this.listenTempTheme()
  }
}
