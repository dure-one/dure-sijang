package app.dure.sijang;

import android.os.Bundle;
import androidx.appcompat.app.AppCompatActivity;
import android.util.Log;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.webkit.WebSettings;

/**
 * WebViewActivity hosts a native Android WebView in Activity Embedding split.
 * Uses WebView caching via WebViewManager for efficient tab switching.
 */
public class WebViewActivity extends AppCompatActivity {

    private static final String TAG = "WebViewActivity";
    public static WebViewActivity currentInstance = null;

    private String url;
    private int tabId = -1;
    private WebView webView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        url = getIntent().getStringExtra("url");
        if (url == null) {
            url = "https://dure.app";
        }
        tabId = getIntent().getIntExtra("tab_id", -1);

        Log.i(TAG, "onCreate: URL=" + url + ", tabId=" + tabId);

        // Get or create cached WebView from WebViewManager
        webView = WebViewManager.getInstance().getOrCreateWebView(this, tabId, url);

        if (webView.getParent() != null) {
            Log.w(TAG, "WebView already has a parent, using as-is");
        }

        setContentView(webView);

        Log.i(TAG, "WebView created/retrieved for tab " + tabId + " (cache size: " +
              WebViewManager.getInstance().getCacheSize() + ")");

        currentInstance = this;
    }

    @Override
    protected void onDestroy() {
        Log.i(TAG, "onDestroy: tabId=" + tabId);

        // Detach WebView from this Activity (but keep in cache)
        if (webView != null && webView.getParent() != null) {
            ((android.view.ViewGroup) webView.getParent()).removeView(webView);
        }

        if (currentInstance == this) {
            currentInstance = null;
        }
        super.onDestroy();
    }

    public void loadUrl(String newUrl) {
        Log.i(TAG, "loadUrl: " + newUrl + " for tabId=" + tabId);
        url = newUrl;
        if (webView != null) {
            webView.loadUrl(newUrl);
        }
    }
}
