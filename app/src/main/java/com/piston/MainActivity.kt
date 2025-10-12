package com.piston

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import android.widget.Button
import android.widget.Toast

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // SteamOS Launch Button
        val btnLaunchSteamOS = findViewById<Button>(R.id.btnLaunchSteamOS)
        btnLaunchSteamOS.setOnClickListener {
            val result = SteamOSNative.launchSteamOS("/sdcard/Deck.iso")
            Toast.makeText(this, "Launch result: $result", Toast.LENGTH_SHORT).show()
        }

        // Termux Org Page Button
        val btnTermuxOrg = findViewById<Button>(R.id.btnTermuxOrg)
        btnTermuxOrg.setOnClickListener {
            openTermuxOrgPage()
        }
    }

    private fun openTermuxOrgPage() {
        try {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse("https://github.com/termux"))
            startActivity(intent)
            Toast.makeText(this, "Opening Termux Org Page...", Toast.LENGTH_SHORT).show()
        } catch (e: Exception) {
            Toast.makeText(this, "Error opening Termux Org Page: ${e.message}", Toast.LENGTH_LONG).show()
        }
    }
}
