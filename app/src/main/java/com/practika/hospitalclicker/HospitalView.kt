package com.practika.hospitalclicker

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.RadialGradient
import android.graphics.Shader
import android.graphics.Typeface
import android.util.AttributeSet
import android.view.MotionEvent
import android.view.View
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.min
import kotlin.math.sin
import kotlin.random.Random

/**
 * Главный экран: большая больница, по которой кликают, вокруг неё кружат
 * нанятые врачи, при клике разлетаются частицы, сверху падают таблетки.
 */
class HospitalView(context: Context, attrs: AttributeSet? = null) : View(context, attrs) {

    /** Вызывается при нажатии на больницу (координаты внутри вида). */
    var onHospitalClick: ((Float, Float) -> Unit)? = null

    /** Эмодзи врачей, кружащих вокруг больницы. */
    var orbiters: List<String> = emptyList()

    /** Значок активной болезни/события рядом с больницей. */
    var badgeEmoji: String? = null

    /** Сколько «таблеток» в секунду падает сверху (зависит от дохода). */
    var rainPerSecond = 0.0

    var running = false
        set(value) {
            field = value
            lastFrame = 0L
            if (value) postInvalidateOnAnimation()
        }

    private class Particle(
        val text: String,
        var x: Float,
        var y: Float,
        var vx: Float,
        var vy: Float,
        val gravity: Float,
        var life: Float,
        val maxLife: Float,
        val size: Float,
        val color: Int?,
    )

    private val particles = ArrayList<Particle>()
    private val density = resources.displayMetrics.density

    private val emojiPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { textAlign = Paint.Align.CENTER }
    private val numberPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textAlign = Paint.Align.CENTER
        typeface = Typeface.DEFAULT_BOLD
        setShadowLayer(4f, 0f, 2f, Color.argb(90, 0, 0, 0))
    }
    private val glowPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private var glowFor = 0f

    private var lastFrame = 0L
    private var time = 0f
    private var scale = 1f
    private var targetScale = 1f
    private var rainAcc = 0.0

    private val burstEmojis = listOf("😊", "💊", "❤️", "🩹", "✨", "💉")
    private val rainEmojis = listOf("💊", "💊", "💉", "🩹", "❤️", "💵")

    private val cx get() = width / 2f
    private val cy get() = height / 2f
    private val outerRadius get() = min(width, height) / 2f - 22 * density
    private val hospitalRadius get() = outerRadius * 0.55f

    fun addFloatingText(text: String, x: Float, y: Float, color: Int) {
        particles += Particle(
            text, x + Random.nextInt(-20, 20) * density, y - 20 * density,
            0f, -70 * density, 0f, 1.0f, 1.0f, 20 * density, color,
        )
        trim()
    }

    private fun burst(x: Float, y: Float) {
        repeat(5) {
            val angle = Random.nextDouble(0.0, 2 * PI)
            val speed = Random.nextDouble(80.0, 220.0).toFloat() * density
            particles += Particle(
                burstEmojis.random(), x, y,
                (cos(angle) * speed).toFloat(), (sin(angle) * speed).toFloat() - 120 * density,
                420 * density, 0.9f, 0.9f, Random.nextInt(16, 26) * density, null,
            )
        }
        trim()
    }

    private fun trim() {
        while (particles.size > 160) particles.removeAt(0)
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> {
                val i = event.actionIndex
                val x = event.getX(i)
                val y = event.getY(i)
                if (hypot(x - cx, y - cy) <= hospitalRadius * 1.15f) {
                    scale = 0.88f
                    targetScale = 1f
                    burst(x, y)
                    onHospitalClick?.invoke(x, y)
                    if (!running) invalidate()
                }
                return true
            }
            MotionEvent.ACTION_UP -> {
                performClick()
                return true
            }
        }
        return true
    }

    override fun performClick(): Boolean {
        super.performClick()
        return true
    }

    override fun onDraw(canvas: Canvas) {
        val now = System.nanoTime()
        val dt = if (lastFrame == 0L) 0f else ((now - lastFrame) / 1e9f).coerceAtMost(0.05f)
        lastFrame = now
        time += dt
        scale += (targetScale - scale) * min(1f, dt * 12)

        val r = hospitalRadius
        // Мягкое свечение вокруг больницы
        if (glowFor != r) {
            glowFor = r
            glowPaint.shader = RadialGradient(
                cx, cy, r * 1.3f,
                intArrayOf(Color.argb(200, 255, 255, 255), Color.argb(0, 255, 255, 255)),
                null, Shader.TileMode.CLAMP,
            )
        }
        canvas.drawCircle(cx, cy, r * 1.3f, glowPaint)

        // Падающие таблетки
        rainAcc += rainPerSecond * dt
        while (rainAcc >= 1) {
            rainAcc -= 1
            particles += Particle(
                rainEmojis.random(), Random.nextFloat() * width, -20 * density,
                0f, Random.nextInt(60, 110) * density, 0f, 4f, 4f,
                Random.nextInt(12, 18) * density, null,
            )
        }

        // Врачи на орбитах: внутреннее кольцо по часовой, внешнее — против
        val inner = orbiters.take(INNER_RING)
        val outer = orbiters.drop(INNER_RING)
        drawRing(canvas, inner, outerRadius * 0.78f, time * 0.35f, 22 * density)
        drawRing(canvas, outer, outerRadius * 1.0f, -time * 0.22f, 18 * density)

        // Больница с лёгкой «пульсацией»
        val pulse = 1f + 0.025f * sin(time * 3f)
        val size = r * 1.45f * scale * pulse
        emojiPaint.alpha = 255
        emojiPaint.textSize = size
        canvas.drawText("🏥", cx, cy + size * 0.35f, emojiPaint)

        badgeEmoji?.let {
            val bs = r * 0.5f * (1f + 0.12f * sin(time * 6f))
            emojiPaint.textSize = bs
            canvas.drawText(it, cx + r * 0.85f, cy - r * 0.55f + bs * 0.35f, emojiPaint)
        }

        // Частицы и всплывающие числа
        val it = particles.iterator()
        while (it.hasNext()) {
            val p = it.next()
            p.life -= dt
            if (p.life <= 0 || p.y > height + 40 * density) {
                it.remove()
                continue
            }
            p.vy += p.gravity * dt
            p.x += p.vx * dt
            p.y += p.vy * dt
            val alpha = (255 * (p.life / p.maxLife).coerceIn(0f, 1f)).toInt()
            if (p.color != null) {
                numberPaint.color = p.color
                numberPaint.alpha = alpha
                numberPaint.textSize = p.size
                canvas.drawText(p.text, p.x, p.y, numberPaint)
            } else {
                emojiPaint.alpha = if (p.maxLife > 2f) min(alpha * 2, 200) else alpha
                emojiPaint.textSize = p.size
                canvas.drawText(p.text, p.x, p.y, emojiPaint)
            }
        }
        emojiPaint.alpha = 255

        if (running) postInvalidateOnAnimation()
    }

    private fun drawRing(canvas: Canvas, items: List<String>, radius: Float, phase: Float, size: Float) {
        if (items.isEmpty()) return
        emojiPaint.textSize = size
        emojiPaint.alpha = 255
        val step = (2 * PI / items.size).toFloat()
        items.forEachIndexed { i, emoji ->
            val a = phase + i * step
            val bob = sin(time * 4f + i) * 3 * density
            val x = cx + cos(a) * radius
            val y = cy + sin(a) * radius * 0.85f + bob
            canvas.drawText(emoji, x, y + size * 0.35f, emojiPaint)
        }
    }

    companion object {
        private const val INNER_RING = 14
        const val MAX_ORBITERS = 40
    }
}
