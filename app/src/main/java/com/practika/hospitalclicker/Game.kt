package com.practika.hospitalclicker

import kotlin.math.floor
import kotlin.math.pow

/** Тип улучшения: усиливает клик или даёт пассивный доход в секунду. */
enum class UpgradeKind { CLICK, AUTO }

data class Upgrade(
    val id: String,
    val emoji: String,
    val title: String,
    val description: String,
    val kind: UpgradeKind,
    val baseCost: Double,
    val power: Double,
    var level: Int = 0,
) {
    /** Цена растёт на 15% с каждым купленным уровнем. */
    val cost: Double get() = floor(baseCost * 1.15.pow(level))
}

data class Rank(val title: String, val threshold: Double)

class Game {
    var money = 0.0
    var totalEarned = 0.0
    var patientsHealed = 0L

    val upgrades = listOf(
        Upgrade("stethoscope", "🩺", "Стетоскоп", "+1 ₽ за клик", UpgradeKind.CLICK, 15.0, 1.0),
        Upgrade("nurse", "👩‍⚕️", "Медсестра", "+1 ₽/сек", UpgradeKind.AUTO, 25.0, 1.0),
        Upgrade("pills", "💊", "Аптека", "+3 ₽/сек", UpgradeKind.AUTO, 110.0, 3.0),
        Upgrade("firstaid", "🧰", "Аптечка", "+5 ₽ за клик", UpgradeKind.CLICK, 250.0, 5.0),
        Upgrade("doctor", "🧑‍⚕️", "Терапевт", "+10 ₽/сек", UpgradeKind.AUTO, 600.0, 10.0),
        Upgrade("lab", "🧪", "Лаборатория", "+40 ₽/сек", UpgradeKind.AUTO, 3_500.0, 40.0),
        Upgrade("scalpel", "🔪", "Скальпель хирурга", "+40 ₽ за клик", UpgradeKind.CLICK, 5_000.0, 40.0),
        Upgrade("surgeon", "🥼", "Хирург", "+150 ₽/сек", UpgradeKind.AUTO, 20_000.0, 150.0),
        Upgrade("ambulance", "🚑", "Скорая помощь", "+600 ₽/сек", UpgradeKind.AUTO, 110_000.0, 600.0),
        Upgrade("defib", "⚡", "Дефибриллятор", "+400 ₽ за клик", UpgradeKind.CLICK, 150_000.0, 400.0),
        Upgrade("mri", "🧲", "Аппарат МРТ", "+2 500 ₽/сек", UpgradeKind.AUTO, 650_000.0, 2_500.0),
        Upgrade("helicopter", "🚁", "Санавиация", "+12 000 ₽/сек", UpgradeKind.AUTO, 4_000_000.0, 12_000.0),
        Upgrade("wing", "🏥", "Новый корпус", "+60 000 ₽/сек", UpgradeKind.AUTO, 30_000_000.0, 60_000.0),
        Upgrade("nobel", "🏅", "Нобелевская премия", "+250 000 ₽/сек", UpgradeKind.AUTO, 250_000_000.0, 250_000.0),
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
        get() = 1.0 + upgrades.filter { it.kind == UpgradeKind.CLICK }.sumOf { it.power * it.level }

    val incomePerSecond: Double
        get() = upgrades.filter { it.kind == UpgradeKind.AUTO }.sumOf { it.power * it.level }

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

    fun buy(upgrade: Upgrade): Boolean {
        val cost = upgrade.cost
        if (money < cost) return false
        money -= cost
        upgrade.level++
        return true
    }

    fun reset() {
        money = 0.0
        totalEarned = 0.0
        patientsHealed = 0
        upgrades.forEach { it.level = 0 }
    }

    companion object {
        private val suffixes = listOf("", " тыс.", " млн", " млрд", " трлн", " квдрлн")

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
