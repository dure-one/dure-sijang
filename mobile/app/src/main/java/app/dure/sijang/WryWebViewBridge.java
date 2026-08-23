package app.dure.sijang;

import android.app.Activity;
import android.content.Intent;
import android.util.Log;

public class WryWebViewBridge {

    private static final String TAG = "WryWebViewBridge";

    // JNI native methods (implemented in Rust)
    private static native void createWryWebViewNative(Activity activity, String url, int tabId);
    private static native void switchWryWebViewNative(int tabId);

    // Load native library
    static {
        System.loadLibrary("dure_sijang");
    }

    public static void launchWebView(Activity activity, String url, int tabId) {
        Log.i(TAG, "launchWebView: url=" + url + ", tabId=" + tabId);

        Intent intent = new Intent(activity, WebViewActivity.class);
        intent.putExtra("url", url);
        intent.putExtra("tab_id", tabId);

        try {
            activity.startActivity(intent);
            Log.i(TAG, "Successfully launched WebViewActivity");
        } catch (Exception e) {
            Log.e(TAG, "Failed to launch WebViewActivity", e);
            throw e;
        }
    }

    /**
     * Create wry WebView via JNI (called from WebViewActivity)
     */
    public static void createWryWebView(Activity activity, String url, int tabId) {
        Log.i(TAG, "createWryWebView: url=" + url + ", tabId=" + tabId);
        try {
            createWryWebViewNative(activity, url, tabId);
            Log.i(TAG, "Successfully created wry WebView via JNI");
        } catch (Exception e) {
            Log.e(TAG, "Failed to create wry WebView via JNI", e);
        }
    }

    /**
     * Switch to different tab's wry WebView
     */
    public static void switchWryWebView(int tabId) {
        Log.i(TAG, "switchWryWebView: tabId=" + tabId);
        try {
            switchWryWebViewNative(tabId);
            Log.i(TAG, "Successfully switched wry WebView via JNI");
        } catch (Exception e) {
            Log.e(TAG, "Failed to switch wry WebView via JNI", e);
        }
    }

    public static void destroyWebView() {
        Log.i(TAG, "destroyWebView called");

        if (WebViewActivity.currentInstance != null) {
            try {
                WebViewActivity.currentInstance.finish();
                Log.i(TAG, "Successfully destroyed WebViewActivity");
            } catch (Exception e) {
                Log.e(TAG, "Failed to destroy WebViewActivity", e);
            }
        } else {
            Log.w(TAG, "No WebViewActivity instance to destroy");
        }
    }

    public static void navigateWebView(String url) {
        Log.i(TAG, "navigateWebView: url=" + url);

        if (WebViewActivity.currentInstance != null) {
            try {
                WebViewActivity.currentInstance.loadUrl(url);
                Log.i(TAG, "Successfully navigated WebView");
            } catch (Exception e) {
                Log.e(TAG, "Failed to navigate WebView", e);
            }
        } else {
            Log.w(TAG, "No WebViewActivity instance to navigate");
        }
    }

    public static void updateSplitRatio(float ratio) {
        DureSijangApplication.updateSplitRatio(ratio);
    }
}
