package com.practika.hospitalclicker

import android.animation.Animator
import android.animation.AnimatorListenerAdapter
import android.animation.AnimatorSet
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
import android.view.MotionEvent
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
    private lateinit var patientButton: TextView
    private lateinit var patientsText: TextView
    private lateinit var emergencyButton: TextView
    private lateinit var clickArea: FrameLayout
    private lateinit var upgradesContainer: LinearLayout

    private class UpgradeRow(
        val root: View,
        val levelText: TextView,
        val costText: TextView,
    )

    private val rows = mutableMapOf<String, UpgradeRow>()

    private var lastTick = 0L
    private var lastRankIndex = 0
    private var nextEmergencyAt = 0L
    private var emergencyHideAt = 0L
    private var ticksSinceSave = 0

    private val sickFaces = listOf("🤒", "🤕", "🤧", "🤢", "😷", "🥴")
    private val healedFaces = listOf("😊", "😁", "🥳", "😎", "🙂")

    private val tickRunnable = object : Runnable {
        override fun run() {
            val now = SystemClock.elapsedRealtime()
            game.tick((now - lastTick) / 1000.0)
            lastTick = now
            updateEmergency(now)
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
        patientButton = findViewById(R.id.patientButton)
        patientsText = findViewById(R.id.patientsText)
        emergencyButton = findViewById(R.id.emergencyButton)
        clickArea = findViewById(R.id.clickArea)
        upgradesContainer = findViewById(R.id.upgradesContainer)

        load()
        lastRankIndex = game.rankIndex

        buildUpgradeRows()
        setupPatientButton()

        emergencyButton.setOnClickListener { onEmergencyClicked() }
        findViewById<View>(R.id.resetButton).setOnClickListener { confirmReset() }

        refreshUi()
    }

    override fun onResume() {
        super.onResume()
        applyOfflineEarnings()
        save()
        lastTick = SystemClock.elapsedRealtime()
        scheduleNextEmergency()
        handler.post(tickRunnable)
    }

    override fun onPause() {
        super.onPause()
        handler.removeCallbacks(tickRunnable)
        save()
    }

    // ---------- Клик по пациенту ----------

    private fun setupPatientButton() {
        patientButton.setOnTouchListener { v, event ->
            when (event.actionMasked) {
                MotionEvent.ACTION_DOWN -> {
                    v.animate().scaleX(0.9f).scaleY(0.9f).setDuration(60).start()
                    onPatientClicked(event.x + v.left, event.y + v.top)
                    true
                }
                MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                    v.animate().scaleX(1f).scaleY(1f).setDuration(90).start()
                    if (event.actionMasked == MotionEvent.ACTION_UP) v.performClick()
                    true
                }
                else -> true
            }
        }
    }

    private fun onPatientClicked(x: Float, y: Float) {
        val earned = game.click()
        showFloatingText("+${Game.format(earned)} ₽", x, y, Color.parseColor("#0E9F9A"))

        // Пациент на мгновение выздоравливает, потом приходит следующий
        patientButton.text = healedFaces.random()
        handler.removeCallbacks(nextPatientRunnable)
        handler.postDelayed(nextPatientRunnable, 250)
        refreshUi()
    }

    private val nextPatientRunnable = Runnable { patientButton.text = sickFaces.random() }

    private fun showFloatingText(text: String, x: Float, y: Float, color: Int) {
        val tv = TextView(this).apply {
            this.text = text
            setTextColor(color)
            textSize = 20f
            setTypeface(typeface, Typeface.BOLD)
            elevation = 20f
        }
        val lp = FrameLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
        )
        clickArea.addView(tv, lp)
        tv.x = x - 40 + Random.nextInt(-30, 30)
        tv.y = y - 60

        val up = ObjectAnimator.ofFloat(tv, View.TRANSLATION_Y, tv.y, tv.y - 220f)
        val fade = ObjectAnimator.ofFloat(tv, View.ALPHA, 1f, 0f)
        AnimatorSet().apply {
            playTogether(up, fade)
            duration = 800
            addListener(object : AnimatorListenerAdapter() {
                override fun onAnimationEnd(animation: Animator) {
                    clickArea.removeView(tv)
                }
            })
            start()
        }
    }

    // ---------- Экстренный пациент ----------

    private fun scheduleNextEmergency() {
        nextEmergencyAt = SystemClock.elapsedRealtime() + Random.nextLong(25_000, 60_000)
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
        val reward = game.emergencyReward()
        game.earn(reward)
        game.patientsHealed++
        val cx = emergencyButton.x + emergencyButton.width / 2f
        val cy = emergencyButton.y + emergencyButton.height
        showFloatingText("🚑 +${Game.format(reward)} ₽", cx - 60, cy + 60, Color.parseColor("#E53950"))
        Toast.makeText(this, "Вы спасли экстренного пациента!", Toast.LENGTH_SHORT).show()
        scheduleNextEmergency()
        refreshUi()
    }

    // ---------- Улучшения ----------

    private fun buildUpgradeRows() {
        upgradesContainer.removeAllViews()
        rows.clear()
        for (upgrade in game.upgrades) {
            val row = createUpgradeRow(upgrade)
            upgradesContainer.addView(row.root)
            rows[upgrade.id] = row
        }
    }

    private fun createUpgradeRow(upgrade: Upgrade): UpgradeRow {
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setBackgroundResource(R.drawable.bg_card)
            elevation = dp(2).toFloat()
            setPadding(dp(12), dp(10), dp(12), dp(10))
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT,
            ).apply { topMargin = dp(8) }
            isClickable = true
        }

        val emoji = TextView(this).apply {
            text = upgrade.emoji
            textSize = 30f
            gravity = Gravity.CENTER
            layoutParams = LinearLayout.LayoutParams(dp(52), dp(52))
        }

        val middle = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
                .apply { marginStart = dp(8) }
        }
        val title = TextView(this).apply {
            text = upgrade.title
            textSize = 16f
            setTextColor(getColor(R.color.text))
            setTypeface(typeface, Typeface.BOLD)
        }
        val desc = TextView(this).apply {
            text = upgrade.description
            textSize = 13f
            setTextColor(getColor(R.color.text_muted))
        }
        val level = TextView(this).apply {
            textSize = 12f
            setTextColor(getColor(R.color.primary_dark))
        }
        middle.addView(title)
        middle.addView(desc)
        middle.addView(level)

        val cost = TextView(this).apply {
            textSize = 15f
            setTypeface(typeface, Typeface.BOLD)
            gravity = Gravity.END
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT,
                ViewGroup.LayoutParams.WRAP_CONTENT,
            ).apply { marginStart = dp(8) }
        }

        root.addView(emoji)
        root.addView(middle)
        root.addView(cost)

        root.setOnClickListener {
            if (game.buy(upgrade)) {
                root.animate().scaleX(0.97f).scaleY(0.97f).setDuration(70).withEndAction {
                    root.animate().scaleX(1f).scaleY(1f).setDuration(90).start()
                }.start()
                save()
                refreshUi()
            } else {
                val need = upgrade.cost - game.money
                Toast.makeText(this, "Не хватает ${Game.format(need)} ₽", Toast.LENGTH_SHORT).show()
            }
        }

        return UpgradeRow(root, level, cost)
    }

    // ---------- Отрисовка ----------

    private fun refreshUi() {
        moneyText.text = "${Game.format(game.money)} ₽"
        incomeText.text = "${Game.format(game.incomePerSecond)} ₽/сек  •  ${Game.format(game.clickPower)} ₽ за клик"
        rankText.text = "🏥 ${game.rank.title}"
        rankProgress.progress = (game.rankProgress * rankProgress.max).toInt()
        nextRankText.text = game.nextRank?.let {
            "Следующий статус: ${it.title} (${Game.format(it.threshold)} ₽ заработано)"
        } ?: "Максимальный статус достигнут!"
        patientsText.text = "Вылечено пациентов: ${game.patientsHealed}"

        if (game.rankIndex > lastRankIndex) {
            lastRankIndex = game.rankIndex
            Toast.makeText(this, "🎉 Новый статус: ${game.rank.title}!", Toast.LENGTH_LONG).show()
        }

        for (upgrade in game.upgrades) {
            val row = rows[upgrade.id] ?: continue
            val affordable = game.money >= upgrade.cost
            row.levelText.text = "Уровень ${upgrade.level}"
            row.costText.text = "${Game.format(upgrade.cost)} ₽"
            row.costText.setTextColor(getColor(if (affordable) R.color.primary else R.color.disabled))
            row.root.alpha = if (affordable) 1f else 0.7f
        }
    }

    // ---------- Сохранение ----------

    private fun save() {
        val prefs = getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
        prefs.putString("money", game.money.toString())
        prefs.putString("total", game.totalEarned.toString())
        prefs.putLong("patients", game.patientsHealed)
        prefs.putLong("savedAt", System.currentTimeMillis())
        game.upgrades.forEach { prefs.putInt("lvl_${it.id}", it.level) }
        prefs.apply()
    }

    private fun load() {
        val prefs = getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        game.money = prefs.getString("money", null)?.toDoubleOrNull() ?: 0.0
        game.totalEarned = prefs.getString("total", null)?.toDoubleOrNull() ?: 0.0
        game.patientsHealed = prefs.getLong("patients", 0)
        game.upgrades.forEach { it.level = prefs.getInt("lvl_${it.id}", 0) }
    }

    /** Доход, пока игра была свёрнута или закрыта (максимум за 3 часа). */
    private fun applyOfflineEarnings() {
        val savedAt = getSharedPreferences(PREFS, Context.MODE_PRIVATE).getLong("savedAt", 0)
        if (savedAt <= 0 || game.incomePerSecond <= 0) return
        val seconds = ((System.currentTimeMillis() - savedAt) / 1000.0)
            .coerceIn(0.0, OFFLINE_LIMIT_SEC)
        if (seconds < 10) return
        val earned = game.incomePerSecond * seconds
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
            .setMessage("Весь прогресс больницы будет удалён.")
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
        private const val OFFLINE_LIMIT_SEC = 3 * 60 * 60.0
    }
}
