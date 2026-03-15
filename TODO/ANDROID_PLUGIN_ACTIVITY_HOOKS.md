# Android Plugin Activity Hooks

Abstract Activity-level APIs so Tauri plugins can register ActivityResultLaunchers and access lifecycle hooks without manual glue code in MainActivity.kt.

## Current State

MainActivity.kt manually wires each plugin that needs Activity APIs:

```kotlin
class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Manual per-plugin wiring
        SelfUpdatePlugin.unknownSourcesLauncher = registerForActivityResult(...)
        SelfUpdatePlugin.apkInstallLauncher = registerForActivityResult(...)
    }
}
```

## Proposed Solution

1. Define a plugin interface with Activity lifecycle hooks:

```kotlin
interface TauriActivityPlugin {
    fun onActivityCreate(activity: Activity, registry: ActivityResultRegistry) {}
    fun onActivityResume(activity: Activity) {}
    fun onActivityPause(activity: Activity) {}
    // etc.
}
```

2. MainActivity becomes generic - iterates registered plugins:

```kotlin
class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        TauriPluginManager.activityPlugins.forEach {
            it.onActivityCreate(this, activityResultRegistry)
        }
    }
}
```

3. Each plugin registers its own launchers:

```kotlin
class SelfUpdatePlugin : Plugin(), TauriActivityPlugin {
    private lateinit var unknownSourcesLauncher: ActivityResultLauncher<Intent>

    override fun onActivityCreate(activity: Activity, registry: ActivityResultRegistry) {
        unknownSourcesLauncher = registry.register("unknown_sources", ...) { result ->
            handleUnknownSourcesResult(result)
        }
    }
}
```

## Benefits

- Zero per-plugin code in MainActivity.kt
- Plugins are fully self-contained
- Adding new plugins with Activity needs requires no MainActivity changes
- Cleaner separation of concerns

## Considerations

- Need to ensure plugin registration order is deterministic
- ActivityResultRegistry keys must be unique across plugins
- Consider how this integrates with Tauri's existing plugin lifecycle
