import { exec } from 'child_process'
import { access, readFile } from 'fs/promises'
import path from 'path'
import ini from 'ini'
import { app, nativeTheme } from 'electron'

enum DesktopEnvironments {
  PLASMA = 'plasma',
  PLASMA_WAYLAND = 'plasmawayland',
  KDE = 'KDE',
  CINNAMON = 'cinnamon',
  GNOME = 'Gnome',
  UNITY = 'Unity',
  BUDGIE = 'Budgie',
  MATE = 'Mate',
  XFCE = 'Xfce'
}

const defaultTheme = {
  primary: '#212121',
  secondary: '#282828',
  tertiary: '#151515',
  textPrimary: '#ffffff',
  textSecondary: '#565656',
  textInverse: '#000000',
  accent: '#65CB88',
  divider: 'rgba(79, 79, 79, 0.67)'
}

interface KdeGlobals {
  General?: {
    ColorScheme: string
  }
  'Colors:View': {
    BackgroundNormal: string
    BackgroundAlternate: string
    ForegroundNormal: string
    ForegroundInactive: string
    DecorationFocus: string
  }
  'Colors:Window': {
    BackgroundNormal: string
  }
  'Colors:Selection': {
    BackgroundNormal: string
  }
}

export class SystemThemeHandler {
  public async getWindowsStyle() {
    // https://github.com/electron/electron/issues/23487

    const accentQuery = (
      await execAsync('reg query HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\DWM /v ColorizationAfterglow')
    ).stdout.split(' ')

    const accent = dwordToRgb(accentQuery[accentQuery.length - 1].trim())
    let theme: ThemeItem

    if (nativeTheme.shouldUseDarkColors) {
      theme = {
        primary: '#1C1C1C',
        secondary: '#282828',
        tertiary: '#151515',
        textPrimary: '#FFFFFF',
        textSecondary: '#D4D4D4',
        textInverse: '#000000',
        accent,
        divider: 'rgba(79, 79, 79, 0.67)'
      }
    } else {
      theme = {
        primary: '#EEEEEE',
        secondary: '#F9F9F9',
        tertiary: '#FFFFFF',
        textPrimary: '#000000',
        textSecondary: '#636363',
        textInverse: '#000000',
        accent,
        divider: 'rgba(79, 79, 79, 0.67)'
      }
    }

    return {
      id: 'system_default',
      name: 'System Theme (Beta)',
      author: 'Moosync',
      theme
    }
  }

  public async getLinuxStyle(): Promise<ThemeDetails | undefined> {
    const de = this.getDesktopEnvironment()

    switch (de) {
      case DesktopEnvironments.KDE:
      case DesktopEnvironments.PLASMA:
      case DesktopEnvironments.PLASMA_WAYLAND:
        return this.getKDETheme()

      case DesktopEnvironments.GNOME:
      case DesktopEnvironments.UNITY:
      case DesktopEnvironments.BUDGIE:
      default:
        return this.getGnomeTheme()

      case DesktopEnvironments.CINNAMON:
        return this.getCinnamonTheme()

      case DesktopEnvironments.MATE:
        return this.getMateTheme()

      // TODO: Parse GTK2.0 themes for xfce
      case DesktopEnvironments.XFCE:
        return
    }
  }

  private getDesktopEnvironment() {
    return process.env['DESKTOP_SESSION'] as DesktopEnvironments
  }

  private async getKDEConfigUtil() {
    try {
      await execAsync('type -p kf5-config')
      return 'kf5-config'
    } catch (_) {
      console.info('kf5-config not found')
    }

    try {
      await execAsync('type -p kde4-config')
      return 'kde4-config'
    } catch (_) {
      console.info('kde4-config not found')
    }

    try {
      await execAsync('type -p kde-config')
      return 'kde-config'
    } catch (_) {
      console.info('kde-config not found')
    }
  }

  private async getKDEConfigDirs() {
    const execUtil = await this.getKDEConfigUtil()
    console.debug('Got config tool', execUtil)

    if (execUtil) {
      try {
        const directory = (await execAsync(`${execUtil} --path config`)).stdout
        return directory.split(':')
      } catch (e) {
        console.error(e)
      }
    }
  }

  private async getKDETheme() {
    const dirs = await this.getKDEConfigDirs()
    if (dirs) {
      const kdeDefaultsIndex = dirs.findIndex((val) => val.includes('kdedefaults'))
      if (kdeDefaultsIndex !== -1) {
        const dir = dirs[kdeDefaultsIndex]
        dirs.splice(kdeDefaultsIndex, 1)
        dirs.unshift(dir)
      }

      for (const directory of dirs) {
        try {
          const config = path.join(directory, 'kdeglobals')
          await access(config)
          return this.parseKDETheme(config)
        } catch (_) {
          console.warn(path.join(directory, 'kdeglobals'), 'does not exist')
        }
      }
    }
  }

  private async findColorSchemes(themeName: string) {
    const dirs = ['/usr/share/color-schemes', path.join(app.getPath('home'), '.local', 'share', 'color-schemes')]
    for (const dir of dirs) {
      const themePath = path.join(dir, themeName + '.colors')
      try {
        await access(themePath)
        return themePath
      } catch (_) {
        console.warn(themePath, 'does not exist')
      }
    }
  }

  private async parseKDETheme(file: string): Promise<ThemeDetails | undefined> {
    const data = ini.parse(await readFile(file, 'utf-8')) as KdeGlobals

    let colorsData: KdeGlobals | undefined
    if (data['General'] && data['General']['ColorScheme']) {
      const themeName = data['General']['ColorScheme']
      const colorsFile = await this.findColorSchemes(themeName)
      if (colorsFile) {
        colorsData = ini.parse(await readFile(colorsFile, 'utf-8')) as KdeGlobals
      }
    }

    if (data['Colors:View'] || data['Colors:Window'] || data['Colors:Selection']) {
      colorsData = data
    }

    if (colorsData) {
      const view = colorsData['Colors:View']
      const window = colorsData['Colors:Window']
      const selection = colorsData['Colors:Selection']

      const theme = {
        primary: rgbToHex(view['BackgroundNormal']) ?? defaultTheme.primary,
        secondary: rgbToHex(view['BackgroundAlternate']) ?? defaultTheme.secondary,
        tertiary: rgbToHex(window['BackgroundNormal']) ?? defaultTheme.tertiary,
        textPrimary: rgbToHex(view['ForegroundNormal']) ?? defaultTheme.textPrimary,
        textSecondary: rgbToHex(view['ForegroundInactive']) ?? defaultTheme.textSecondary,
        textInverse: rgbToHex(view['ForegroundNormal'], true) ?? defaultTheme.textInverse,
        accent: rgbToHex(selection['BackgroundNormal']) ?? defaultTheme.accent,
        divider: rgbToHex(view['DecorationFocus']) ?? defaultTheme.divider
      }

      return {
        id: 'system_default',
        name: 'System Theme (Plasma) (Beta)',
        author: 'Moosync',
        theme
      }
    }
  }

  private async findVar(varName: string, filename: string): Promise<string | undefined> {
    const themeVar = (await execAsync(`grep '@define-color ${varName} ' ${filename}`)).stdout
      .replaceAll(`@define-color ${varName}`, '')
      .replaceAll(';', '')
      .trim()
    if (themeVar.startsWith('@')) {
      return this.findVar(themeVar.substring(1, themeVar.length), filename)
    }

    return themeVar
  }

  private async parseGTKTheme(themePath: string): Promise<ThemeDetails | undefined> {
    const filename = path.join(themePath, 'gtk-3.0', 'gtk.css')
    try {
      const theme = {
        primary: (await this.findVar('theme_base_color', filename)) ?? defaultTheme.primary,
        secondary: (await this.findVar('wm_bg', filename)) ?? defaultTheme.secondary,
        tertiary: (await this.findVar('theme_bg_color', filename)) ?? defaultTheme.tertiary,
        textPrimary: (await this.findVar('theme_text_color', filename)) ?? defaultTheme.textPrimary,
        textSecondary: (await this.findVar('placeholder_text_color', filename)) ?? defaultTheme.textSecondary,
        textInverse: (await this.findVar('theme_unfocused_selected_fg_color', filename)) ?? defaultTheme.textInverse,
        accent: (await this.findVar('theme_selected_bg_color', filename)) ?? defaultTheme.accent,
        divider: (await this.findVar('borders', filename)) ?? defaultTheme.divider
      }

      return {
        id: 'system_default',
        name: 'System Theme (GTK) (Beta)',
        author: 'Moosync',
        theme: theme as ThemeItem
      }
    } catch (e) {
      console.error('Error while fetching theme')
    }
  }

  private async findGTKTheme(theme: string) {
    const themePaths = ['/usr/share/themes', path.join(app.getPath('home'), '.themes')]

    for (const dir of themePaths) {
      try {
        const themeDir = path.join(dir, theme.trim().replaceAll(/['"]+/g, ''))
        await access(themeDir)
        return this.parseGTKTheme(themeDir)
      } catch (e) {
        console.error('Cant access', dir)
      }
    }
  }

  private async getGnomeTheme() {
    try {
      const themeName = (await execAsync('gsettings get org.gnome.desktop.interface gtk-theme')).stdout
      return this.findGTKTheme(themeName)
    } catch (e) {
      console.error('Cant find GTK theme', e)
    }
  }

  private async getCinnamonTheme() {
    try {
      const themeName = (await execAsync('gsettings get org.cinnamon.desktop.interface gtk-theme')).stdout
      return this.findGTKTheme(themeName)
    } catch (e) {
      console.error('Cant find GTK theme', e)
    }
  }

  private async getMateTheme() {
    try {
      const themeName = (await execAsync('gsettings get org.mate.interface gtk-theme')).stdout
      return this.findGTKTheme(themeName)
    } catch (e) {
      console.error('Cant find GTK theme', e)
    }
  }
}

function rgbToHex(commaSeperated: string, inverse = false) {
  if (commaSeperated) {
    const split = commaSeperated.split(',')
    let r = parseInt(split[0]),
      g = parseInt(split[1]),
      b = parseInt(split[2])

    if (inverse) {
      r = 255 - r
      g = 255 - g
      b = 255 - b
    }

    return '#' + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)
  }
}

function dwordToRgb(dword: string) {
  return '#' + dword.substring(4)
}

async function execAsync(command: string) {
  return new Promise<{ stdout: string; stderr: string }>((resolve, reject) => {
    exec(command, (err, stdout, stderr) => {
      if (err) {
        reject(err)
      }

      resolve({ stdout, stderr })
    })
  })
}
