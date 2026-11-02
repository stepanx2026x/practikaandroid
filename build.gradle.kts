plugins {
    java
    application
}

group = "sakura"
version = "1.0"

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
    // Собираем под Java 17, чтобы .jar запускался на любой Java 17+
    options.release.set(17)
}

application {
    mainClass.set("sakura.Main")
}

tasks.jar {
    archiveFileName.set("SakuraYokai.jar")
    manifest {
        attributes["Main-Class"] = "sakura.Main"
    }
}
