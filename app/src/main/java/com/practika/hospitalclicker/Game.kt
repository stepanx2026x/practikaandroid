package com.practika.hospitalclicker

import kotlin.math.cbrt
import kotlin.math.floor
import kotlin.math.pow
import kotlin.random.Random

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

    /** Цена покупки сразу [n] уровней. */
    fun costFor(n: Int): Double = floor(baseCost * 1.15.pow(level) * bulkFactor(n))
}

/** Сумма геометрической прогрессии цен: 1 + 1.15 + 1.15² + … (n слагаемых). */
fun bulkFactor(n: Int): Double {
    var sum = 0.0
    var term = 1.0
    repeat(n) {
        sum += term
        term *= 1.15
    }
    return sum
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
    /** Цена найма без учёта карты и наследия (см. [Game.hireCost]). */
    val baseHireCost: Double get() = baseCost * 1.15.pow(count)

    val qualificationTitle: String get() = QUALIFICATIONS[qualification]
    val nextQualificationTitle: String? get() = QUALIFICATIONS.getOrNull(qualification + 1)
    val canUpgradeQualification: Boolean get() = qualification < QUALIFICATIONS.lastIndex

    /** Повышение квалификации дорожает в 8 раз за каждый уровень. */
    val qualificationCost: Double get() = floor(baseCost * 20 * 8.0.pow(qualification))

    /** Собственный множитель врача: квалификация (×2 за уровень) и купленные предметы. */
    val multiplier: Double
        get() = 2.0.pow(qualification) *
            items.filter { it.owned }.fold(1.0) { acc, item -> acc * item.multiplier }

    /** Доход без глобальных бонусов (звёзды, карта, события). */
    val rawIncomePerDoctor: Double get() = baseIncome * multiplier
    val rawIncome: Double get() = rawIncomePerDoctor * count

    fun isUnlocked(item: Item) = count >= item.requiredCount

    companion object {
        val QUALIFICATIONS = listOf(
            "Интерн", "Ординатор", "Специалист", "Кандидат наук", "Профессор", "Академик",
        )
    }
}

data class Rank(val title: String, val threshold: Double)

/** Карта (локация больницы) со своими модификаторами и болезнями. */
class HospitalMap(
    val id: String,
    val emoji: String,
    val title: String,
    val description: String,
    /** Сколько звёзд репутации нужно заработать за всё время, чтобы открыть карту. */
    val starsRequired: Long,
    val incomeMult: Double = 1.0,
    val clickMult: Double = 1.0,
    val hireCostMult: Double = 1.0,
    /** Доля вспышек болезней среди всех событий. */
    val outbreakChance: Double = 0.5,
    val background: String,
)

/** Улучшение наследия — покупается за звёзды и сохраняется после престижа. */
class Legacy(
    val id: String,
    val emoji: String,
    val title: String,
    val description: String,
    val cost: Long,
    var owned: Boolean = false,
)

enum class EventKind { OUTBREAK, BONUS }

class EventType(
    val id: String,
    val emoji: String,
    val title: String,
    val description: String,
    val kind: EventKind,
    val durationSec: Double,
    val incomeMult: Double = 1.0,
    val clickMult: Double = 1.0,
    /** Для вспышек: сколько раз нужно вылечить пациента, чтобы победить болезнь. */
    val cureClicks: Int = 0,
    /** Награда в секундах дохода: за победу над вспышкой или сразу (для мгновенных бонусов). */
    val rewardSeconds: Double = 0.0,
    /** На каких картах встречается (null — на всех). */
    val maps: Set<String>? = null,
    val weight: Int = 10,
)

class Achievement(
    val id: String,
    val emoji: String,
    val title: String,
    val description: String,
    val condition: (Game) -> Boolean,
)

class ActiveEvent(val type: EventType, var remaining: Double, var progress: Int = 0)

class Game {
    var money = 0.0
    /** Заработано за текущий забег (с последнего престижа). */
    var totalEarned = 0.0
    var lifetimeEarned = 0.0
    var patientsHealed = 0L

    // --- Престиж ---
    var stars = 0L
    var starsSpent = 0L
    var prestigeCount = 0
    val availableStars: Long get() = stars - starsSpent

    var mapId = "village"
    var event: ActiveEvent? = null
    var nextEventIn = 40.0

    // --- Статистика ---
    var totalClicks = 0L
    var clickEarned = 0.0
    var outbreaksCured = 0
    var outbreaksFailed = 0
    var emergencies = 0
    var playTimeSec = 0.0
    var bestIncome = 0.0
    var startedAt = System.currentTimeMillis()
    val curedEventIds = mutableSetOf<String>()
    val visitedMaps = mutableSetOf("village")

    /** Сколько покупать за раз: 1, 10 или 100. */
    var buyAmount = 1

    val unlockedAchievements = mutableSetOf<String>()

    /** Сообщения для игрока (начало/конец событий). */
    var onMessage: ((String) -> Unit)? = null

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
        doctor("pediatrician", "👶", "Педиатр", 300.0, 6.0,
            "🧸" to "Игрушки для пациентов",
            "🍭" to "Леденцы за смелость",
            "🎨" to "Яркая палата"),
        doctor("doctor", "🧑‍⚕️", "Терапевт", 600.0, 10.0,
            "🩺" to "Профессиональный стетоскоп",
            "📋" to "Электронная медкарта",
            "🩸" to "Цифровой тонометр"),
        doctor("dentist", "🦷", "Стоматолог", 1_500.0, 22.0,
            "🪥" to "Ультразвуковая щётка",
            "🪑" to "Стоматологическое кресло",
            "😁" to "Виниры"),
        doctor("lab", "🧪", "Лаборант", 3_500.0, 40.0,
            "🔬" to "Микроскоп",
            "🧫" to "Чашки Петри",
            "🧬" to "ПЦР-анализатор"),
        doctor("ophthalmologist", "👁️", "Офтальмолог", 8_000.0, 80.0,
            "👓" to "Набор линз",
            "🔤" to "Таблица для проверки зрения",
            "🔦" to "Лазерный офтальмоскоп"),
        doctor("surgeon", "🥼", "Хирург", 20_000.0, 150.0,
            "🔪" to "Титановый скальпель",
            "🧤" to "Стерильные перчатки",
            "💡" to "Операционная лампа"),
        doctor("cardiologist", "❤️", "Кардиолог", 50_000.0, 320.0,
            "📈" to "Кардиограф",
            "⌚" to "Холтер-монитор",
            "💓" to "Кардиостимулятор"),
        doctor("ambulance", "🚑", "Фельдшер скорой", 110_000.0, 600.0,
            "📟" to "Рация",
            "🫀" to "Портативный дефибриллятор",
            "🚨" to "Мигалка"),
        doctor("anesthesiologist", "😴", "Анестезиолог", 280_000.0, 1_300.0,
            "😷" to "Кислородная маска",
            "💤" to "Наркозный аппарат",
            "📺" to "Монитор пациента"),
        doctor("mri", "🩻", "Рентгенолог", 650_000.0, 2_500.0,
            "🩻" to "Цифровой рентген",
            "🧲" to "Аппарат МРТ",
            "🖥️" to "ИИ-диагностика"),
        doctor("psychotherapist", "🛋️", "Психотерапевт", 1_600_000.0, 5_500.0,
            "🪴" to "Уютный кабинет",
            "📓" to "Дневник настроения",
            "🎧" to "Расслабляющая музыка"),
        doctor("neuro", "🧠", "Нейрохирург", 4_000_000.0, 12_000.0,
            "🔭" to "Нейромикроскоп",
            "🤖" to "Хирургический робот",
            "🔌" to "Нейроинтерфейс"),
        doctor("oncologist", "🎗️", "Онколог", 11_000_000.0, 28_000.0,
            "💊" to "Таргетные препараты",
            "☢️" to "Линейный ускоритель",
            "🛡️" to "Иммунотерапия"),
        doctor("professor", "🎓", "Профессор медицины", 30_000_000.0, 60_000.0,
            "📚" to "Научная библиотека",
            "🏛️" to "Собственный НИИ",
            "🏅" to "Нобелевская премия"),
        doctor("geneticist", "🧬", "Генетик", 120_000_000.0, 200_000.0,
            "🧪" to "Секвенатор ДНК",
            "✂️" to "Редактор генов",
            "💻" to "Суперкомпьютер"),
        doctor("transplantologist", "🫁", "Трансплантолог", 500_000_000.0, 700_000.0,
            "🧊" to "Криоконтейнер",
            "🫁" to "Искусственные органы",
            "🖨️" to "Биопринтер"),
        doctor("nanosurgeon", "🦠", "Нанохирург", 2_500_000_000.0, 3_000_000.0,
            "🐜" to "Нанороботы",
            "⚛️" to "Квантовый сканер",
            "💠" to "Умные наноматериалы"),
        doctor("spacedoctor", "🚀", "Космический врач", 15_000_000_000.0, 15_000_000.0,
            "🛰️" to "Орбитальная станция",
            "👨‍🚀" to "Медицинский скафандр",
            "🌌" to "Межзвёздная клиника"),
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
        Rank("Орбитальный госпиталь", 1_000_000_000_000.0),
        Rank("Галактический медцентр", 50_000_000_000_000.0),
    )

    val maps = listOf(
        HospitalMap("village", "🏡", "Сельская больница", "Тихое место для старта. Без бонусов и штрафов.",
            0, background = "#EAF6F6"),
        HospitalMap("city", "🏙️", "Городская клиника", "Доход ×1.5. Вспышки болезней чуть чаще.",
            3, incomeMult = 1.5, outbreakChance = 0.6, background = "#E6EEF9"),
        HospitalMap("mountains", "🏔️", "Горный санаторий", "Клик ×3, доход ×1.25. Чистый воздух — эпидемии реже. Бывают обморожения.",
            10, incomeMult = 1.25, clickMult = 3.0, outbreakChance = 0.3, background = "#EEF3F8"),
        HospitalMap("island", "🏝️", "Тропический остров", "Доход ×2.5, но малярия и частые вспышки.",
            30, incomeMult = 2.5, outbreakChance = 0.7, background = "#FFF4DD"),
        HospitalMap("megapolis", "🌆", "Мегаполис", "Доход ×4, но найм врачей на 25% дороже. Смог.",
            100, incomeMult = 4.0, hireCostMult = 1.25, outbreakChance = 0.6, background = "#ECE8F6"),
        HospitalMap("mars", "🔴", "Колония на Марсе", "Доход ×8, клик ×4. Опасный марсианский вирус.",
            300, incomeMult = 8.0, clickMult = 4.0, outbreakChance = 0.75, background = "#FBE8E2"),
    )

    val legacies = listOf(
        Legacy("hands", "✋", "Золотые руки", "Клик ×2 навсегда", 2),
        Legacy("staff", "🧑‍🏫", "Опытные кадры", "Найм врачей на 15% дешевле", 3),
        Legacy("night", "🌙", "Ночные смены", "Доход офлайн до 12 часов вместо 3", 4),
        Legacy("capital", "💼", "Стартовый капитал", "Каждый забег начинается с 50 тыс. ₽", 5),
        Legacy("ambulance", "🚨", "Быстрая скорая", "Экстренные пациенты появляются в 2 раза чаще", 5),
        Legacy("immunity", "🛡️", "Коллективный иммунитет", "Вспышки болезней вдвое слабее, штраф вдвое меньше", 8),
        Legacy("grants", "🍀", "Удачные гранты", "Хорошие события длятся в 2 раза дольше", 10),
        Legacy("fame", "🌟", "Мировая слава", "Каждая звезда даёт +3% вместо +2%", 25),
    )

    val achievements: List<Achievement> = buildList {
        fun a(id: String, emoji: String, title: String, desc: String, cond: (Game) -> Boolean) =
            add(Achievement(id, emoji, title, desc, cond))

        listOf(
            1L to "Первый пациент", 100L to "Участковый", 1_000L to "Народный доктор",
            10_000L to "Тысяча рук", 100_000L to "Спаситель района", 1_000_000L to "Миллион здоровых",
        ).forEach { (n, t) -> a("patients_$n", "🤒", t, "Вылечить ${format(n.toDouble())} пациентов") { it.patientsHealed >= n } }

        listOf(
            1e3 to "Первая зарплата", 1e6 to "Миллионер в халате", 1e9 to "Медицинский магнат",
            1e12 to "Триллион на здоровье", 1e15 to "Бюджет континента",
        ).forEach { (n, t) -> a("earned_$n", "💰", t, "Заработать ${format(n)} ₽ за всё время") { it.lifetimeEarned >= n } }

        listOf(
            10.0 to "Своя практика", 1e3 to "Слаженная команда", 1e5 to "Конвейер здоровья",
            1e7 to "Медицинская империя", 1e9 to "Здравоохранение планеты",
        ).forEach { (n, t) -> a("income_$n", "📈", t, "Доход ${format(n)} ₽/сек") { it.incomePerSecond >= n } }

        listOf(
            10 to "Штат укомплектован", 100 to "Большой коллектив", 500 to "Целая армия в халатах",
            1_000 to "Тысяча специалистов",
        ).forEach { (n, t) -> a("staff_$n", "👨‍⚕️", t, "Нанять всего $n врачей") { g -> g.doctors.sumOf { it.count } >= n } }

        a("all_types", "🧑‍🤝‍🧑", "Многопрофильная больница", "Нанять хотя бы одного врача каждой специальности") { g -> g.doctors.all { it.count > 0 } }
        a("academic", "🎓", "Академик", "Поднять любого врача до квалификации «Академик»") { g -> g.doctors.any { !it.canUpgradeQualification } }
        listOf(10 to "Хорошо оснащены", 30 to "Склад чудес", 60 to "Всё для всех").forEach { (n, t) ->
            a("items_$n", "🧰", t, "Купить $n предметов для врачей") { g -> g.doctors.sumOf { d -> d.items.count { it.owned } } >= n }
        }
        listOf(1_000L to "Пальцы-молнии", 10_000L to "Неутомимый главврач", 100_000L to "Клик-легенда").forEach { (n, t) ->
            a("clicks_$n", "👆", t, "Сделать ${format(n.toDouble())} кликов") { it.totalClicks >= n }
        }
        listOf(1 to "Первая победа", 10 to "Эпидемиолог", 50 to "Гроза вирусов").forEach { (n, t) ->
            a("cured_$n", "🦠", t, "Остановить $n вспышек болезней") { it.outbreaksCured >= n }
        }
        a("cured_plague", "🐀", "Победитель чумы", "Остановить чуму") { "plague" in it.curedEventIds }
        a("cured_mars", "👾", "Ксенобиолог", "Остановить марсианский вирус") { "marsvirus" in it.curedEventIds }
        listOf(1 to "Скорая помощь", 25 to "Ангел на колёсах", 100 to "Король скорой").forEach { (n, t) ->
            a("emergency_$n", "🚑", t, "Спасти $n экстренных пациентов") { it.emergencies >= n }
        }
        listOf(1 to "Аккредитован", 5 to "Пять звёзд", 20 to "Вечный главврач").forEach { (n, t) ->
            a("prestige_$n", "⭐", t, "Пройти аккредитацию $n раз") { it.prestigeCount >= n }
        }
        listOf(10L to "Известная больница", 100L to "Звёздная клиника").forEach { (n, t) ->
            a("stars_$n", "🌟", t, "Набрать $n звёзд репутации") { it.stars >= n }
        }
        a("mars", "🔴", "Марсианский доктор", "Перевезти больницу на Марс") { "mars" in it.visitedMaps }
        a("traveler", "🗺️", "Путешественник", "Побывать на всех картах") { g -> g.maps.all { it.id in g.visitedMaps } }
        a("hour", "⏰", "Ночная смена", "Провести в игре 1 час") { it.playTimeSec >= 3600 }
    }

    val events = listOf(
        // Вспышки болезней
        EventType("flu", "🤧", "Эпидемия гриппа", "Доход снижен. Лечите пациентов, чтобы остановить грипп!",
            EventKind.OUTBREAK, 45.0, incomeMult = 0.6, cureClicks = 60, rewardSeconds = 90.0, weight = 12),
        EventType("measles", "🔴", "Вспышка кори", "Корь распространяется! Доход сильно снижен.",
            EventKind.OUTBREAK, 50.0, incomeMult = 0.5, cureClicks = 90, rewardSeconds = 120.0, weight = 9),
        EventType("cholera", "💧", "Холера", "Заражена вода в районе. Срочно лечите больных!",
            EventKind.OUTBREAK, 55.0, incomeMult = 0.4, cureClicks = 120, rewardSeconds = 180.0,
            maps = setOf("village", "city", "island", "megapolis"), weight = 7),
        EventType("plague", "🐀", "Чума", "Чёрная смерть! Доход почти остановился.",
            EventKind.OUTBREAK, 60.0, incomeMult = 0.25, cureClicks = 160, rewardSeconds = 300.0,
            maps = setOf("village", "city", "island", "megapolis", "mountains"), weight = 4),
        EventType("typhus", "🦠", "Тиф", "Тиф косит пациентов. Доход снижен.",
            EventKind.OUTBREAK, 50.0, incomeMult = 0.5, cureClicks = 100, rewardSeconds = 150.0,
            maps = setOf("village", "city", "megapolis"), weight = 6),
        EventType("malaria", "🦟", "Малярия", "Комары разносят малярию по острову!",
            EventKind.OUTBREAK, 50.0, incomeMult = 0.5, cureClicks = 100, rewardSeconds = 180.0,
            maps = setOf("island"), weight = 14),
        EventType("frostbite", "🥶", "Лавина обморожений", "Туристы попали под лавину!",
            EventKind.OUTBREAK, 45.0, incomeMult = 0.6, cureClicks = 70, rewardSeconds = 120.0,
            maps = setOf("mountains"), weight = 12),
        EventType("smog", "🌫️", "Смог-кашель", "Город накрыл смог, все кашляют.",
            EventKind.OUTBREAK, 50.0, incomeMult = 0.5, cureClicks = 100, rewardSeconds = 150.0,
            maps = setOf("megapolis"), weight = 14),
        EventType("marsvirus", "👾", "Марсианский вирус", "Неизвестный патоген из марсианского грунта!",
            EventKind.OUTBREAK, 60.0, incomeMult = 0.2, cureClicks = 220, rewardSeconds = 400.0,
            maps = setOf("mars"), weight = 16),
        // Хорошие события
        EventType("grant", "💰", "Госфинансирование", "Доход ×2!",
            EventKind.BONUS, 30.0, incomeMult = 2.0, weight = 10),
        EventType("article", "📰", "Статья в медицинском журнале", "Пациенты в восторге — клик ×5!",
            EventKind.BONUS, 20.0, clickMult = 5.0, weight = 10),
        EventType("interns", "🎓", "Выпуск интернов", "Свежие руки — доход ×1.5.",
            EventKind.BONUS, 60.0, incomeMult = 1.5, weight = 8),
        EventType("vaccine", "💉", "Массовая вакцинация", "Пациенты здоровее — доход ×1.3.",
            EventKind.BONUS, 45.0, incomeMult = 1.3, weight = 8),
        EventType("donation", "🎁", "Благотворительный фонд", "Пожертвование больнице!",
            EventKind.BONUS, 0.0, rewardSeconds = 120.0, weight = 6),
    )

    // ---------- Модификаторы ----------

    val map: HospitalMap get() = maps.first { it.id == mapId }

    fun hasLegacy(id: String) = legacies.any { it.id == id && it.owned }

    fun isMapUnlocked(m: HospitalMap) = stars >= m.starsRequired

    /** Бонус от звёзд репутации: +2% (или +3%) за каждую. */
    val starBonus: Double get() = 1.0 + stars * (if (hasLegacy("fame")) 0.03 else 0.02)

    /** Насколько событие влияет на доход/клик (иммунитет ослабляет вспышки). */
    private fun eventMult(mult: Double, kind: EventKind): Double =
        if (kind == EventKind.OUTBREAK && hasLegacy("immunity") && mult < 1) 1 - (1 - mult) / 2 else mult

    private val eventIncomeMult: Double
        get() = event?.let { eventMult(it.type.incomeMult, it.type.kind) } ?: 1.0

    private val eventClickMult: Double
        get() = event?.let { eventMult(it.type.clickMult, it.type.kind) } ?: 1.0

    /** Каждое достижение даёт +1% к доходу и клику. */
    val achievementBonus: Double get() = 1.0 + unlockedAchievements.size * 0.01

    val clickPower: Double
        get() = (1.0 + tools.sumOf { it.power * it.level }) * starBonus * achievementBonus * map.clickMult *
            (if (hasLegacy("hands")) 2.0 else 1.0) * eventClickMult

    /** Пассивный доход без учёта текущего события. */
    val baseIncomePerSecond: Double
        get() = doctors.sumOf { it.rawIncome } * globalIncomeMult

    private val globalIncomeMult: Double get() = starBonus * achievementBonus * map.incomeMult

    val incomePerSecond: Double get() = baseIncomePerSecond * eventIncomeMult

    /** Полный множитель дохода врача (для отображения в карточке). */
    fun doctorIncome(d: Doctor): Double = d.rawIncome * globalIncomeMult * eventIncomeMult
    fun doctorIncomePerOne(d: Doctor): Double = d.rawIncomePerDoctor * globalIncomeMult * eventIncomeMult

    /** Цена найма [n] врачей сразу. */
    fun hireCost(d: Doctor, n: Int = 1): Double =
        floor(d.baseHireCost * bulkFactor(n) * map.hireCostMult * (if (hasLegacy("staff")) 0.85 else 1.0))

    val offlineLimitSec: Double get() = if (hasLegacy("night")) 12 * 3600.0 else 3 * 3600.0

    // ---------- Статусы ----------

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

    // ---------- Престиж ----------

    /** Сколько звёзд даст престиж сейчас: кубический корень из заработанного за забег / 1 млн. */
    val pendingStars: Long get() = floor(cbrt(totalEarned / PRESTIGE_UNIT)).toLong()

    /** Сколько всего нужно заработать за забег для следующей звезды. */
    val nextStarAt: Double get() = (pendingStars + 1.0).pow(3) * PRESTIGE_UNIT

    fun canPrestige() = pendingStars >= 1

    /** Аккредитация: сбрасывает больницу, начисляет звёзды репутации. */
    fun prestige(): Long {
        val gained = pendingStars
        if (gained < 1) return 0
        stars += gained
        prestigeCount++
        resetRun()
        return gained
    }

    private fun resetRun() {
        money = if (hasLegacy("capital")) 50_000.0 else 0.0
        totalEarned = 0.0
        tools.forEach { it.level = 0 }
        doctors.forEach { d ->
            d.count = 0
            d.qualification = 0
            d.items.forEach { it.owned = false }
        }
        event = null
        nextEventIn = 40.0
    }

    fun buyLegacy(l: Legacy): Boolean {
        if (l.owned || availableStars < l.cost) return false
        starsSpent += l.cost
        l.owned = true
        if (l.id == "capital" && totalEarned == 0.0 && money < 50_000) money = 50_000.0
        return true
    }

    fun selectMap(m: HospitalMap): Boolean {
        if (!isMapUnlocked(m)) return false
        mapId = m.id
        visitedMaps += m.id
        // Болезнь другой местности не переезжает вместе с больницей
        event?.let { if (it.type.maps != null && m.id !in it.type.maps) event = null }
        return true
    }

    // ---------- Игровой процесс ----------

    fun earn(amount: Double) {
        money += amount
        totalEarned += amount
        lifetimeEarned += amount
    }

    /** Клик по пациенту — возвращает заработанную сумму. */
    fun click(): Double {
        val amount = clickPower
        earn(amount)
        patientsHealed++
        totalClicks++
        clickEarned += amount
        val e = event
        if (e != null && e.type.kind == EventKind.OUTBREAK) {
            e.progress++
            if (e.progress >= e.type.cureClicks) cureOutbreak(e)
        }
        return amount
    }

    private fun cureOutbreak(e: ActiveEvent) {
        event = null
        outbreaksCured++
        curedEventIds += e.type.id
        val reward = maxOf(baseIncomePerSecond * e.type.rewardSeconds, clickPower * 50)
        earn(reward)
        scheduleNextEvent()
        onMessage?.invoke("✅ ${e.type.title} побеждена! Награда: ${format(reward)} ₽")
    }

    fun tick(seconds: Double) {
        if (seconds <= 0) return
        earn(incomePerSecond * seconds)
        playTimeSec += seconds
        if (incomePerSecond > bestIncome) bestIncome = incomePerSecond
        checkAchievements()

        val e = event
        if (e != null) {
            e.remaining -= seconds
            if (e.remaining <= 0) {
                event = null
                scheduleNextEvent()
                if (e.type.kind == EventKind.OUTBREAK) {
                    outbreaksFailed++
                    val lossShare = if (hasLegacy("immunity")) 0.05 else 0.10
                    val loss = money * lossShare
                    money -= loss
                    onMessage?.invoke("☠️ ${e.type.title} не остановлена. Потрачено на лечение: ${format(loss)} ₽")
                } else {
                    onMessage?.invoke("${e.type.emoji} «${e.type.title}» закончилось")
                }
            }
        } else {
            nextEventIn -= seconds
            if (nextEventIn <= 0) startRandomEvent()
        }
    }

    private fun scheduleNextEvent() {
        nextEventIn = Random.nextDouble(50.0, 110.0)
    }

    fun startRandomEvent() {
        val kind = if (Random.nextDouble() < map.outbreakChance) EventKind.OUTBREAK else EventKind.BONUS
        val pool = events.filter { it.kind == kind && (it.maps == null || mapId in it.maps) }
        val type = weightedRandom(pool)
        if (type.durationSec <= 0) {
            // Мгновенное событие
            val reward = maxOf(baseIncomePerSecond * type.rewardSeconds, clickPower * 30)
            earn(reward)
            scheduleNextEvent()
            onMessage?.invoke("${type.emoji} ${type.title}: +${format(reward)} ₽")
            return
        }
        val duration = if (type.kind == EventKind.BONUS && hasLegacy("grants")) type.durationSec * 2 else type.durationSec
        event = ActiveEvent(type, duration)
        val prefix = if (type.kind == EventKind.OUTBREAK) "⚠️ " else ""
        onMessage?.invoke("$prefix${type.emoji} ${type.title}! ${type.description}")
    }

    private fun weightedRandom(pool: List<EventType>): EventType {
        var r = Random.nextInt(pool.sumOf { it.weight })
        for (e in pool) {
            r -= e.weight
            if (r < 0) return e
        }
        return pool.last()
    }

    /** Награда за экстренного пациента: 30 секунд дохода, но не меньше 20 кликов. */
    fun emergencyReward(): Double = maxOf(baseIncomePerSecond * 30, clickPower * 20)

    fun rescueEmergency(): Double {
        val reward = emergencyReward()
        earn(reward)
        patientsHealed++
        emergencies++
        return reward
    }

    fun checkAchievements() {
        for (a in achievements) {
            if (a.id !in unlockedAchievements && a.condition(this)) {
                unlockedAchievements += a.id
                onMessage?.invoke("🏆 Достижение: ${a.title}! (+1% к доходу и клику)")
            }
        }
    }

    private fun pay(cost: Double): Boolean {
        if (money < cost) return false
        money -= cost
        return true
    }

    fun buyTool(tool: Tool, n: Int = buyAmount): Boolean = pay(tool.costFor(n)).also { if (it) tool.level += n }

    fun hire(doctor: Doctor, n: Int = buyAmount): Boolean = pay(hireCost(doctor, n)).also { if (it) doctor.count += n }

    fun upgradeQualification(doctor: Doctor): Boolean {
        if (doctor.count == 0 || !doctor.canUpgradeQualification) return false
        return pay(doctor.qualificationCost).also { if (it) doctor.qualification++ }
    }

    fun buyItem(doctor: Doctor, item: Item): Boolean {
        if (item.owned || !doctor.isUnlocked(item)) return false
        return pay(item.cost).also { if (it) item.owned = true }
    }

    /** Полный сброс, включая престиж. */
    fun reset() {
        stars = 0
        starsSpent = 0
        prestigeCount = 0
        lifetimeEarned = 0.0
        patientsHealed = 0
        mapId = "village"
        legacies.forEach { it.owned = false }
        totalClicks = 0
        clickEarned = 0.0
        outbreaksCured = 0
        outbreaksFailed = 0
        emergencies = 0
        playTimeSec = 0.0
        bestIncome = 0.0
        startedAt = System.currentTimeMillis()
        curedEventIds.clear()
        visitedMaps.clear()
        visitedMaps += "village"
        unlockedAchievements.clear()
        resetRun()
    }

    companion object {
        const val PRESTIGE_UNIT = 1_000_000.0

        private val suffixes = listOf("", " тыс.", " млн", " млрд", " трлн", " квдрлн", " квнтлн")

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
            if (m == floor(m)) "×${m.toInt()}" else "×${"%.2f".format(m).trimEnd('0').trimEnd(',', '.')}"

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
