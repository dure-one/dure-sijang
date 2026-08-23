package app.dure.sijang;

import android.app.Activity;
import android.util.Log;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.webkit.WebSettings;
import java.util.HashMap;
import java.util.Map;

/**
 * Singleton manager for WebView caching to enable efficient tab switching.
 * Caches up to 10 WebViews with LRU eviction.
 */
public class WebViewManager {

    private static final String TAG = "WebViewManager";
    private static final int MAX_CACHED_WEBVIEWS = 10;

    private static WebViewManager instance;

    private final Map<Integer, WebView> webViewCache;
    private final Map<Integer, Long> lastAccessTime;

    private WebViewManager() {
        webViewCache = new HashMap<>();
        lastAccessTime = new HashMap<>();
    }

    public static synchronized WebViewManager getInstance() {
        if (instance == null) {
            instance = new WebViewManager();
        }
        return instance;
    }

    public WebView getOrCreateWebView(Activity activity, int tabId, String url) {
        Log.i(TAG, "getOrCreateWebView: tabId=" + tabId + ", url=" + url);

        // Return cached WebView if exists
        if (webViewCache.containsKey(tabId)) {
            WebView cachedView = webViewCache.get(tabId);
            lastAccessTime.put(tabId, System.currentTimeMillis());
            Log.i(TAG, "Returning cached WebView for tab " + tabId);
            return cachedView;
        }

        // Check cache size limit
        if (webViewCache.size() >= MAX_CACHED_WEBVIEWS) {
            evictLeastRecentlyUsed();
        }

        // Create new WebView
        WebView webView = new WebView(activity);
        configureWebView(webView);
        webView.loadUrl(url);

        // Cache the WebView
        webViewCache.put(tabId, webView);
        lastAccessTime.put(tabId, System.currentTimeMillis());

        Log.i(TAG, "Created new WebView for tab " + tabId + " (cache size: " + webViewCache.size() + ")");
        return webView;
    }

    private void configureWebView(WebView webView) {
        WebSettings settings = webView.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setLoadWithOverviewMode(true);
        settings.setUseWideViewPort(true);
        settings.setBuiltInZoomControls(true);
        settings.setDisplayZoomControls(false);

        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, String url) {
                view.loadUrl(url);
                return true;
            }
        });
    }

    public void removeWebView(int tabId) {
        Log.i(TAG, "removeWebView: tabId=" + tabId);
        WebView webView = webViewCache.remove(tabId);
        lastAccessTime.remove(tabId);
        if (webView != null) {
            webView.destroy();
            Log.i(TAG, "Destroyed WebView for tab " + tabId + " (remaining: " + webViewCache.size() + ")");
        }
    }

    public void clearCache() {
        Log.i(TAG, "clearCache: destroying " + webViewCache.size() + " WebViews");
        for (WebView webView : webViewCache.values()) {
            if (webView != null) {
                webView.destroy();
            }
        }
        webViewCache.clear();
        lastAccessTime.clear();
    }

    private void evictLeastRecentlyUsed() {
        if (lastAccessTime.isEmpty()) return;
        int lruTabId = -1;
        long oldestTime = Long.MAX_VALUE;
        for (Map.Entry<Integer, Long> entry : lastAccessTime.entrySet()) {
            if (entry.getValue() < oldestTime) {
                oldestTime = entry.getValue();
                lruTabId = entry.getKey();
            }
        }
        if (lruTabId != -1) {
            Log.i(TAG, "Evicting LRU WebView for tab " + lruTabId);
            removeWebView(lruTabId);
        }
    }

    public int getCacheSize() {
        return webViewCache.size();
    }
}
