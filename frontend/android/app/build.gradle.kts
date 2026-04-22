import java.util.Properties

plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

val releaseConfigProperties = Properties().apply {
    listOf("local.properties", "key.properties")
        .map { rootProject.file(it) }
        .filter { it.exists() }
        .forEach { file ->
            file.inputStream().use { load(it) }
        }
}

fun releaseConfigValue(propertyKey: String, envKey: String): String? {
    val propertyValue = releaseConfigProperties.getProperty(propertyKey)?.trim().orEmpty()
    if (propertyValue.isNotEmpty()) {
        return propertyValue
    }

    val envValue = System.getenv(envKey)?.trim().orEmpty()
    if (envValue.isNotEmpty()) {
        return envValue
    }

    return null
}

val releaseApplicationId = releaseConfigValue("applicationId", "ANDROID_APPLICATION_ID")
val releaseKeystorePath = releaseConfigValue("storeFile", "ANDROID_KEYSTORE_PATH")
val releaseKeystorePassword = releaseConfigValue("storePassword", "ANDROID_KEYSTORE_PASSWORD")
val releaseKeyAlias = releaseConfigValue("keyAlias", "ANDROID_KEY_ALIAS")
val releaseKeyPassword = releaseConfigValue("keyPassword", "ANDROID_KEY_PASSWORD")

val releaseConfigErrors = buildList {
    if (releaseApplicationId == null) add("ANDROID_APPLICATION_ID / applicationId")
    if (releaseKeystorePath == null) add("ANDROID_KEYSTORE_PATH / storeFile")
    if (releaseKeystorePassword == null) add("ANDROID_KEYSTORE_PASSWORD / storePassword")
    if (releaseKeyAlias == null) add("ANDROID_KEY_ALIAS / keyAlias")
    if (releaseKeyPassword == null) add("ANDROID_KEY_PASSWORD / keyPassword")
}

val releaseTaskRequested = gradle.startParameter.taskNames.any { taskName ->
    taskName.contains("release", ignoreCase = true) ||
        taskName.contains("bundle", ignoreCase = true)
}

android {
    namespace = "dev.phillui.personal_inventory"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_17.toString()
    }

    defaultConfig {
        applicationId = releaseApplicationId ?: "dev.phillui.personal_inventory"
        minSdk = flutter.minSdkVersion
        targetSdk = 34
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        if (releaseConfigErrors.isEmpty()) {
            create("release") {
                storeFile = rootProject.file(releaseKeystorePath!!)
                storePassword = releaseKeystorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            if (releaseConfigErrors.isEmpty()) {
                signingConfig = signingConfigs.getByName("release")
            }
        }
    }
}

if (releaseTaskRequested && releaseConfigErrors.isNotEmpty()) {
    throw GradleException(
        "Android release build requires ${releaseConfigErrors.joinToString(", ")} " +
            "from frontend/android/key.properties, frontend/android/local.properties, or CI env vars."
    )
}

flutter {
    source = "../.."
}
