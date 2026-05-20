import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.ksp)
    alias(libs.plugins.hilt)
    id("kotlin-parcelize")
    id("org.openapi.generator") version "7.4.0"
}

android {
    namespace = "dev.crqch.sunder"
    compileSdk {
        version = release(36)
    }

    defaultConfig {
        applicationId = "dev.crqch.sunder"
        minSdk = 35
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    val secretsFile = rootProject.file("secrets.properties")
    val secrets = Properties().apply {
        if (secretsFile.exists()) {
            load(secretsFile.inputStream())
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            val prodUrl = secrets["API_URL_RELEASE"]
            buildConfigField("String", "BASE_URL", prodUrl.toString())
        }

        debug {
            val debugUrl = secrets["API_URL_DEBUG"] ?: "\"http://127.0.0.1:4000\""
            android.buildFeatures.buildConfig = true
            buildConfigField("String", "BASE_URL", debugUrl.toString())
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17

    }

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.material.icons.extended)
    implementation(libs.androidx.navigation.compose)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.tooling)
    debugImplementation(libs.androidx.compose.ui.test.manifest)

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)
    implementation(libs.hilt.navigation.compose)

    implementation("androidx.datastore:datastore-preferences:1.1.1")

    implementation(libs.androidx.room.runtime)
    implementation(libs.androidx.room.ktx)
    ksp(libs.androidx.room.compiler)

    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.3")

    implementation(libs.retrofit)
    implementation(libs.retrofit.converter.moshi)
    implementation(libs.retrofit.converter.scalars)

    // Moshi for JSON parsing (if you chose moshi above)
    implementation(libs.moshi.kotlin)
    implementation(libs.okhttp.logging.interceptor)

}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

openApiGenerate {
    generatorName.set("kotlin")
    library.set("jvm-retrofit2")
    inputSpec.set("$projectDir/openapi.json")
    outputDir.set("$buildDir/generated/openapi")
    apiPackage.set("dev.crqch.sunder.api")
    modelPackage.set("dev.crqch.sunder.models")
    configOptions.set(
        mapOf(
            "useCoroutines" to "true",
            "serializationLibrary" to "moshi"
        )
    )
}

kotlin {
    sourceSets.main {
        kotlin.srcDir("$buildDir/generated/openapi/src/main/kotlin")
    }
}