package app.dure.sijang;

import android.os.Bundle;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.webkit.WebSettings;
import androidx.appcompat.app.AppCompatActivity;
import android.util.Log;

public class WebViewActivity extends AppCompatActivity {
    private static final String TAG = "WebViewActivity";
    private WebView webView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        Log.d(TAG, "onCreate: Initializing WebView");

        // Create WebView programmatically
        webView = new WebView(this);
        setContentView(webView);

        // Configure WebView settings
        WebSettings settings = webView.getSettings();

        // Get optional settings from Intent (default: true)
        boolean enableJs = getIntent().getBooleanExtra("enable_js", true);
        boolean enableDomStorage = getIntent().getBooleanExtra("enable_dom_storage", true);

        settings.setJavaScriptEnabled(enableJs);
        settings.setDomStorageEnabled(enableDomStorage);
        settings.setDatabaseEnabled(true);
        settings.setCacheMode(WebSettings.LOAD_DEFAULT);

        // Enable mixed content (HTTPS pages with HTTP resources)
        settings.setMixedContentMode(WebSettings.MIXED_CONTENT_ALWAYS_ALLOW);

        // Set WebViewClient to handle navigation within WebView
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public void onPageFinished(WebView view, String url) {
                super.onPageFinished(view, url);
                Log.d(TAG, "Page loaded: " + url);
            }
        });

        // Load URL from Intent
        String url = getIntent().getStringExtra("url");
        if (url != null && !url.isEmpty()) {
            Log.d(TAG, "Loading URL: " + url);
            webView.loadUrl(url);
        } else {
            Log.e(TAG, "No URL provided in Intent");
            finish();
        }
    }

    @Override
    public void onBackPressed() {
        // If WebView can navigate back, do that instead of finishing Activity
        if (webView != null && webView.canGoBack()) {
            Log.d(TAG, "Navigating WebView back");
            webView.goBack();
        } else {
            Log.d(TAG, "Finishing WebViewActivity");
            super.onBackPressed();
        }
    }

    @Override
    protected void onDestroy() {
        Log.d(TAG, "onDestroy: Cleaning up WebView");
        if (webView != null) {
            webView.destroy();
            webView = null;
        }
        super.onDestroy();
    }
}
