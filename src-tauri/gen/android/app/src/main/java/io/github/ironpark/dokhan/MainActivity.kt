package io.github.ironpark.dokhan

import android.os.Bundle
import android.view.View
import androidx.core.graphics.Insets
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    val rootView = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(rootView) { view, windowInsets ->
      val types = WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      val insets = windowInsets.getInsets(types)
      // Edge-to-edge (enforced from Android 15) no longer resizes the window for the
      // keyboard, so lift the WebView above it; the search inputs sit at the bottom.
      val ime = windowInsets.getInsets(WindowInsetsCompat.Type.ime())
      view.setPadding(insets.left, insets.top, insets.right, maxOf(insets.bottom, ime.bottom))

      WindowInsetsCompat.Builder(windowInsets)
        .setInsets(types, Insets.NONE)
        .setInsets(WindowInsetsCompat.Type.ime(), Insets.NONE)
        .build()
    }
    ViewCompat.requestApplyInsets(rootView)
  }
}
