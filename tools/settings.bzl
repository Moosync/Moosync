"Build settings for cross-platform configuration."

# Use native Starlark build settings (no skylib needed)
TargetOsInfo = provider(
    doc = "Provides the target OS being built for.",
    fields = {"value": "The target OS string (e.g., 'android', 'linux', 'windows')"},
)

def _target_os_impl(ctx):
    return TargetOsInfo(value = ctx.build_setting_value)

target_os_setting = rule(
    implementation = _target_os_impl,
    build_setting = config.string(flag = True),
)
