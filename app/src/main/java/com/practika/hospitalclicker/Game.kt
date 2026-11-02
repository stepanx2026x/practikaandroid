package com.practika.hospitalclicker

import kotlin.math.floor
import kotlin.math.pow

/** Инструмент главврача — усиливает клик. */
class Tool(
    val id: String,
    val emoji: String,
    val title: String,
    val description: String,
    val baseCost: Double,
    val power: Double,
    var level: Int = 0,
) {
    /** Цена растёт на 15% с каждым купленным уровнем. */
    val cost: Double get() = floor(baseCost * 1.15.pow(level))
}

/** Предмет для врача: покупается один раз и умножает доход этого врача. */
class Item(
    val id: String,
    val emoji: String,
    val title: String,
    val multiplier: Double,
    val cost: Double,
    /** Сколько врачей этого типа нужно нанять, чтобы предмет стал доступен. */
    val requiredCount: Int,
    var owned: Boolean = false,
)

/** Врач: его можно нанимать (количество), повышать квалификацию и снабжать предметами. */
class Doctor(
    val id: String,
    val emoji: String,
    val title: String,
    val baseCost: Double,
    val baseIncome: Double,
    val items: List<Item>,
    var count: Int = 0,
    var qualification: Int = 0,
) {
    val hireCost: Double get() = floor(baseCost * 1.15.pow(count))

    val qualificationTitle: String get() = QUALIFICATIONS[qualification]
    val nextQualificationTitle: String? get() = QUALIFICATIONS.getOrNull(qualification + 1)
    val canUpgradeQualification: Boolean get() = qualification < QUALIFICATIONS.lastIndex

    /** Повышение квалификации дорожает в 8 раз за каждый уровень. */
    val qualificationCost: Double get() = floor(baseCost * 20 * 8.0.pow(qualification))

    /** Общий множитель: квалификация (×2 за уровень) и купленные предметы. */
    val multiplier: Double
        get() = 2.0.pow(qualification) *
            items.filter { it.owned }.fold(1.0) { acc, item -> acc * item.multiplier }

    val incomePerDoctor: Double get() = baseIncome * multiplier
    val income: Double get() = incomePerDoctor * count

    fun isUnlocked(item: Item) = count >= item.requiredCount

    companion object {
        val QUALIFICATIONS = listOf(
            "Интерн", "Ординатор", "Специалист", "Кандидат наук", "Профессор", "Академик",
        )
    }
}

data class Rank(val title: String, val threshold: Double)

class Game {
    var money = 0.0
    var totalEarned = 0.0
    var patientsHealed = 0L

    val tools = listOf(
        Tool("stethoscope", "🩺", "Стетоскоп главврача", "+1 ₽ за клик", 15.0, 1.0),
        Tool("firstaid", "🧰", "Аптечка", "+5 ₽ за клик", 250.0, 5.0),
        Tool("scalpel", "✂️", "Набор инструментов", "+40 ₽ за клик", 5_000.0, 40.0),
        Tool("defib", "⚡", "Дефибриллятор", "+400 ₽ за клик", 150_000.0, 400.0),
        Tool("bionic", "🦾", "Бионическая рука", "+3 000 ₽ за клик", 2_500_000.0, 3_000.0),
    )

    val doctors = listOf(
        doctor("nurse", "👩‍⚕️", "Медсестра", 25.0, 1.0,
            "💉" to "Одноразовые шприцы",
            "🩹" to "Перевязочный набор",
            "🌡️" to "Электронный градусник"),
        doctor("pills", "💊", "Фармацевт", 110.0, 3.0,
            "📦" to "Склад лекарств",
            "🧾" to "Электронные рецепты",
            "⚖️" to "Точные весы"),
        doctor("doctor", "🧑‍⚕️", "Терапевт", 600.0, 10.0,
            "🩺" to "Профессиональный стетоскоп",
            "📋" to "Электронная медкарта",
            "🩸" to "Цифровой тонометр"),
        doctor("lab", "🧪", "Лаборант", 3_500.0, 40.0,
            "🔬" to "Микроскоп",
            "🧫" to "Чашки Петри",
            "🧬" to "ПЦР-анализатор"),
        doctor("surgeon", "🥼", "Хирург", 20_000.0, 150.0,
            "🔪" to "Титановый скальпель",
            "🧤" to "Стерильные перчатки",
            "💡" to "Операционная лампа"),
        doctor("ambulance", "🚑", "Фельдшер скорой", 110_000.0, 600.0,
            "📟" to "Рация",
            "🫀" to "Портативный дефибриллятор",
            "🚨" to "Мигалка"),
        doctor("mri", "🩻", "Рентгенолог", 650_000.0, 2_500.0,
            "🩻" to "Цифровой рентген",
            "🧲" to "Аппарат МРТ",
            "🖥️" to "ИИ-диагностика"),
        doctor("neuro", "🧠", "Нейрохирург", 4_000_000.0, 12_000.0,
            "🔭" to "Нейромикроскоп",
            "🤖" to "Хирургический робот",
            "🔌" to "Нейроинтерфейс"),
        doctor("professor", "🎓", "Профессор медицины", 30_000_000.0, 60_000.0,
            "📚" to "Научная библиотека",
            "🏛️" to "Собственный НИИ",
            "🏅" to "Нобелевская премия"),
    )

    val ranks = listOf(
        Rank("Фельдшерский пункт", 0.0),
        Rank("Медпункт", 500.0),
        Rank("Поликлиника", 10_000.0),
        Rank("Районная больница", 250_000.0),
        Rank("Городская больница", 5_000_000.0),
        Rank("Областной медцентр", 100_000_000.0),
        Rank("Федеральный медцентр", 2_000_000_000.0),
        Rank("Мировая клиника", 50_000_000_000.0),
    )

    val clickPower: Double
        get() = 1.0 + tools.sumOf { it.power * it.level }

    val incomePerSecond: Double
        get() = doctors.sumOf { it.income }

    val rankIndex: Int
        get() = ranks.indexOfLast { totalEarned >= it.threshold }.coerceAtLeast(0)

    val rank: Rank get() = ranks[rankIndex]
    val nextRank: Rank? get() = ranks.getOrNull(rankIndex + 1)

    /** Прогресс до следующего статуса от 0 до 1. */
    val rankProgress: Double
        get() {
            val next = nextRank ?: return 1.0
            val cur = rank.threshold
            return ((totalEarned - cur) / (next.threshold - cur)).coerceIn(0.0, 1.0)
        }

    fun earn(amount: Double) {
        money += amount
        totalEarned += amount
    }

    /** Клик по пациенту — возвращает заработанную сумму. */
    fun click(): Double {
        val amount = clickPower
        earn(amount)
        patientsHealed++
        return amount
    }

    fun tick(seconds: Double) {
        if (seconds > 0) earn(incomePerSecond * seconds)
    }

    /** Награда за экстренного пациента: 30 секунд дохода, но не меньше 20 кликов. */
    fun emergencyReward(): Double = maxOf(incomePerSecond * 30, clickPower * 20)

    private fun pay(cost: Double): Boolean {
        if (money < cost) return false
        money -= cost
        return true
    }

    fun buyTool(tool: Tool): Boolean = pay(tool.cost).also { if (it) tool.level++ }

    fun hire(doctor: Doctor): Boolean = pay(doctor.hireCost).also { if (it) doctor.count++ }

    fun upgradeQualification(doctor: Doctor): Boolean {
        if (doctor.count == 0 || !doctor.canUpgradeQualification) return false
        return pay(doctor.qualificationCost).also { if (it) doctor.qualification++ }
    }

    fun buyItem(doctor: Doctor, item: Item): Boolean {
        if (item.owned || !doctor.isUnlocked(item)) return false
        return pay(item.cost).also { if (it) item.owned = true }
    }

    fun reset() {
        money = 0.0
        totalEarned = 0.0
        patientsHealed = 0
        tools.forEach { it.level = 0 }
        doctors.forEach { d ->
            d.count = 0
            d.qualification = 0
            d.items.forEach { it.owned = false }
        }
    }

    companion object {
        private val suffixes = listOf("", " тыс.", " млн", " млрд", " трлн", " квдрлн")

        /** Три предмета на врача: ×1.5 / ×2 / ×3, открываются при 1 / 10 / 25 нанятых. */
        private fun doctor(
            id: String, emoji: String, title: String, baseCost: Double, baseIncome: Double,
            vararg items: Pair<String, String>,
        ): Doctor {
            val tiers = listOf(
                Triple(1.5, 8.0, 1),
                Triple(2.0, 60.0, 10),
                Triple(3.0, 600.0, 25),
            )
            val list = items.mapIndexed { i, (emoji2, name) ->
                val (mult, costFactor, req) = tiers[i]
                Item("${id}_$i", emoji2, name, mult, floor(baseCost * costFactor), req)
            }
            return Doctor(id, emoji, title, baseCost, baseIncome, list)
        }

        fun formatMultiplier(m: Double): String =
            if (m == floor(m)) "×${m.toInt()}" else "×$m"

        fun format(value: Double): String {
            if (value < 1_000) {
                return if (value < 10 && value != floor(value)) "%.1f".format(value)
                else "%.0f".format(floor(value))
            }
            var v = value
            var i = 0
            while (v >= 1_000 && i < suffixes.lastIndex) {
                v /= 1_000
                i++
            }
            return "%.2f".format(v).trimEnd('0').trimEnd(',', '.') + suffixes[i]
        }
    }
}
