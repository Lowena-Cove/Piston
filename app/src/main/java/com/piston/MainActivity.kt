package com.piston

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import android.widget.*
import android.view.View
import android.content.Intent
import android.net.Uri
import android.os.Environment
import android.provider.DocumentsContract
import android.app.AlertDialog
import java.io.File

class MainActivity : AppCompatActivity() {
    private lateinit var statusText: TextView
    private lateinit var launchButton: Button
    private lateinit var selectIsoButton: Button
    private lateinit var systemInfoButton: Button
    private var selectedIsoPath: String = "/sdcard/Deck.iso"
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Create layout programmatically for better compatibility
        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 32, 32, 32)
        }
        
        // Title
        val titleText = TextView(this).apply {
            text = "Piston - SteamOS Launcher (Legacy Edition)"
            textSize = 20f
            setPadding(0, 0, 0, 24)
        }
        layout.addView(titleText)
        
        // Status text
        statusText = TextView(this).apply {
            text = "Ready to launch SteamOS. Select an ISO file first."
            setPadding(0, 0, 0, 16)
        }
        layout.addView(statusText)
        
        // ISO path display
        val isoPathText = TextView(this).apply {
            text = "ISO Path: $selectedIsoPath"
            textSize = 12f
            setPadding(0, 0, 0, 16)
        }
        layout.addView(isoPathText)
        
        // Select ISO button
        selectIsoButton = Button(this).apply {
            text = "Select SteamOS ISO"
            setOnClickListener {
                selectIsoFile()
            }
        }
        layout.addView(selectIsoButton)
        
        // System info button
        systemInfoButton = Button(this).apply {
            text = "Check System Compatibility"
            setOnClickListener {
                showSystemInfo()
            }
        }
        layout.addView(systemInfoButton)
        
        // Launch button
        launchButton = Button(this).apply {
            text = "Launch SteamOS"
            setOnClickListener {
                launchSteamOS()
            }
        }
        layout.addView(launchButton)
        
        // Instructions
        val instructionsText = TextView(this).apply {
            text = """
                Instructions:
                1. Download a SteamOS recovery image
                2. Select the ISO file using the button above
                3. Check system compatibility
                4. Launch SteamOS
                
                This version is optimized for older and virtualized systems.
                It will automatically detect your system capabilities and
                choose the best launch method.
            """.trimIndent()
            textSize = 12f
            setPadding(0, 24, 0, 0)
        }
        layout.addView(instructionsText)
        
        setContentView(layout)
        
        // Check if default ISO exists
        checkDefaultIso()
    }
    
    private fun checkDefaultIso() {
        val defaultIso = File(selectedIsoPath)
        if (!defaultIso.exists()) {
            statusText.text = "Default ISO not found. Please select a SteamOS ISO file."
            launchButton.isEnabled = false
        } else {
            statusText.text = "Default ISO found: ${defaultIso.name}"
            launchButton.isEnabled = true
        }
    }
    
    private fun selectIsoFile() {
        // For simplicity, show a dialog to enter path manually
        // In a real implementation, you'd use file picker
        val builder = AlertDialog.Builder(this)
        builder.setTitle("Enter ISO Path")
        
        val input = EditText(this)
        input.setText(selectedIsoPath)
        builder.setView(input)
        
        builder.setPositiveButton("OK") { _, _ ->
            selectedIsoPath = input.text.toString()
            val isoFile = File(selectedIsoPath)
            if (isoFile.exists()) {
                statusText.text = "ISO selected: ${isoFile.name}"
                launchButton.isEnabled = true
            } else {
                statusText.text = "ISO file not found: $selectedIsoPath"
                launchButton.isEnabled = false
            }
        }
        
        builder.setNegativeButton("Cancel") { dialog, _ ->
            dialog.cancel()
        }
        
        builder.show()
    }
    
    private fun showSystemInfo() {
        // Get system info from native code
        val systemInfo = SteamOSNative.getSystemInfo()
        
        val builder = AlertDialog.Builder(this)
        builder.setTitle("System Compatibility")
        builder.setMessage(systemInfo)
        builder.setPositiveButton("OK") { dialog, _ ->
            dialog.dismiss()
        }
        builder.show()
    }
    
    private fun launchSteamOS() {
        if (!File(selectedIsoPath).exists()) {
            Toast.makeText(this, "ISO file not found: $selectedIsoPath", Toast.LENGTH_LONG).show()
            return
        }
        
        statusText.text = "Launching SteamOS... This may take a while."
        launchButton.isEnabled = false
        
        // Launch in background thread
        Thread {
            try {
                val result = SteamOSNative.launchSteamOS(selectedIsoPath)
                
                runOnUiThread {
                    launchButton.isEnabled = true
                    when {
                        result == 0 -> {
                            statusText.text = "SteamOS launched successfully!"
                            Toast.makeText(this, "SteamOS session completed", Toast.LENGTH_SHORT).show()
                        }
                        result == -1 -> {
                            statusText.text = "Failed to detect system capabilities"
                            Toast.makeText(this, "System detection failed", Toast.LENGTH_LONG).show()
                        }
                        result == -2 -> {
                            statusText.text = "Failed to create launch script"
                            Toast.makeText(this, "Script creation failed", Toast.LENGTH_LONG).show()
                        }
                        else -> {
                            statusText.text = "Launch failed with code: $result"
                            Toast.makeText(this, "Launch failed: $result", Toast.LENGTH_LONG).show()
                        }
                    }
                }
            } catch (e: Exception) {
                runOnUiThread {
                    launchButton.isEnabled = true
                    statusText.text = "Error: ${e.message}"
                    Toast.makeText(this, "Error: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
        }.start()
    }
}
