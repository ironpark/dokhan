# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class io.github.ironpark.dokhan.* {
  native <methods>;
}

-keep class io.github.ironpark.dokhan.WryActivity {
  public <init>(...);

  void setWebView(io.github.ironpark.dokhan.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class io.github.ironpark.dokhan.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class io.github.ironpark.dokhan.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class io.github.ironpark.dokhan.RustWebChromeClient,io.github.ironpark.dokhan.RustWebViewClient {
  public <init>(...);
}
