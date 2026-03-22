import java.io.File
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.api.execution.ExecSpec

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    // Map Gradle target names to Bazel platform names and ABI folders
    private val targetToPlatform = mapOf(
        "aarch64" to Pair("arm64-v8a", "arm64-v8a"),
        "armv7" to Pair("armeabi-v7a", "armeabi-v7a"),
        "x86_64" to Pair("x86_64", "x86_64")
    )

    private val appPackage = "app.moosync.moosync"

    @TaskAction
    fun assemble() {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")

        val (platform, abi) = targetToPlatform[target]
            ?: throw GradleException("Unknown target: $target")

        val rootDir = File(project.projectDir, rootDirRel).canonicalFile
        val jniLibsDir = File(project.projectDir, "src/main/jniLibs/$abi")
        val destFile = File(jniLibsDir, "libapp_lib.so")

        // Skip Bazel build if -PskipBazel is passed (for iterating on Kotlin code)
        val skipBazel = project.hasProperty("skipBazel")
        if (skipBazel && destFile.exists()) {
            println("Skipping Bazel build (-PskipBazel). Using existing .so")
        } else {
            val bazelTarget = "//tauri:moosync_android"
            val platformFlag = "//toolchains/android:$platform"

            // Build with Bazel (includes UI as a dependency)
            val bazelArgs = mutableListOf(
                "build",
                bazelTarget,
                "--platforms=$platformFlag",
                "--//tools:target_os=android",
                "--action_env=ANDROID_NDK_HOME"
            )
            if (release) {
                bazelArgs.add("-c")
                bazelArgs.add("opt")
                bazelArgs.add("--//:release")
            }

            project.exec { spec: ExecSpec ->
                spec.workingDir(rootDir)
                spec.executable("bazel")
                spec.args(bazelArgs)
            }.assertNormalExitValue()

            // Copy the built .so to jniLibs
            val soFile = File(rootDir, "bazel-bin/tauri/libapp_lib.so")
            jniLibsDir.mkdirs()
            soFile.copyTo(destFile, overwrite = true)
            println("Copied ${soFile.path} to ${destFile.path}")
        }

        // Copy UI assets (only once)
        val assetsDir = File(project.projectDir, "src/main/assets")
        val uiDistDir = File(rootDir, "bazel-bin/ui/gen_dist/moosync_dist")
        if (!File(assetsDir, "index.html").exists() && uiDistDir.exists()) {
            println("Copying UI assets from ${uiDistDir.path}")
            uiDistDir.copyRecursively(assetsDir, overwrite = true)
        }

        // Generate Tauri/Wry kotlin files from crate sources (only once)
        val genDir = File(project.projectDir, "src/main/java/app/moosync/moosync")
        if (!File(genDir, "WryActivity.kt").exists()) {
            generateKotlinFiles(rootDir, genDir)
        }

        // Copy proguard rules from tauri crate (only once)
        val proguardDest = File(project.projectDir, "proguard-tauri.pro")
        if (!proguardDest.exists()) {
            copyProguardRules(rootDir, proguardDest)
        }
    }

    private fun generateKotlinFiles(rootDir: File, genDir: File) {
        // Find Bazel external directory via bazel-Moosync symlink
        val externalDir = File(rootDir, "bazel-Moosync/external")
        if (!externalDir.exists()) {
            println("Warning: bazel-moosync/external not found, skipping kotlin generation")
            return
        }

        // Find tauri and wry crate directories
        val tauriDir = externalDir.listFiles()?.find {
            it.name.contains("tauri-2.") && !it.name.contains("plugin")
        }
        // Match wry crate specifically (not tauri-runtime-wry)
        val wryDir = externalDir.listFiles()?.find {
            it.name.contains("__wry-") || it.name.endsWith("+wry-0.54.3")
        }

        if (tauriDir == null || wryDir == null) {
            println("Warning: Could not find tauri/wry crates in $externalDir")
            return
        }

        genDir.mkdirs()

        // Copy and process TauriActivity.kt from tauri
        val tauriCodegenDir = File(tauriDir, "mobile/android-codegen")
        tauriCodegenDir.listFiles()?.filter { it.extension == "kt" }?.forEach { file ->
            val content = file.readText().replace("{{package}}", appPackage)
            File(genDir, file.name).writeText(content)
            println("Generated ${file.name} from tauri crate")
        }

        // Copy and process wry kotlin files
        val wryKotlinDir = File(wryDir, "src/android/kotlin")
        wryKotlinDir.listFiles()?.filter { it.extension == "kt" }?.forEach { file ->
            val content = file.readText()
                .replace("{{package}}", appPackage)
                .replace("{{library}}", "app_lib")
                .replace("{{class-extension}}", "")
                .replace("{{class-init}}", "")
            File(genDir, file.name).writeText(content)
            println("Generated ${file.name} from wry crate")
        }
    }

    private fun copyProguardRules(rootDir: File, destFile: File) {
        val externalDir = File(rootDir, "bazel-Moosync/external")
        if (!externalDir.exists()) {
            println("Warning: bazel-Moosync/external not found, skipping proguard copy")
            return
        }

        val tauriDir = externalDir.listFiles()?.find {
            it.name.contains("tauri-2.") && !it.name.contains("plugin")
        }
        if (tauriDir == null) {
            println("Warning: Could not find tauri crate in $externalDir")
            return
        }

        val proguardSrc = File(tauriDir, "mobile/proguard-tauri.pro")
        if (proguardSrc.exists()) {
            val content = proguardSrc.readText().replace("\$PACKAGE", appPackage)
            destFile.writeText(content)
            println("Generated proguard-tauri.pro from tauri crate")
        }
    }
}