plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("com.google.devtools.ksp") version "2.1.0-1.0.29"
}

android {
    namespace = "app.moosync.audioplayer"
    compileSdk = 34

    defaultConfig {
        minSdk = 26

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
//        ndk {
//            abiFilters.addAll(listOf("armeabi-v7a", "arm64-v8a", "x86", "x86_64"))
//        }
    }

//    sourceSets {
//        getByName("main") {
//            jniLibs.srcDir("src/main/jniLibs")
//        }
//    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
}

dependencies {
    implementation("com.github.bumptech.glide:glide:4.14.2")
    ksp("com.github.bumptech.glide:ksp:4.14.2")
    implementation("com.pierfrancescosoffritti.androidyoutubeplayer:core:12.1.1")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("androidx.legacy:legacy-support-v4:1.0.0")
    implementation("androidx.core:core-ktx:1.9.0")
    implementation("androidx.appcompat:appcompat:1.6.0")
    implementation("com.google.android.material:material:1.7.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
    implementation(project(":tauri-android"))
}

//val rustBasePath = "./librespot-jni"
//val archTriplets = mapOf(
//    "armeabi-v7a" to "armv7-linux-androideabi",
//    "arm64-v8a" to "aarch64-linux-android",
//    "x86_64" to "x86_64-linux-android"
//)
//
//archTriplets.forEach { (arch, target) ->
//    val cargoTargetDirectory = "$rustBasePath/target"
//
//    // Build with cargo
//    tasks.register("cargoBuild${arch.capitalize()}", Exec::class) {
//        group = "build"
//        description = "Building core for $arch"
//        workingDir(file(rustBasePath))
//        commandLine("cargo", "ndk", "-t", target, "build")
//    }
//
//    // Sync shared native dependencies
//    tasks.register("syncRustDeps${arch.capitalize()}", Sync::class) {
//        dependsOn("cargoBuild${arch.capitalize()}")
//        from("$rustBasePath/src/libs/$arch")
//        include("*.so")
//        into("src/main/jniLibs/$arch")
//    }
//
//    // Copy build libs into this app's libs directory
//    tasks.register("rustDeploy${arch.capitalize()}", Copy::class) {
//        dependsOn("syncRustDeps${arch.capitalize()}")
//        group = "build"
//        description = "Copy rust libs for ($arch) to jniLibs"
//        from("$cargoTargetDirectory/$target/debug")
//        include("*.so")
//        into("src/main/jniLibs/$arch")
//    }
//
//    // Hook up tasks to execute before building Java
//    tasks.withType<JavaCompile> {
//        dependsOn("rustDeploy${arch.capitalize()}")
//    }
//    tasks.named("preBuild").configure {
//        dependsOn("rustDeploy${arch.capitalize()}")
//    }
//
//    // Hook up clean tasks
//    tasks.register("clean${arch.capitalize()}", Delete::class) {
//        group = "clean"
//        description = "Deleting built libs for $arch"
//        delete(fileTree("$cargoTargetDirectory/$target/debug") {
//            include("*.so")
//        })
//    }
//    tasks.named("clean").configure {
//        dependsOn("clean${arch.capitalize()}")
//    }
//}

