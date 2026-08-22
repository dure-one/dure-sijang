package app.dure.sijang;

import android.app.Activity;
import android.content.Intent;
import android.util.Log;

public class WryWebViewBridge {

    private static final String TAG = "WryWebViewBridge";

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
}
