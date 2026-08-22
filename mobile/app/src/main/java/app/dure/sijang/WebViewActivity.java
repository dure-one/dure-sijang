package app.dure.sijang;

import android.os.Bundle;
import androidx.appcompat.app.AppCompatActivity;
import android.util.Log;
import android.widget.TextView;

public class WebViewActivity extends AppCompatActivity {

    private static final String TAG = "WebViewActivity";
    public static WebViewActivity currentInstance = null;

    private String url;
    private int tabId = -1;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        url = getIntent().getStringExtra("url");
        if (url == null) {
            url = "https://dure.app";
        }
        tabId = getIntent().getIntExtra("tab_id", -1);

        Log.i(TAG, "onCreate: URL=" + url + ", tabId=" + tabId);

        // Create a placeholder view (wry WebView integration will be done later via JNI)
        TextView textView = new TextView(this);
        textView.setText("WebView will load here\nURL: " + url + "\nTab ID: " + tabId);
        textView.setTextSize(16f);
        textView.setPadding(32, 32, 32, 32);

        setContentView(textView);
        currentInstance = this;
    }

    @Override
    protected void onDestroy() {
        Log.i(TAG, "onDestroy: tabId=" + tabId);
        if (currentInstance == this) {
            currentInstance = null;
        }
        super.onDestroy();
    }

    public void loadUrl(String newUrl) {
        Log.i(TAG, "loadUrl: " + newUrl + " for tabId=" + tabId);
        url = newUrl;
        // TODO: Implement wry WebView navigation when integrated
    }
}
