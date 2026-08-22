/// Android Activity Embedding integration for browser WebViews
///
/// This module provides JNI bridge functions to launch, destroy, and navigate
/// WebViewActivity instances from Rust code.

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JString, JValue},
    sys::jint,
    JNIEnv, JavaVM,
};

#[cfg(target_os = "android")]
use anyhow::{Context, Result};

/// Launch a new WebViewActivity with the given URL and tab ID
///
/// This creates a new Activity that will be embedded below the NativeActivity
/// using Android's Activity Embedding feature (30/70 split).
#[cfg(target_os = "android")]
pub fn launch_webview_activity(url: &str, tab_id: i32) -> Result<()> {
    log::info!("Launching WebViewActivity: url={}, tab_id={}", url, tab_id);

    // Get JNI environment
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;

    // Get NativeActivity instance
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    // Convert URL to JString
    let url_jstring = env.new_string(url)
        .context("Failed to create JString for URL")?;

    // Call WryWebViewBridge.launchWebView(activity, url, tabId)
    env.call_static_method(
        "app/dure/sijang/WryWebViewBridge",
        "launchWebView",
        "(Landroid/app/Activity;Ljava/lang/String;I)V",
        &[
            JValue::Object(&activity),
            JValue::Object(&url_jstring),
            JValue::Int(tab_id as jint),
        ],
    ).context("Failed to call WryWebViewBridge.launchWebView")?;

    log::info!("Successfully launched WebViewActivity for tab {}", tab_id);
    Ok(())
}

/// Destroy the currently active WebViewActivity
///
/// This finishes the WebViewActivity and cleans up its resources.
#[cfg(target_os = "android")]
pub fn destroy_webview_activity() -> Result<()> {
    log::info!("Destroying WebViewActivity");

    // Get JNI environment
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;

    // Call WryWebViewBridge.destroyWebView()
    env.call_static_method(
        "app/dure/sijang/WryWebViewBridge",
        "destroyWebView",
        "()V",
        &[],
    ).context("Failed to call WryWebViewBridge.destroyWebView")?;

    log::info!("Successfully destroyed WebViewActivity");
    Ok(())
}

/// Navigate the current WebViewActivity to a new URL
///
/// This loads a new URL in the existing WebViewActivity without recreating it.
#[cfg(target_os = "android")]
pub fn navigate_webview(url: &str) -> Result<()> {
    log::info!("Navigating WebView to: {}", url);

    // Get JNI environment
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;

    // Convert URL to JString
    let url_jstring = env.new_string(url)
        .context("Failed to create JString for URL")?;

    // Call WryWebViewBridge.navigateWebView(url)
    env.call_static_method(
        "app/dure/sijang/WryWebViewBridge",
        "navigateWebView",
        "(Ljava/lang/String;)V",
        &[JValue::Object(&url_jstring)],
    ).context("Failed to call WryWebViewBridge.navigateWebView")?;

    log::info!("Successfully navigated WebView");
    Ok(())
}

/// No-op implementations for non-Android platforms
#[cfg(not(target_os = "android"))]
pub fn launch_webview_activity(_url: &str, _tab_id: i32) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn destroy_webview_activity() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn navigate_webview(_url: &str) -> anyhow::Result<()> {
    Ok(())
}
