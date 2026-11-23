package io.github.workforceplanningmanagement.mobile.core

import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import java.time.LocalDate

/** The JSON the core reads and writes: unknown members are ignored, so a newer service does not break an older app. */
internal val Json: kotlinx.serialization.json.Json = kotlinx.serialization.json.Json {
    ignoreUnknownKeys = true
    explicitNulls = false
}

/** An ISO 8601 calendar date (`2026-11-23`), with no time and no zone. */
public object LocalDateSerializer : KSerializer<LocalDate> {
    override val descriptor: kotlinx.serialization.descriptors.SerialDescriptor =
        PrimitiveSerialDescriptor("LocalDate", PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: LocalDate): Unit = encoder.encodeString(value.toString())

    override fun deserialize(decoder: Decoder): LocalDate = LocalDate.parse(decoder.decodeString())
}
