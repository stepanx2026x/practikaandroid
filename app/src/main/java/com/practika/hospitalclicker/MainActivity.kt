package com.practika.hospitalclicker

import android.animation.ObjectAnimator
import android.app.Activity
import android.app.AlertDialog
import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast
import kotlin.random.Random

class MainActivity : Activity() {

    private val game = Game()
    private val handler = Handler(Looper.getMainLooper())

    private lateinit var moneyText: TextView
    private lateinit var incomeText: TextView
    private lateinit var rankText: TextView
    private lateinit var nextRankText: TextView
    private lateinit var rankProgress: ProgressBar
    private lateinit var hospitalView: HospitalView
    private lateinit var newsText: TextView
    private lateinit var patientsText: TextView
    private lateinit var emergencyButton: TextView
    private lateinit var clickArea: FrameLayout
    private lateinit var upgradesContainer: LinearLayout

    private lateinit var eventBanner: LinearLayout
    private lateinit var eventEmoji: TextView
    private lateinit var eventTitle: TextView
    private lateinit var eventDesc: TextView
    private lateinit var eventProgress: ProgressBar
    private lateinit var eventTimer: TextView

    private lateinit var tabs: List<TextView>
    private lateinit var tabContents: List<LinearLayout>

    /** Функции, обновляющие карточки магазина при каждом тике. */
    private val updaters = mutableListOf<() -> Unit>()

    private var lastTick = 0L
    private var lastRankIndex = 0
    private var nextEmergencyAt = 0L
    private var emergencyHideAt = 0L
    private var ticksSinceSave = 0

    private var orbitersKey = ""
    private var newsCountdown = 0

    private val tickRunnable = object : Runnable {
        override fun run() {
            val now = SystemClock.elapsedRealtime()
            game.tick((now - lastTick) / 1000.0)
            lastTick = now
            updateEmergency(now)
            if (--newsCountdown <= 0) {
                newsCountdown = NEWS_TICKS
                newsText.text = News.next(game)
            }
            refreshUi()
            if (++ticksSinceSave >= 50) { // ~ каждые 5 секунд
                ticksSinceSave = 0
                save()
            }
            handler.postDelayed(this, TICK_MS)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        moneyText = findViewById(R.id.moneyText)
        incomeText = findViewById(R.id.incomeText)
        rankText = findViewById(R.id.rankText)
        nextRankText = findViewById(R.id.nextRankText)
        rankProgress = findViewById(R.id.rankProgress)
        hospitalView = findViewById(R.id.hospitalView)
        newsText = findViewById(R.id.newsText)
        newsText.isSelected = true // включает бегущую строку
        patientsText = findViewById(R.id.patientsText)
        emergencyButton = findViewById(R.id.emergencyButton)
        clickArea = findViewById(R.id.clickArea)
        upgradesContainer = findViewById(R.id.upgradesContainer)
        eventBanner = findViewById(R.id.eventBanner)
        eventEmoji = findViewById(R.id.eventEmoji)
        eventTitle = findViewById(R.id.eventTitle)
        eventDesc = findViewById(R.id.eventDesc)
        eventProgress = findViewById(R.id.eventProgress)
        eventTimer = findViewById(R.id.eventTimer)
        tabs = listOf(
            findViewById(R.id.tabDoctors),
            findViewById(R.id.tabTools),
            findViewById(R.id.tabPrestige),
            findViewById(R.id.tabMaps),
            findViewById(R.id.tabAchievements),
            findViewById(R.id.tabStats),
        )

        load()
        // Достижения, выполненные до обновления, засчитываем молча
        game.checkAchievements()
        game.onMessage = { msg -> Toast.makeText(this, msg, Toast.LENGTH_LONG).show() }
        lastRankIndex = game.rankIndex

        buildShop()
        setupTabs()
        hospitalView.onHospitalClick = { x, y -> onHospitalClicked(x, y) }

        emergencyButton.setOnClickListener { onEmergencyClicked() }

        refreshUi()
    }

    override fun onResume() {
        super.onResume()
        applyOfflineEarnings()
        save()
        lastTick = SystemClock.elapsedRealtime()
        scheduleNextEmergency()
        handler.post(tickRunnable)
        hospitalView.running = true
    }

    override fun onPause() {
        super.onPause()
        handler.removeCallbacks(tickRunnable)
        hospitalView.running = false
        save()
    }

    // ---------- Клик по больнице ----------

    private fun onHospitalClicked(x: Float, y: Float) {
        val earned = game.click()
        hospitalView.addFloatingText("+${Game.format(earned)} ₽", x, y, Color.parseColor("#0E9F9A"))
        refreshUi()
    }

    // ---------- Экстренный пациент ----------

    private fun scheduleNextEmergency() {
        nextEmergencyAt = SystemClock.elapsedRealtime() + Random.nextLong(25_000, 60_000) / (if (game.hasLegacy("ambulance")) 2 else 1)
        emergencyHideAt = 0L
        emergencyButton.visibility = View.GONE
    }

    private fun updateEmergency(now: Long) {
        if (emergencyButton.visibility == View.GONE && now >= nextEmergencyAt) {
            emergencyButton.visibility = View.VISIBLE
            emergencyHideAt = now + 8_000
            emergencyButton.alpha = 0f
            emergencyButton.animate().alpha(1f).setDuration(250).start()
            ObjectAnimator.ofFloat(emergencyButton, View.ROTATION, 0f, -4f, 4f, -4f, 0f).apply {
                duration = 500
                start()
            }
        } else if (emergencyButton.visibility == View.VISIBLE && now >= emergencyHideAt) {
            scheduleNextEmergency()
        }
    }

    private fun onEmergencyClicked() {
        val reward = game.rescueEmergency()
        val cx = emergencyButton.x + emergencyButton.width / 2f
        val cy = emergencyButton.y + emergencyButton.height
        hospitalView.addFloatingText("🚑 +${Game.format(reward)} ₽", cx, cy + 40, Color.parseColor("#E53950"))
        Toast.makeText(this, "Вы спасли экстренного пациента!", Toast.LENGTH_SHORT).show()
        scheduleNextEmergency()
        refreshUi()
    }

    // ---------- Магазин: врачи и инструменты ----------

    private fun setupTabs() {
        tabs.forEachIndexed { i, tab -> tab.setOnClickListener { selectTab(i) } }
        selectTab(0)
    }

    private fun selectTab(index: Int) {
        tabs.forEachIndexed { i, tab ->
            tab.setTextColor(getColor(if (i == index) R.color.primary_dark else R.color.text_muted))
        }
        tabContents.forEachIndexed { i, content ->
            content.visibility = if (i == index) View.VISIBLE else View.GONE
        }
    }

    private fun buildShop() {
        upgradesContainer.removeAllViews()
        updaters.clear()
        val doctorsList = vertical()
        val toolsList = vertical()
        val prestigeList = vertical()
        val mapsList = vertical()
        val achievementsList = vertical()
        val statsList = vertical()
        doctorsList.addView(createBuyAmountSelector())
        game.doctors.forEachIndexed { i, d -> doctorsList.addView(createDoctorCard(i, d)) }
        toolsList.addView(createBuyAmountSelector())
        game.tools.forEach { toolsList.addView(createToolCard(it)) }
        buildPrestigeTab(prestigeList)
        game.maps.forEach { mapsList.addView(createMapCard(it)) }
        buildAchievementsTab(achievementsList)
        buildStatsTab(statsList)
        tabContents = listOf(doctorsList, toolsList, prestigeList, mapsList, achievementsList, statsList)
        tabContents.forEach { upgradesContainer.addView(it) }
    }

    private fun vertical() = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }

    private fun card(vertical: Boolean): LinearLayout = LinearLayout(this).apply {
        orientation = if (vertical) LinearLayout.VERTICAL else LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        setBackgroundResource(R.drawable.bg_card)
        elevation = dp(2).toFloat()
        setPadding(dp(12), dp(10), dp(12), dp(10))
        layoutParams = LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
        ).apply { topMargin = dp(8) }
    }

    private fun text(size: Float, colorRes: Int, bold: Boolean = false) = TextView(this).apply {
        textSize = size
        setTextColor(getColor(colorRes))
        if (bold) setTypeface(typeface, Typeface.BOLD)
    }

    private fun actionButton(): TextView = TextView(this).apply {
        textSize = 13f
        gravity = Gravity.CENTER
        setTextColor(Color.WHITE)
        setTypeface(typeface, Typeface.BOLD)
        setPadding(dp(10), dp(6), dp(10), dp(6))
        minWidth = dp(96)
        layoutParams = LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
        ).apply { marginStart = dp(8) }
    }

    private fun setButtonState(button: TextView, enabled: Boolean) {
        setBg(button, if (enabled) R.drawable.bg_button else R.drawable.bg_button_disabled)
    }

    /** Меняет фон, только если он действительно изменился (UI обновляется 10 раз в секунду). */
    private fun setBg(view: View, res: Int) {
        if (view.tag != res) {
            view.setBackgroundResource(res)
            view.tag = res
        }
    }

    private fun pulse(view: View) {
        view.animate().scaleX(0.95f).scaleY(0.95f).setDuration(70).withEndAction {
            view.animate().scaleX(1f).scaleY(1f).setDuration(90).start()
        }.start()
    }

    private fun afterPurchase(view: View) {
        pulse(view)
        save()
        refreshUi()
    }

    private fun notEnough(cost: Double) {
        Toast.makeText(this, "Не хватает ${Game.format(cost - game.money)} ₽", Toast.LENGTH_SHORT).show()
    }

    private fun createDoctorCard(index: Int, doctor: Doctor): View {
        val root = card(vertical = true)

        // --- Найм ---
        val top = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }
        val emoji = TextView(this).apply {
            text = doctor.emoji
            textSize = 30f
            gravity = Gravity.CENTER
            layoutParams = LinearLayout.LayoutParams(dp(52), dp(52))
        }
        val info = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
                .apply { marginStart = dp(8) }
        }
        val title = text(16f, R.color.text, bold = true).apply { text = doctor.title }
        val status = text(13f, R.color.primary_dark)
        val income = text(12f, R.color.text_muted)
        info.addView(title)
        info.addView(status)
        info.addView(income)
        val hireButton = actionButton()
        top.addView(emoji)
        top.addView(info)
        top.addView(hireButton)
        root.addView(top)

        // --- Квалификация ---
        val qualRow = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, dp(8), 0, 0)
        }
        val qualText = text(13f, R.color.text).apply {
            layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
        }
        val qualButton = actionButton()
        qualRow.addView(qualText)
        qualRow.addView(qualButton)
        root.addView(qualRow)

        // --- Предметы ---
        root.addView(text(13f, R.color.text_muted, bold = true).apply {
            text = "Предметы:"
            setPadding(0, dp(8), 0, dp(4))
        })
        val itemsRow = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL }
        root.addView(itemsRow)

        val itemUpdaters = doctor.items.mapIndexed { i, item ->
            val chip = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                gravity = Gravity.CENTER_HORIZONTAL
                setPadding(dp(4), dp(6), dp(4), dp(6))
                layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)
                    .apply { if (i > 0) marginStart = dp(6) }
            }
            val itemEmoji = TextView(this).apply {
                text = item.emoji
                textSize = 24f
                gravity = Gravity.CENTER
            }
            val itemName = text(11f, R.color.text).apply {
                text = item.title
                gravity = Gravity.CENTER
                maxLines = 2
            }
            val itemEffect = text(12f, R.color.primary_dark, bold = true).apply {
                text = "${Game.formatMultiplier(item.multiplier)} доход"
                gravity = Gravity.CENTER
            }
            val itemState = text(11f, R.color.text_muted, bold = true).apply { gravity = Gravity.CENTER }
            chip.addView(itemEmoji)
            chip.addView(itemName)
            chip.addView(itemEffect)
            chip.addView(itemState)
            itemsRow.addView(chip)

            chip.setOnClickListener {
                when {
                    item.owned -> Toast.makeText(
                        this, "${item.title}: уже куплено", Toast.LENGTH_SHORT,
                    ).show()
                    !doctor.isUnlocked(item) -> Toast.makeText(
                        this,
                        "Наймите ещё ${item.requiredCount - doctor.count} (${doctor.title}), чтобы открыть «${item.title}»",
                        Toast.LENGTH_SHORT,
                    ).show()
                    game.buyItem(doctor, item) -> {
                        Toast.makeText(
                            this, "${item.emoji} ${item.title}: доход «${doctor.title}» ${Game.formatMultiplier(item.multiplier)}!",
                            Toast.LENGTH_SHORT,
                        ).show()
                        afterPurchase(chip)
                    }
                    else -> notEnough(item.cost)
                }
            }

            val updater: () -> Unit = {
                val unlocked = doctor.isUnlocked(item)
                when {
                    item.owned -> {
                        setBg(chip, R.drawable.bg_chip_owned)
                        chip.alpha = 1f
                        itemState.text = "✓ Есть"
                        itemState.setTextColor(Color.parseColor("#2EAD5B"))
                    }
                    !unlocked -> {
                        setBg(chip, R.drawable.bg_chip)
                        chip.alpha = 0.55f
                        itemState.text = "🔒 нужно ${item.requiredCount}"
                        itemState.setTextColor(getColor(R.color.text_muted))
                    }
                    else -> {
                        val affordable = game.money >= item.cost
                        setBg(chip, if (affordable) R.drawable.bg_chip_ready else R.drawable.bg_chip)
                        chip.alpha = 1f
                        itemState.text = "${Game.format(item.cost)} ₽"
                        itemState.setTextColor(getColor(if (affordable) R.color.primary else R.color.text_muted))
                    }
                }
            }
            updater
        }

        hireButton.setOnClickListener {
            if (game.hire(doctor)) afterPurchase(hireButton) else notEnough(game.hireCost(doctor, game.buyAmount))
        }
        qualButton.setOnClickListener {
            when {
                doctor.count == 0 -> Toast.makeText(this, "Сначала наймите врача", Toast.LENGTH_SHORT).show()
                !doctor.canUpgradeQualification -> Toast.makeText(this, "Максимальная квалификация", Toast.LENGTH_SHORT).show()
                game.upgradeQualification(doctor) -> {
                    Toast.makeText(this, "🎓 ${doctor.title}: теперь ${doctor.qualificationTitle}!", Toast.LENGTH_SHORT).show()
                    afterPurchase(qualButton)
                }
                else -> notEnough(doctor.qualificationCost)
            }
        }

        updaters += {
            // Следующий врач появляется, когда нанят предыдущий
            val visible = index == 0 || doctor.count > 0 || game.doctors[index - 1].count > 0
            root.visibility = if (visible) View.VISIBLE else View.GONE
            if (visible) {
                status.text = "Нанято: ${doctor.count}  •  ${doctor.qualificationTitle}"
                income.text = if (doctor.count > 0) {
                    "+${Game.format(game.doctorIncome(doctor))} ₽/сек (${Game.format(game.doctorIncomePerOne(doctor))} за каждого, ${Game.formatMultiplier(doctor.multiplier)})"
                } else {
                    "+${Game.format(game.doctorIncomePerOne(doctor))} ₽/сек за каждого"
                }
                val hireCost = game.hireCost(doctor, game.buyAmount)
                hireButton.text = "Нанять ×${game.buyAmount}\n${Game.format(hireCost)} ₽"
                setButtonState(hireButton, game.money >= hireCost)

                val next = doctor.nextQualificationTitle
                if (next == null) {
                    qualText.text = "🎓 ${doctor.qualificationTitle} — высшая квалификация"
                    qualButton.text = "Макс."
                    setButtonState(qualButton, false)
                } else {
                    qualText.text = "🎓 ${doctor.qualificationTitle} → $next (×2 доход)"
                    qualButton.text = "Повысить\n${Game.format(doctor.qualificationCost)} ₽"
                    setButtonState(qualButton, doctor.count > 0 && game.money >= doctor.qualificationCost)
                }
                itemUpdaters.forEach { it() }
            }
        }
        return root
    }

    private fun createToolCard(tool: Tool): View {
        val root = card(vertical = false)
        val emoji = TextView(this).apply {
            text = tool.emoji
            textSize = 30f
            gravity = Gravity.CENTER
            layoutParams = LinearLayout.LayoutParams(dp(52), dp(52))
        }
        val middle = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
                .apply { marginStart = dp(8) }
        }
        middle.addView(text(16f, R.color.text, bold = true).apply { text = tool.title })
        middle.addView(text(13f, R.color.text_muted).apply { text = tool.description })
        val level = text(12f, R.color.primary_dark)
        middle.addView(level)
        val button = actionButton()

        root.addView(emoji)
        root.addView(middle)
        root.addView(button)

        button.setOnClickListener {
            if (game.buyTool(tool)) afterPurchase(button) else notEnough(tool.costFor(game.buyAmount))
        }
        updaters += {
            level.text = "Уровень ${tool.level}"
            val cost = tool.costFor(game.buyAmount)
            button.text = "Купить ×${game.buyAmount}\n${Game.format(cost)} ₽"
            setButtonState(button, game.money >= cost)
        }
        return root
    }

    // ---------- Престиж ----------

    private fun buildPrestigeTab(list: LinearLayout) {
        // Карточка аккредитации
        val main = card(vertical = true)
        main.addView(text(17f, R.color.text, bold = true).apply { text = "⭐ Аккредитация больницы" })
        main.addView(text(13f, R.color.text_muted).apply {
            text = "Пройдите аккредитацию, чтобы получить звёзды репутации. Больница, врачи и " +
                "инструменты сбросятся, но звёзды навсегда увеличат весь доход и клик, а за " +
                "свободные звёзды можно купить улучшения наследия и открыть новые карты."
            setPadding(0, dp(4), 0, dp(8))
        })
        val starsInfo = text(14f, R.color.text, bold = true)
        val pendingInfo = text(13f, R.color.primary_dark)
        val statsInfo = text(12f, R.color.text_muted).apply { setPadding(0, dp(4), 0, 0) }
        val prestigeButton = actionButton().apply {
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT,
            ).apply { topMargin = dp(10) }
            setPadding(dp(10), dp(12), dp(10), dp(12))
            textSize = 15f
        }
        main.addView(starsInfo)
        main.addView(pendingInfo)
        main.addView(statsInfo)
        main.addView(prestigeButton)
        list.addView(main)

        prestigeButton.setOnClickListener {
            if (!game.canPrestige()) {
                Toast.makeText(
                    this,
                    "Заработайте за забег хотя бы ${Game.format(Game.PRESTIGE_UNIT)} ₽",
                    Toast.LENGTH_SHORT,
                ).show()
                return@setOnClickListener
            }
            val gain = game.pendingStars
            AlertDialog.Builder(this)
                .setTitle("Пройти аккредитацию?")
                .setMessage(
                    "Вы получите ⭐ $gain.\n\nДеньги, врачи, предметы и инструменты будут сброшены. " +
                        "Звёзды, наследие и открытые карты сохранятся.",
                )
                .setPositiveButton("Пройти") { _, _ ->
                    val gained = game.prestige()
                    lastRankIndex = 0
                    save()
                    refreshUi()
                    AlertDialog.Builder(this)
                        .setTitle("🎉 Аккредитация пройдена!")
                        .setMessage(
                            "Получено ⭐ $gained. Теперь у вас ⭐ ${game.stars}: " +
                                "доход и клик ${Game.formatMultiplier(game.starBonus)}.",
                        )
                        .setPositiveButton("Ура", null)
                        .show()
                }
                .setNegativeButton("Отмена", null)
                .show()
        }

        updaters += {
            starsInfo.text = "Звёзды репутации: ⭐ ${game.stars} (свободно: ${game.availableStars})\n" +
                "Бонус ко всему доходу: ${Game.formatMultiplier(game.starBonus)}"
            val pending = game.pendingStars
            pendingInfo.text = "За этот забег: +⭐ $pending  •  следующая звезда при " +
                "${Game.format(game.nextStarAt)} ₽ заработано (сейчас ${Game.format(game.totalEarned)} ₽)"
            statsInfo.text = "Аккредитаций: ${game.prestigeCount}  •  Заработано за всё время: " +
                "${Game.format(game.lifetimeEarned)} ₽  •  Вылечено: ${game.patientsHealed}"
            prestigeButton.text = if (pending > 0) "Пройти аккредитацию (+⭐ $pending)" else "Аккредитация пока недоступна"
            setButtonState(prestigeButton, pending > 0)
        }

        list.addView(text(16f, R.color.text, bold = true).apply {
            text = "Наследие (за свободные звёзды)"
            setPadding(dp(4), dp(14), 0, 0)
        })
        game.legacies.forEach { list.addView(createLegacyCard(it)) }

        // Полный сброс
        list.addView(TextView(this).apply {
            text = "Полный сброс игры"
            textSize = 13f
            setTextColor(getColor(R.color.text_muted))
            gravity = Gravity.CENTER
            setPadding(0, dp(18), 0, dp(8))
            setOnClickListener { confirmReset() }
        })
    }

    private fun createLegacyCard(legacy: Legacy): View {
        val root = card(vertical = false)
        root.addView(TextView(this).apply {
            text = legacy.emoji
            textSize = 28f
            gravity = Gravity.CENTER
            layoutParams = LinearLayout.LayoutParams(dp(48), dp(48))
        })
        val middle = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
                .apply { marginStart = dp(8) }
        }
        middle.addView(text(15f, R.color.text, bold = true).apply { text = legacy.title })
        middle.addView(text(13f, R.color.text_muted).apply { text = legacy.description })
        root.addView(middle)
        val button = actionButton()
        root.addView(button)

        button.setOnClickListener {
            when {
                legacy.owned -> Unit
                game.buyLegacy(legacy) -> {
                    Toast.makeText(this, "${legacy.emoji} ${legacy.title} — навсегда ваше!", Toast.LENGTH_SHORT).show()
                    afterPurchase(button)
                }
                else -> Toast.makeText(
                    this, "Нужно ⭐ ${legacy.cost}, свободно ⭐ ${game.availableStars}", Toast.LENGTH_SHORT,
                ).show()
            }
        }
        updaters += {
            if (legacy.owned) {
                button.text = "✓ Есть"
                setBg(button, R.drawable.bg_chip_owned)
                button.setTextColor(Color.parseColor("#2EAD5B"))
            } else {
                button.text = "Купить\n⭐ ${legacy.cost}"
                button.setTextColor(Color.WHITE)
                setButtonState(button, game.availableStars >= legacy.cost)
            }
        }
        return root
    }

    // ---------- Карты ----------

    private fun createMapCard(m: HospitalMap): View {
        val root = card(vertical = false)
        root.addView(TextView(this).apply {
            text = m.emoji
            textSize = 30f
            gravity = Gravity.CENTER
            layoutParams = LinearLayout.LayoutParams(dp(52), dp(52))
        })
        val middle = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
                .apply { marginStart = dp(8) }
        }
        middle.addView(text(16f, R.color.text, bold = true).apply { text = m.title })
        middle.addView(text(13f, R.color.text_muted).apply { text = m.description })
        val diseases = game.events
            .filter { it.kind == EventKind.OUTBREAK && (it.maps == null || m.id in it.maps) }
            .joinToString(" ") { it.emoji }
        middle.addView(text(12f, R.color.text_muted).apply { text = "Болезни: $diseases" })
        root.addView(middle)
        val button = actionButton()
        root.addView(button)

        button.setOnClickListener {
            when {
                game.mapId == m.id -> Unit
                !game.isMapUnlocked(m) -> Toast.makeText(
                    this,
                    "Откроется, когда у вас будет ⭐ ${m.starsRequired} (сейчас ⭐ ${game.stars})",
                    Toast.LENGTH_SHORT,
                ).show()
                game.selectMap(m) -> {
                    Toast.makeText(this, "${m.emoji} Больница переехала: ${m.title}", Toast.LENGTH_SHORT).show()
                    afterPurchase(root)
                }
            }
        }
        updaters += {
            val unlocked = game.isMapUnlocked(m)
            root.alpha = if (unlocked) 1f else 0.6f
            when {
                game.mapId == m.id -> {
                    button.text = "✓ Здесь"
                    setBg(button, R.drawable.bg_chip_owned)
                    button.setTextColor(Color.parseColor("#2EAD5B"))
                }
                unlocked -> {
                    button.text = "Переехать"
                    button.setTextColor(Color.WHITE)
                    setButtonState(button, true)
                }
                else -> {
                    button.text = "🔒 ⭐ ${m.starsRequired}"
                    button.setTextColor(Color.WHITE)
                    setButtonState(button, false)
                }
            }
        }
        return root
    }

    // ---------- Покупка ×1 / ×10 / ×100 ----------

    private fun createBuyAmountSelector(): View {
        val row = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(dp(4), dp(8), 0, 0)
        }
        row.addView(text(13f, R.color.text_muted, bold = true).apply {
            text = "Покупать по:"
            layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
        })
        val chips = BUY_AMOUNTS.map { amount ->
            TextView(this).apply {
                text = "×$amount"
                textSize = 14f
                gravity = Gravity.CENTER
                setTypeface(typeface, Typeface.BOLD)
                setPadding(dp(14), dp(6), dp(14), dp(6))
                layoutParams = LinearLayout.LayoutParams(
                    ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                ).apply { marginStart = dp(6) }
                setOnClickListener {
                    game.buyAmount = amount
                    refreshUi()
                }
                row.addView(this)
            }
        }
        updaters += {
            chips.forEachIndexed { i, chip ->
                val selected = BUY_AMOUNTS[i] == game.buyAmount
                setBg(chip, if (selected) R.drawable.bg_button else R.drawable.bg_chip)
                chip.setTextColor(if (selected) Color.WHITE else getColor(R.color.text))
            }
        }
        return row
    }

    // ---------- Достижения ----------

    private fun buildAchievementsTab(list: LinearLayout) {
        val header = card(vertical = true)
        val summary = text(15f, R.color.text, bold = true)
        header.addView(summary)
        header.addView(text(12f, R.color.text_muted).apply {
            text = "Каждое достижение навсегда даёт +1% к доходу и клику. Нажмите на значок, чтобы узнать условие."
        })
        list.addView(header)

        val grid = card(vertical = true)
        list.addView(grid)
        val perRow = 5
        game.achievements.chunked(perRow).forEach { chunk ->
            val row = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL }
            chunk.forEach { ach ->
                val tile = TextView(this).apply {
                    text = ach.emoji
                    textSize = 26f
                    gravity = Gravity.CENTER
                    layoutParams = LinearLayout.LayoutParams(0, dp(56), 1f).apply {
                        setMargins(dp(3), dp(3), dp(3), dp(3))
                    }
                    setOnClickListener {
                        val got = ach.id in game.unlockedAchievements
                        Toast.makeText(
                            this@MainActivity,
                            "${if (got) "🏆" else "🔒"} ${ach.title}\n${ach.description}",
                            Toast.LENGTH_SHORT,
                        ).show()
                    }
                }
                row.addView(tile)
                updaters += {
                    val got = ach.id in game.unlockedAchievements
                    setBg(tile, if (got) R.drawable.bg_chip_ready else R.drawable.bg_chip)
                    tile.alpha = if (got) 1f else 0.3f
                }
            }
            // Пустые ячейки, чтобы последняя строка не растягивалась
            repeat(perRow - chunk.size) {
                row.addView(View(this).apply {
                    layoutParams = LinearLayout.LayoutParams(0, dp(56), 1f).apply {
                        setMargins(dp(3), dp(3), dp(3), dp(3))
                    }
                })
            }
            grid.addView(row)
        }

        updaters += {
            val n = game.unlockedAchievements.size
            summary.text = "🏆 Получено $n из ${game.achievements.size}  •  бонус +$n%"
        }
    }

    // ---------- Статистика ----------

    private fun buildStatsTab(list: LinearLayout) {
        val c = card(vertical = true)
        c.addView(text(17f, R.color.text, bold = true).apply {
            text = "📊 Статистика"
            setPadding(0, 0, 0, dp(6))
        })
        list.addView(c)

        fun row(label: String, value: () -> String) {
            val r = LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                setPadding(0, dp(4), 0, dp(4))
            }
            r.addView(text(14f, R.color.text_muted).apply {
                text = label
                layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
            })
            val v = text(14f, R.color.text, bold = true).apply { gravity = Gravity.END }
            r.addView(v)
            c.addView(r)
            updaters += { v.text = value() }
        }

        row("Деньги") { "${Game.format(game.money)} ₽" }
        row("Заработано за забег") { "${Game.format(game.totalEarned)} ₽" }
        row("Заработано за всё время") { "${Game.format(game.lifetimeEarned)} ₽" }
        row("Доход сейчас") { "${Game.format(game.incomePerSecond)} ₽/сек" }
        row("Рекорд дохода") { "${Game.format(game.bestIncome)} ₽/сек" }
        row("Сила клика") { "${Game.format(game.clickPower)} ₽" }
        row("Кликов сделано") { Game.format(game.totalClicks.toDouble()) }
        row("Заработано кликами") { "${Game.format(game.clickEarned)} ₽" }
        row("Вылечено пациентов") { Game.format(game.patientsHealed.toDouble()) }
        row("Врачей в штате") { Game.format(game.doctors.sumOf { it.count }.toDouble()) }
        row("Предметов куплено") { "${game.doctors.sumOf { d -> d.items.count { it.owned } }} из ${game.doctors.size * 3}" }
        row("Экстренных пациентов спасено") { game.emergencies.toString() }
        row("Вспышек остановлено") { game.outbreaksCured.toString() }
        row("Вспышек пропущено") { game.outbreaksFailed.toString() }
        row("Аккредитаций") { game.prestigeCount.toString() }
        row("Звёзды репутации") { "⭐ ${game.stars}" }
        row("Достижения") { "${game.unlockedAchievements.size} из ${game.achievements.size}" }
        row("Карта") { "${game.map.emoji} ${game.map.title}" }
        row("Время в игре") { formatDuration(game.playTimeSec) }
        row("Больница открыта") {
            java.text.SimpleDateFormat("dd.MM.yyyy", java.util.Locale.getDefault())
                .format(java.util.Date(game.startedAt))
        }
    }

    private fun formatDuration(seconds: Double): String {
        val s = seconds.toLong()
        val h = s / 3600
        val m = (s % 3600) / 60
        return if (h > 0) "$h ч $m мин" else "$m мин ${s % 60} с"
    }

    // ---------- Отрисовка ----------

    private fun refreshUi() {
        moneyText.text = "${Game.format(game.money)} ₽"
        incomeText.text = "${Game.format(game.incomePerSecond)} ₽/сек  •  ${Game.format(game.clickPower)} ₽ за клик"
        rankText.text = "${game.map.emoji} ${game.rank.title}" +
            (if (game.stars > 0) "  •  ⭐ ${game.stars} (+${Game.format((game.starBonus - 1) * 100)}%)" else "")
        rankProgress.progress = (game.rankProgress * rankProgress.max).toInt()
        nextRankText.text = game.nextRank?.let {
            "Следующий статус: ${it.title} (${Game.format(it.threshold)} ₽ заработано)"
        } ?: "Максимальный статус достигнут!"

        if (game.rankIndex > lastRankIndex) {
            lastRankIndex = game.rankIndex
            Toast.makeText(this, "🎉 Новый статус: ${game.rank.title}!", Toast.LENGTH_LONG).show()
        }

        updateEventBanner()
        updateHospitalView()
        val bg = Color.parseColor(game.map.background)
        if (clickArea.tag != bg) {
            clickArea.setBackgroundColor(bg)
            clickArea.tag = bg
        }
        updaters.forEach { it() }
    }

    private fun updateHospitalView() {
        val key = game.doctors.joinToString(",") { it.count.toString() }
        if (key != orbitersKey) {
            orbitersKey = key
            // По кругу берём по одному врачу каждой специальности, пока не наберём лимит
            val left = game.doctors.map { it.count }.toMutableList()
            val result = mutableListOf<String>()
            while (result.size < HospitalView.MAX_ORBITERS && left.any { it > 0 }) {
                game.doctors.forEachIndexed { i, d ->
                    if (left[i] > 0 && result.size < HospitalView.MAX_ORBITERS) {
                        result += d.emoji
                        left[i]--
                    }
                }
            }
            hospitalView.orbiters = result
        }
        hospitalView.badgeEmoji = game.event?.type?.emoji
        hospitalView.rainPerSecond = if (game.incomePerSecond <= 0) 0.0
        else (kotlin.math.log10(1 + game.incomePerSecond) * 0.7).coerceAtMost(8.0)
        patientsText.text = "Вылечено: ${Game.format(game.patientsHealed.toDouble())}"
    }

    private fun updateEventBanner() {
        val e = game.event
        if (e == null) {
            eventBanner.visibility = View.GONE
            return
        }
        eventBanner.visibility = View.VISIBLE
        val outbreak = e.type.kind == EventKind.OUTBREAK
        setBg(eventBanner, if (outbreak) R.drawable.bg_event_bad else R.drawable.bg_event_good)
        eventEmoji.text = e.type.emoji
        eventTitle.text = e.type.title
        eventTimer.text = "${e.remaining.toInt() + 1} с"
        if (outbreak) {
            eventDesc.text = "Вылечено ${e.progress} из ${e.type.cureClicks} — жмите на пациента!"
            eventProgress.progress = (e.progress * 1000 / e.type.cureClicks)
            eventProgress.progressTintList = android.content.res.ColorStateList.valueOf(Color.parseColor("#E53950"))
        } else {
            eventDesc.text = e.type.description
            eventProgress.progress = (e.remaining / e.type.durationSec * 1000).toInt().coerceIn(0, 1000)
            eventProgress.progressTintList = android.content.res.ColorStateList.valueOf(Color.parseColor("#2EAD5B"))
        }
    }

    // ---------- Сохранение ----------

    private fun save() {
        val prefs = getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
        prefs.putString("money", game.money.toString())
        prefs.putString("total", game.totalEarned.toString())
        prefs.putLong("patients", game.patientsHealed)
        prefs.putString("lifetime", game.lifetimeEarned.toString())
        prefs.putLong("stars", game.stars)
        prefs.putLong("starsSpent", game.starsSpent)
        prefs.putInt("prestiges", game.prestigeCount)
        prefs.putString("map", game.mapId)
        game.legacies.forEach { prefs.putBoolean("legacy_${it.id}", it.owned) }
        prefs.putLong("clicks", game.totalClicks)
        prefs.putString("clickEarned", game.clickEarned.toString())
        prefs.putInt("outbreaksCured", game.outbreaksCured)
        prefs.putInt("outbreaksFailed", game.outbreaksFailed)
        prefs.putInt("emergencies", game.emergencies)
        prefs.putString("playTime", game.playTimeSec.toString())
        prefs.putString("bestIncome", game.bestIncome.toString())
        prefs.putLong("startedAt", game.startedAt)
        prefs.putInt("buyAmount", game.buyAmount)
        prefs.putStringSet("curedEvents", game.curedEventIds.toSet())
        prefs.putStringSet("visitedMaps", game.visitedMaps.toSet())
        prefs.putStringSet("achievements", game.unlockedAchievements.toSet())
        prefs.putLong("savedAt", System.currentTimeMillis())
        game.tools.forEach { prefs.putInt("lvl_${it.id}", it.level) }
        game.doctors.forEach { d ->
            prefs.putInt("lvl_${d.id}", d.count)
            prefs.putInt("qual_${d.id}", d.qualification)
            d.items.forEach { prefs.putBoolean("item_${it.id}", it.owned) }
        }
        prefs.apply()
    }

    private fun load() {
        val prefs = getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        game.money = prefs.getString("money", null)?.toDoubleOrNull() ?: 0.0
        game.totalEarned = prefs.getString("total", null)?.toDoubleOrNull() ?: 0.0
        game.patientsHealed = prefs.getLong("patients", 0)
        game.lifetimeEarned = prefs.getString("lifetime", null)?.toDoubleOrNull() ?: game.totalEarned
        game.stars = prefs.getLong("stars", 0)
        game.starsSpent = prefs.getLong("starsSpent", 0)
        game.prestigeCount = prefs.getInt("prestiges", 0)
        game.legacies.forEach { it.owned = prefs.getBoolean("legacy_${it.id}", false) }
        game.totalClicks = prefs.getLong("clicks", game.patientsHealed)
        game.clickEarned = prefs.getString("clickEarned", null)?.toDoubleOrNull() ?: 0.0
        game.outbreaksCured = prefs.getInt("outbreaksCured", 0)
        game.outbreaksFailed = prefs.getInt("outbreaksFailed", 0)
        game.emergencies = prefs.getInt("emergencies", 0)
        game.playTimeSec = prefs.getString("playTime", null)?.toDoubleOrNull() ?: 0.0
        game.bestIncome = prefs.getString("bestIncome", null)?.toDoubleOrNull() ?: 0.0
        game.startedAt = prefs.getLong("startedAt", System.currentTimeMillis())
        game.buyAmount = prefs.getInt("buyAmount", 1).takeIf { it in BUY_AMOUNTS } ?: 1
        game.curedEventIds += prefs.getStringSet("curedEvents", emptySet()).orEmpty()
        game.visitedMaps += prefs.getStringSet("visitedMaps", emptySet()).orEmpty()
        game.unlockedAchievements += prefs.getStringSet("achievements", emptySet()).orEmpty()
        val savedMap = prefs.getString("map", "village")
        game.mapId = game.maps.firstOrNull { it.id == savedMap && game.isMapUnlocked(it) }?.id ?: "village"
        game.tools.forEach { it.level = prefs.getInt("lvl_${it.id}", 0) }
        game.doctors.forEach { d ->
            d.count = prefs.getInt("lvl_${d.id}", 0)
            d.qualification = prefs.getInt("qual_${d.id}", 0)
                .coerceIn(0, Doctor.QUALIFICATIONS.lastIndex)
            d.items.forEach { it.owned = prefs.getBoolean("item_${it.id}", false) }
        }
    }

    /** Доход, пока игра была свёрнута или закрыта (максимум за 3 часа). */
    private fun applyOfflineEarnings() {
        val savedAt = getSharedPreferences(PREFS, Context.MODE_PRIVATE).getLong("savedAt", 0)
        if (savedAt <= 0 || game.baseIncomePerSecond <= 0) return
        val seconds = ((System.currentTimeMillis() - savedAt) / 1000.0)
            .coerceIn(0.0, game.offlineLimitSec)
        if (seconds < 10) return
        val earned = game.baseIncomePerSecond * seconds
        game.earn(earned)
        AlertDialog.Builder(this)
            .setTitle("С возвращением, главврач!")
            .setMessage("Пока вас не было, персонал заработал ${Game.format(earned)} ₽")
            .setPositiveButton("Отлично", null)
            .show()
    }

    private fun confirmReset() {
        AlertDialog.Builder(this)
            .setTitle("Начать заново?")
            .setMessage("Весь прогресс будет удалён, включая звёзды репутации, наследие и карты.")
            .setPositiveButton("Сбросить") { _, _ ->
                game.reset()
                lastRankIndex = 0
                save()
                refreshUi()
            }
            .setNegativeButton("Отмена", null)
            .show()
    }

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).toInt()

    companion object {
        private const val PREFS = "hospital_clicker"
        private const val TICK_MS = 100L
        private const val NEWS_TICKS = 120 // новая новость каждые ~12 секунд
        private val BUY_AMOUNTS = listOf(1, 10, 100)
    }
}
