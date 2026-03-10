package com.kogi.mobile

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import java.net.HttpURLConnection
import java.net.URL
import java.util.concurrent.Executors

class MainActivity : AppCompatActivity() {
    private val ioExecutor = Executors.newSingleThreadExecutor()
    private val mainHandler = Handler(Looper.getMainLooper())

    private lateinit var txtHealth: TextView
    private lateinit var txtHost: TextView
    private lateinit var payloadOutput: TextView

    private var activePath: String = "/api/v1/office/dashboard"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        txtHealth = findViewById(R.id.txtHealth)
        txtHost = findViewById(R.id.txtHost)
        payloadOutput = findViewById(R.id.payloadOutput)

        bindModuleButton(R.id.btnDashboard, "/api/v1/office/dashboard")
        bindModuleButton(R.id.btnPortfolio, "/api/v1/office/portfolio")
        bindModuleButton(R.id.btnTimeline, "/api/v1/office/timeline")
        bindModuleButton(R.id.btnWorkspace, "/api/v1/office/workspace")
        bindModuleButton(R.id.btnAssistant, "/api/v1/office/assistant")

        findViewById<Button>(R.id.btnSystem).setOnClickListener {
            activePath = "/api/v1/system"
            fetch(activePath)
        }
        findViewById<Button>(R.id.btnRefresh).setOnClickListener {
            fetch(activePath)
        }

        fetch(activePath)
    }

    override fun onDestroy() {
        super.onDestroy()
        ioExecutor.shutdownNow()
    }

    private fun bindModuleButton(viewId: Int, path: String) {
        findViewById<Button>(viewId).setOnClickListener {
            activePath = path
            fetch(path)
        }
    }

    private fun fetch(path: String) {
        payloadOutput.text = "Loading $path ..."
        ioExecutor.execute {
            val endpoint = "http://10.0.2.2:8080$path"
            val result = runCatching { request(endpoint) }
                .getOrElse { ex -> """{"error":"${ex.message ?: "request_failed"}"}""" }

            mainHandler.post {
                payloadOutput.text = result
                txtHealth.text = if (result.contains("\"status\":\"red\"")) "RED" else if (result.contains("\"status\":\"amber\"")) "AMBER" else "GREEN"
                txtHost.text = if (result.contains("kogi-host")) "kogi-host-001" else "kogi-mobile"
            }
        }
    }

    private fun request(urlString: String): String {
        val connection = URL(urlString).openConnection() as HttpURLConnection
        return try {
            connection.requestMethod = "GET"
            connection.connectTimeout = 6000
            connection.readTimeout = 6000
            connection.inputStream.bufferedReader().use { it.readText() }
        } finally {
            connection.disconnect()
        }
    }
}
