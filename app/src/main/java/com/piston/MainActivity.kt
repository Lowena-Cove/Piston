package com.piston

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import android.widget.Button
import android.widget.Toast

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // TODO: Replace with actual layout
        val button = Button(this).apply {
            text = "Launch SteamOS"
            setOnClickListener {
                val result = SteamOSNative.launchSteamOS("/sdcard/Deck.iso")
                Toast.makeText(context, "Launch result: $result", Toast.LENGTH_SHORT).show()
            }
        }
        setContentView(button)
    }
}
