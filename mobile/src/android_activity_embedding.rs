/// Android Activity Embedding integration for browser WebViews
///
/// This module provides JNI bridge functions to launch, destroy, and navigate
/// WebViewActivity instances from Rust code. Uses wry for WebView creation.

#[cfg(target_os = "android")]
use jni::{
    objects::{JClass, JObject, JString, JValue},
    sys::jint,
    JNIEnv, JavaVM,
};

#[cfg(target_os = "android")]
use anyhow::{Context, Result};

#[cfg(target_os = "android")]
use std::sync::Mutex;

#[cfg(target_os = "android")]
static WEBVIEW_MANAGER: once_cell::sync::Lazy<Mutex<crate::android_webview::AndroidWebViewManager>> =
    once_cell::sync::Lazy::new(|| Mutex::new(crate::android_webview::AndroidWebViewManager::new()));

/// Helper to load a class using the Activity's class loader
#[cfg(target_os = "android")]
fn load_class_from_activity<'local>(
    env: &mut JNIEnv<'local>,
    activity: &JObject,
    class_name: &str,
) -> Result<JClass<'local>> {
    // Get the Application context (not Activity) to get the right class loader
    let app_context = env.call_method(
        activity,
        "getApplicationContext",
        "()Landroid/content/Context;",
        &[],
    ).context("Failed to get Application context")?;
    let app_context = app_context.l()
        .context("Application context is not an object")?;

    // Get the context's class loader
    let class_loader = env.call_method(
        app_context,
        "getClassLoader",
        "()Ljava/lang/ClassLoader;",
        &[],
    ).context("Failed to get ClassLoader from context")?;
    let class_loader = class_loader.l()
        .context("ClassLoader is not an object")?;

    // Load the class using the class loader
    let class_name_jstring = env.new_string(class_name)
        .context("Failed to create class name string")?;

    // Try to load the class - this may throw ClassNotFoundException
    let class_obj_result = env.call_method(
        class_loader,
        "loadClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        &[JValue::Object(&class_name_jstring)],
    );

    // Check for exceptions and clear them
    if let Err(e) = class_obj_result {
        // Clear any pending exception
        if env.exception_check().unwrap_or(false) {
            env.exception_clear().ok();
        }
        return Err(e).context(format!("Failed to load class: {}", class_name));
    }

    let class_jobject = class_obj_result.unwrap().l()
        .context("Class is not an object")?;

    // Cast JObject to JClass
    Ok(unsafe { JClass::from_raw(class_jobject.as_raw()) })
}

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

    // Load WryWebViewBridge class using the activity's class loader
    let bridge_class = load_class_from_activity(&mut env, &activity, "app.dure.sijang.WryWebViewBridge")?;

    // Convert URL to JString
    let url_jstring = env.new_string(url)
        .context("Failed to create JString for URL")?;

    // Call WryWebViewBridge.launchWebView(activity, url, tabId)
    env.call_static_method(
        bridge_class,
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

    // Get NativeActivity instance
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    // Load WryWebViewBridge class using the activity's class loader
    let bridge_class = load_class_from_activity(&mut env, &activity, "app.dure.sijang.WryWebViewBridge")?;

    // Call WryWebViewBridge.destroyWebView()
    env.call_static_method(
        bridge_class,
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

    // Get NativeActivity instance
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    // Load WryWebViewBridge class using the activity's class loader
    let bridge_class = load_class_from_activity(&mut env, &activity, "app.dure.sijang.WryWebViewBridge")?;

    // Convert URL to JString
    let url_jstring = env.new_string(url)
        .context("Failed to create JString for URL")?;

    // Call WryWebViewBridge.navigateWebView(url)
    env.call_static_method(
        bridge_class,
        "navigateWebView",
        "(Ljava/lang/String;)V",
        &[JValue::Object(&url_jstring)],
    ).context("Failed to call WryWebViewBridge.navigateWebView")?;

    log::info!("Successfully navigated WebView");
    Ok(())
}

/// JNI callback: Create wry WebView in WebViewActivity context
#[no_mangle]
#[cfg(target_os = "android")]
pub extern "C" fn Java_app_dure_sijang_WryWebViewBridge_createWryWebViewNative(
    mut env: JNIEnv,
    _class: JClass,
    _activity: JObject,
    url: JString,
    tab_id: jint,
) {
    let url_str = match env.get_string(&url) {
        Ok(s) => s.to_string_lossy().to_string(),
        Err(e) => { log::error!("JNI: Failed to convert URL: {}", e); return; }
    };

    log::info!("JNI: createWryWebViewNative tabId={}, url={}", tab_id, url_str);

    let ctx = ndk_context::android_context();
    use raw_window_handle::{AndroidNdkWindowHandle, RawWindowHandle, HasWindowHandle};

    struct WindowWrapper { handle: RawWindowHandle }
    impl HasWindowHandle for WindowWrapper {
        fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
            Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(self.handle) })
        }
    }

    let android_handle = unsafe {
        RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(
            std::ptr::NonNull::new_unchecked(ctx.context().cast())
        ))
    };

    let window = WindowWrapper { handle: android_handle };
    let mut manager = match WEBVIEW_MANAGER.lock() {
        Ok(m) => m,
        Err(e) => { log::error!("JNI: Failed to lock: {}", e); return; }
    };

    match manager.get_or_create_webview(tab_id, &url_str, &window) {
        Ok(_) => log::info!("JNI: Created wry WebView for tab {}", tab_id),
        Err(e) => log::error!("JNI: Failed to create: {}", e),
    }
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
